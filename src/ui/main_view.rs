use eframe::egui;
use crate::app::ImageViewerApp;
use crate::state::{ViewMode, ResizeEdge, SelectionDragMode};

/* How close (in screen pixels) the pointer must be to a selection box edge
 * before it's considered a "grab" of that edge rather than a click inside
 * or outside the box.
 */
const EDGE_GRAB_THRESHOLD: f32 = 6.0;

/* Returns which edge of `rect` (if any) is within `threshold` pixels of `pos`.
 * Checked in Left/Right/Top/Bottom priority order, so near-corner hits pick
 * whichever axis is listed first rather than being ambiguous.
 */
fn detect_edge(rect: egui::Rect, pos: egui::Pos2, threshold: f32) -> Option<ResizeEdge>
{
    let within_y = pos.y >= rect.top() - threshold && pos.y <= rect.bottom() + threshold;
    let within_x = pos.x >= rect.left() - threshold && pos.x <= rect.right() + threshold;

    if within_y && (pos.x - rect.left()).abs() <= threshold
    {
        Some(ResizeEdge::Left)
    }
    else if within_y && (pos.x - rect.right()).abs() <= threshold
    {
        Some(ResizeEdge::Right)
    }
    else if within_x && (pos.y - rect.top()).abs() <= threshold
    {
        Some(ResizeEdge::Top)
    }
    else if within_x && (pos.y - rect.bottom()).abs() <= threshold
    {
        Some(ResizeEdge::Bottom)
    }
    else
    {
        None
    }
}

/* Draws the central image viewport: the image itself (scaled/panned/flipped
 * appropriately), and handles all of the viewport-level interactions -
 * mouse-wheel and keyboard zoom, right-click-drag panning, and drawing /
 * resizing / clicking a rectangular selection box for cropping or
 * "zoom to selection".
 *
 * If no image is loaded, shows a simple placeholder prompt instead.
 */
pub fn show_main_view(app: &mut ImageViewerApp, ui: &mut egui::Ui)
{
    let frame = egui::Frame::new().fill(ui.style().visuals.panel_fill);
    
    egui::CentralPanel::default().frame(frame).show(ui, |ui|
    {
        ui.set_clip_rect(ui.max_rect());

        if let Some(ref texture) = app.texture
        {
            let available_size = ui.available_size();
            let img_size = texture.size_vec2();

            /* In FitToWindow mode, the scale is recomputed every frame from
             * the available panel size so the image always fits regardless
             * of window size, and `zoom_factor` is kept up to date so it's
             * correct if the user later switches to ActualSize/Custom. In
             * ActualSize/Custom modes, the stored `zoom_factor` is used
             * as-is (set by zoom gestures or the view-mode toggle).
             */
            let current_scale = match app.view_mode
            {
                ViewMode::FitToWindow =>
                {
                    let scale = (available_size.x / img_size.x).min(available_size.y / img_size.y);
                    app.zoom_factor = scale;
                    scale
                }
                ViewMode::ActualSize | ViewMode::Custom => app.zoom_factor,
            };

            let scaled_size = img_size * current_scale;
            let center = ui.max_rect().center() + app.pan_offset;
            let image_rect = egui::Rect::from_center_size(center, scaled_size);
            app.image_rect = Some(image_rect);

            let response = ui.allocate_rect(ui.max_rect(), egui::Sense::click_and_drag());

            /* Handle Mouse Scroll / Zoom (including Ctrl + mwheel)
             *
             * Treats either a non-zero scroll wheel delta OR a pinch/ctrl-
             * scroll "zoom delta" as a zoom request, normalizing both into a
             * single (scroll_active, scroll_up) pair so the rest of the
             * logic below doesn't need to care which input triggered it.
             */
            let (scroll_active, scroll_up) = ui.input(|i| {
                let scroll_y = i.smooth_scroll_delta.y;
                let zoom_d = i.zoom_delta();
                if scroll_y != 0.0 {
                    (true, scroll_y > 0.0)
                } else if (zoom_d - 1.0).abs() > 0.001 {
                    (true, zoom_d > 1.0)
                } else {
                    (false, false)
                }
            });

            if scroll_active && response.hovered()
            {
                let mouse_pos = response.hover_pos().unwrap_or_else(|| ui.max_rect().center());
                let screen_center = ui.max_rect().center();
                let mouse_offset_from_center = mouse_pos - screen_center;

                let old_scale = app.zoom_factor;
                let zoom_modifier = if ui.input(|i| i.modifiers.ctrl) { app.default_zoom_step_modified } else { app.default_zoom_step };
                let factor = if scroll_up { 1.0 + zoom_modifier } else { 1.0 / (1.0 + zoom_modifier) };
                let new_scale = old_scale * factor;

                /* Anchor the zoom to the mouse position rather than the
                 * viewport center: re-derive the pan offset so the point
                 * under the cursor stays fixed on screen before/after the
                 * scale change.
                 */
                if old_scale > 0.0
                {
                    app.pan_offset = mouse_offset_from_center - (mouse_offset_from_center - app.pan_offset) * factor;
                    app.zoom_factor = new_scale;
                    app.view_mode = ViewMode::Custom;
                }
            }

            /* Handle Keyboard Zoom (+ / - / =) - Anchored to Viewport Center
             *
             * Unlike scroll-zoom, keyboard zoom always anchors to the
             * viewport center (there's no mouse position to anchor to), so
             * the pan offset is simply scaled by the same zoom factor.
             */
            let key_zoom_in = ui.input(|i| i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals));
            let key_zoom_out = ui.input(|i| i.key_pressed(egui::Key::Minus));

            if key_zoom_in || key_zoom_out
            {
                let old_scale = app.zoom_factor;
                let zoom_modifier = if ui.input(|i| i.modifiers.ctrl) { 0.05 } else { 0.01 };
                let factor = if key_zoom_in { 1.0 + zoom_modifier } else { 1.0 / (1.0 + zoom_modifier) };
                let new_scale = old_scale * factor;

                if old_scale > 0.0
                {
                    app.pan_offset *= factor;
                    app.zoom_factor = new_scale;
                    app.view_mode = ViewMode::Custom;
                }
            }

            /* Flips are implemented by swapping the UV coordinates used to
             * sample the texture (GPU-side), rather than mutating the pixel
             * buffer - swapping min/max on an axis mirrors that axis.
             */
            let u_min = if app.flipped_h { 1.0 } else { 0.0 };
            let u_max = if app.flipped_h { 0.0 } else { 1.0 };
            let v_min = if app.flipped_v { 1.0 } else { 0.0 };
            let v_max = if app.flipped_v { 0.0 } else { 1.0 };

            let uv = egui::Rect::from_min_max(
                egui::pos2(u_min, v_min),
                egui::pos2(u_max, v_max),
            );

            ui.painter().image(
                texture.id(),
                image_rect,
                uv,
                egui::Color32::WHITE,
            );

            // Right-click drag panning
            if response.dragged_by(egui::PointerButton::Secondary)
            {
                app.pan_offset += response.drag_delta();
            }

            /* Check existing valid selection box.
             *
             * A selection only "counts" as an existing box (eligible for
             * edge-resize / click-to-zoom) once it's bigger than a tiny
             * 5x5px threshold, filtering out accidental single-pixel clicks
             * being treated as a real selection.
             */
            let existing_sel = if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
                let r = egui::Rect::from_two_pos(start, end);
                if r.width() > 5.0 && r.height() > 5.0 {
                    Some(r)
                } else {
                    None
                }
            } else {
                None
            };

            // Show a resize cursor when hovering an edge of the existing selection
            if let Some(sel) = existing_sel
            {
                let currently_resizing = match app.selection_drag {
                    SelectionDragMode::Resizing(edge) => Some(edge),
                    _ => None,
                };

                // While actively resizing, keep showing the cursor for the
                // edge being dragged even if the pointer has since strayed
                // outside the edge-grab band; otherwise, detect fresh from
                // hover position.
                let hovered_edge = currently_resizing.or_else(||
                {
                    response.hover_pos().and_then(|p| detect_edge(sel, p, EDGE_GRAB_THRESHOLD))
                });

                if let Some(edge) = hovered_edge
                {
                    let cursor = match edge {
                        ResizeEdge::Left | ResizeEdge::Right => egui::CursorIcon::ResizeHorizontal,
                        ResizeEdge::Top | ResizeEdge::Bottom => egui::CursorIcon::ResizeVertical,
                    };
                    ui.ctx().set_cursor_icon(cursor);
                }
            }

            /* Handle Primary (Left) Click / Drag for Selection Box
             *
             * The drag's intent (resize an edge / draw a new box / do nothing) is decided
             * ONCE here, when the drag starts, and stored in app.selection_drag. Every
             * subsequent frame of the drag (below) just reads that stored mode instead of
             * re-guessing it - this is what prevents a click that narrowly misses the edge
             * band from silently falling back into "move the box's corner to the mouse",
             * which is what caused two edges to jump at once.
             */
            if response.drag_started_by(egui::PointerButton::Primary)
            {
                let press_origin = ui.input(|i| i.pointer.press_origin());
                let current_pos = response.interact_pointer_pos();

                if let (Some(origin), Some(pos)) = (press_origin, current_pos)
                {
                    if let Some(edge) = existing_sel.and_then(|r| detect_edge(r, origin, EDGE_GRAB_THRESHOLD))
                    {
                        app.selection_drag = SelectionDragMode::Resizing(edge);
                    } else if existing_sel.map_or(false, |r| r.contains(origin))
                    {
                        app.selection_drag = SelectionDragMode::Idle;
                    } else
                    {
                        app.selection_start = Some(origin);   // <- true click point
                        app.selection_end = Some(pos);        // <- wherever the drag has reached so far
                        app.selection_drag = SelectionDragMode::Drawing;
                    }
                }
            }

            if response.dragged_by(egui::PointerButton::Primary)
            {
                if let Some(pos) = response.interact_pointer_pos() {
                    match app.selection_drag {
                        SelectionDragMode::Resizing(edge) => {
                            if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
                                let rect = egui::Rect::from_two_pos(start, end);
                                let mut new_min = rect.min;
                                let mut new_max = rect.max;

                                match edge {
                                    ResizeEdge::Left => new_min.x = pos.x,
                                    ResizeEdge::Right => new_max.x = pos.x,
                                    ResizeEdge::Top => new_min.y = pos.y,
                                    ResizeEdge::Bottom => new_max.y = pos.y,
                                }

                                /* Re-normalize in case the dragged edge crossed over the
                                 * opposite edge (e.g. dragging the left side past the right).
                                 */
                                let resized = egui::Rect::from_two_pos(new_min, new_max);
                                app.selection_start = Some(resized.min);
                                app.selection_end = Some(resized.max);
                            }
                        }
                        SelectionDragMode::Drawing => {
                            app.selection_end = Some(pos);
                        }
                        SelectionDragMode::Idle => {
                            // Drag started somewhere that isn't an edge or empty space
                            // (e.g. inside the existing box) - intentionally do nothing.
                        }
                    }
                }
            }

            if response.drag_stopped_by(egui::PointerButton::Primary)
            {
                app.selection_drag = SelectionDragMode::Idle;
            }

            if let (Some(start), Some(end)) = (app.selection_start, app.selection_end)
            {
                let sel_rect = egui::Rect::from_two_pos(start, end);
                
                // Draw a white stroke with a black outline just outside it,
                // giving the marquee visibility against both light and dark
                // image content ("marching ants"-style contrast border).
                if sel_rect.width() > 1.0 && sel_rect.height() > 1.0 {
                    ui.painter().rect_stroke(sel_rect, 0.0, (1.0, egui::Color32::WHITE), egui::StrokeKind::Middle);
                    ui.painter().rect_stroke(sel_rect.expand(1.0), 0.0, (1.0, egui::Color32::BLACK), egui::StrokeKind::Middle);
                }

                /* A plain click (not a drag) inside a large-enough existing
                 * selection zooms the view to fill the viewport with that
                 * selection; a click anywhere else clears the selection
                 * entirely.
                 */
                if response.clicked_by(egui::PointerButton::Primary)
                {
                    if let Some(pos) = response.interact_pointer_pos()
                    {
                        if sel_rect.width() > 5.0 && sel_rect.height() > 5.0 && sel_rect.contains(pos)
                        {
                            app.view_mode = ViewMode::Custom;
                            
                            // Scale so the selection rect exactly fills the
                            // available viewport (uniformly, preserving
                            // aspect ratio via the smaller of the two axis
                            // scale factors).
                            let scale_x = available_size.x / sel_rect.width();
                            let scale_y = available_size.y / sel_rect.height();
                            let zoom_multiplier = scale_x.min(scale_y);
                            
                            app.zoom_factor *= zoom_multiplier;

                            // Re-center the pan offset on the selection's
                            // center rather than the old viewport center.
                            let screen_center = ui.max_rect().center();
                            let sel_center = sel_rect.center();
                            let sel_offset_from_screen_center = sel_center - screen_center;
                            
                            app.pan_offset = (app.pan_offset - sel_offset_from_screen_center) * zoom_multiplier;
                            
                            app.selection_start = None;
                            app.selection_end = None;
                        }
                        else
                        {
                            app.selection_start = None;
                            app.selection_end = None;
                        }
                    }
                }
            }
         }
        else
        {
            ui.centered_and_justified(|ui|
            {
                ui.label("Drop or open an image to start");
            });
        }
    });
}