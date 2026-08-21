use eframe::egui;
use crate::app::ImageViewerApp;
use crate::image_utils;
use crate::state::AdjustState;

// ── Entry point ───────────────────────────────────────────────────────────────

/* Draws the "Image Adjustments" dialog as its own OS-level viewport (a
 * separate window) when `app.show_adjust_dialog` is true, and handles all
 * of its interaction logic.
 *
 * The dialog is laid out as: a side-by-side "Original" vs "Preview"
 * thumbnail pair up top, two columns of adjustment sliders in the middle,
 * and a row of Reset All / Apply / Cancel buttons at the bottom.
 *
 * Because `ctx.show_viewport_immediate` runs its closure with a fresh
 * `viewport_ctx` (not the outer `app`/`ctx`), all state mutations that need
 * `app` are deferred: the closure only sets local flags (`changed`,
 * `apply`, `reset_all`, `cancel`), and the actual side effects (updating
 * the preview texture, applying adjustments to the real image, closing the
 * dialog) happen afterward, once the closure has returned and `app` can be
 * borrowed freely again.
 */
pub fn show_adjust_dialog(app: &mut ImageViewerApp, ctx: &egui::Context)
{
    //let mut open = app.show_adjust_dialog;

    if !app.show_adjust_dialog
    {
        return;
    }

    let mut changed   = false;
    let mut apply     = false;
    let mut reset_all = false;
    let mut cancel    = false;
    let dialog_size = egui::vec2(820.0, 600.0);

    let mut builder = egui::ViewportBuilder::default()
        .with_title("Image Adjustments")
        .with_min_inner_size([680.0, 480.0])
        .with_inner_size(dialog_size)
        .with_resizable(true);

    // Center directly on the active monitor display
    if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size)
    {
        let center_x = (monitor_size.x - dialog_size.x) / 2.0;
        let center_y = (monitor_size.y - dialog_size.y) / 2.0;

        builder = builder.with_position(egui::pos2(center_x, center_y));
    }

    let viewport_id = egui::ViewportId::from_hash_of("image_adjust_viewport");

    ctx.show_viewport_immediate(viewport_id, builder, |viewport_ctx, _class|
    {
        // Intercept native OS window Close (X) button
        if viewport_ctx.input(|i| i.viewport().close_requested())
        {
            cancel = true;
        }

        // Viewports require a layout panel, acting as the new top-level canvas
        egui::CentralPanel::default().show(viewport_ctx, |ui|
        {
            // ── Scope indicator ─────────────────────────────────────────────
            let scope_label = match app.adjust_state.source_rect
            {
                Some((_, _, w, h)) => format!("Editing selection ({w}\u{d7}{h}px)"),
                None => "Editing full image".to_string(),
            };
            ui.label(
                egui::RichText::new(scope_label)
                    .italics()
                    .color(egui::Color32::from_gray(150)),
            );
            ui.add_space(4.0);

            // ── Thumbnail panes ───────────────────────────────────────────────
            let avail_w = ui.available_width();
            let pane_w  = (avail_w - ui.spacing().item_spacing.x) / 2.0;
            let thumb_h = (pane_w * 0.65).clamp(160.0, 380.0);

            ui.horizontal(|ui|
            {
                show_thumb_pane(
                    ui, "Original",
                    app.adjust_state.original_texture.as_ref(),
                    egui::vec2(pane_w, thumb_h),
                );
                show_thumb_pane(
                    ui, "Preview",
                    app.adjust_state.preview_texture.as_ref(),
                    egui::vec2(pane_w, thumb_h),
                );
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Controls (two columns) ────────────────────────────────────────
            {
                let state = &mut app.adjust_state;
                ui.columns(2, |cols|
                {
                    changed |= show_left_controls(&mut cols[0], state);
                    changed |= show_right_controls(&mut cols[1], state);
                });
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // ── Buttons ───────────────────────────────────────────────────────
            ui.horizontal(|ui|
            {
                if ui.button("  Reset All  ").clicked()
                {
                    reset_all = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui|
                {
                    if ui.add_sized([80.0, 24.0], egui::Button::new("Apply")).clicked()
                    {
                        apply = true;
                    }
                    if ui.add_sized([80.0, 24.0], egui::Button::new("Cancel")).clicked()
                    {
                        cancel = true;
                    }
                });
            });
        });
    });

    // ── Post-render logic (borrows app freely now that the closure is done) ───
    /* Reset takes priority over a plain "changed" preview refresh, since
     * `reset()` already implies the sliders changed and we only want one
     * preview rebuild per frame. */
    if reset_all
    {
        app.adjust_state.reset();
        image_utils::update_adjust_preview(ctx, &mut app.adjust_state);
    }
    else if changed
    {
        image_utils::update_adjust_preview(ctx, &mut app.adjust_state);
    }

    /* Apply bakes the current adjustment settings into the full-resolution
     * image (not just the thumbnail preview) and loads that as the new
     * working image, then closes the dialog. Cancel just closes it,
     * discarding any adjustments.
     *
     * If the dialog was scoped to a selection (`source_rect` is set), only
     * that region of the full image is touched and everything outside it
     * is left byte-for-byte untouched; otherwise the whole image is
     * adjusted as before. Either way the selection box itself is left in
     * place afterward (both on Apply and Cancel) so it can immediately be
     * re-adjusted, copied, etc. */
    if apply
    {
        let adjusted = app.image.as_ref().map(|img|
        {
            match app.adjust_state.source_rect
            {
                Some(rect) => image_utils::apply_adjustments_region(img, &app.adjust_state, rect),
                None       => image_utils::apply_adjustments(img, &app.adjust_state),
            }
        });

        if let Some(adj) = adjusted
        {
            app.load_dynamic_image(ctx, adj);
        }
        app.show_adjust_dialog = false;
    }
    else if cancel
    {
        app.show_adjust_dialog = false;
    }
}

// ── Thumbnail pane ────────────────────────────────────────────────────────────

/* Draws one labeled, bordered thumbnail pane (used for both the "Original"
 * and "Preview" panes).
 *
 * Renders a centered bold header above a fixed-size bordered frame. Inside
 * the frame, if a texture is provided it is scaled down (never up) to fit
 * within the frame while preserving aspect ratio, then centered both
 * horizontally and vertically via manual padding. If no texture is
 * provided, a "No image" placeholder label is shown instead.
 */
fn show_thumb_pane(
    ui:      &mut egui::Ui,
    label:   &str,
    texture: Option<&egui::TextureHandle>,
    size:    egui::Vec2,
)
{
    ui.vertical(|ui|
    {
        ui.set_width(size.x);

        // Centred bold header
        ui.horizontal(|ui|
        {
            ui.set_width(size.x);
            ui.centered_and_justified(|ui|
            {
                ui.label(egui::RichText::new(label).strong().size(13.0));
            });
        });

        ui.add_space(2.0);

        // Bordered frame with a controlled inner margin
        let margin  = 4.0_f32;
        let border  = 1.0_f32;
        let inner_w = (size.x - (margin + border) * 2.0).max(10.0);
        let inner_h = (size.y - (margin + border) * 2.0).max(10.0);

        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(margin as i8))
            .show(ui, |ui|
            {
                ui.set_min_size(egui::vec2(inner_w, inner_h));
                ui.set_max_size(egui::vec2(inner_w, inner_h));

                match texture
                {
                    Some(tex) =>
                    {
                        // Scale to fit while preserving aspect ratio
                        let tex_size = tex.size_vec2();
                        let scale    = (inner_w / tex_size.x).min(inner_h / tex_size.y);
                        let draw     = tex_size * scale;

                        // Centre within the pane
                        let pad_x = ((inner_w - draw.x) / 2.0).max(0.0);
                        let pad_y = ((inner_h - draw.y) / 2.0).max(0.0);
                        ui.add_space(pad_y);
                        ui.horizontal(|ui|
                        {
                            ui.add_space(pad_x);
                            ui.add(
                                egui::Image::new((tex.id(), tex_size))
                                    .fit_to_exact_size(draw),
                            );
                        });
                    }
                    None =>
                    {
                        ui.centered_and_justified(|ui|
                        {
                            ui.label(
                                egui::RichText::new("No image")
                                    .color(egui::Color32::from_gray(100)),
                            );
                        });
                    }
                }
            });
    });
}

// ── Control columns ───────────────────────────────────────────────────────────

/* Left column of adjustment sliders: overall Brightness, followed by a
 * "Color Balance" section (Red / Green / Blue channel offsets).
 * Returns true if any slider/drag value changed this frame, so the caller
 * knows whether to regenerate the preview texture.
 */
fn show_left_controls(ui: &mut egui::Ui, state: &mut AdjustState) -> bool
{
    let mut changed = false;

    ui.add_space(4.0);
    changed |= labeled_slider(ui, "Brightness", &mut state.brightness, -255..=255, 1.0);

    ui.add_space(10.0);
    ui.label(
        egui::RichText::new("Color Balance")
            .italics()
            .color(egui::Color32::from_gray(160)),
    );
    ui.separator();
    ui.add_space(2.0);

    changed |= labeled_slider(ui, "Red",   &mut state.red,   -128..=128, 1.0);
    changed |= labeled_slider(ui, "Green", &mut state.green, -128..=128, 1.0);
    changed |= labeled_slider(ui, "Blue",  &mut state.blue,  -128..=128, 1.0);

    changed
}

/* Right column of adjustment sliders: Contrast, Gamma, and Saturation.
 * Returns true if any slider/drag value changed this frame, so the caller
 * knows whether to regenerate the preview texture.
 */
fn show_right_controls(ui: &mut egui::Ui, state: &mut AdjustState) -> bool
{
    let mut changed = false;

    ui.add_space(4.0);
    changed |= labeled_slider(ui, "Contrast",  &mut state.contrast,   -100.0..=100.0, 0.5);
    ui.add_space(4.0);
    changed |= labeled_slider(ui, "Gamma",     &mut state.gamma,        0.1..=5.0,    0.01);
    ui.add_space(4.0);
    changed |= labeled_slider(ui, "Saturation", &mut state.saturation,  0.0..=2.0,   0.01);

    changed
}

/*  ── Generic labeled slider row ────────────────────────────────────────────────
 *
 * Layout:  [label 80px] [slider — expands with window] [DragValue 58px]
 * The slider track grows/shrinks as the modal is resized; label and number
 * box stay fixed-width.
 */

/* Renders one row combining a fixed-width label, a slider that stretches
 * to fill remaining horizontal space, and a fixed-width numeric DragValue
 * box in sync with the slider (edits to either update the same `value`).
 *
 * Generic over any numeric type egui's widgets support (`Num`), so it can
 * back both integer sliders (Brightness, RGB offsets) and float sliders
 * (Contrast, Gamma, Saturation) with one implementation. Returns true if
 * either the slider or the drag value changed this frame.
 */
fn labeled_slider<Num: egui::emath::Numeric>(
    ui:    &mut egui::Ui,
    label: &str,
    value: &mut Num,
    range: std::ops::RangeInclusive<Num>,
    speed: f64,
) -> bool
{
    let mut changed = false;
    let label_w = 80.0_f32;
    let dv_w    = 58.0_f32;
    let spacing = ui.spacing().item_spacing.x;
    let row_h   = ui.spacing().interact_size.y;

    ui.horizontal(|ui|
    {
        ui.add_sized([label_w, row_h], egui::Label::new(label));

        let slider_w = (ui.available_width() - dv_w - spacing).max(10.0);

        let s = ui.add_sized(
            [slider_w, row_h],
            egui::Slider::new(value, range.clone()).show_value(false),
        );

        let d = ui.add_sized(
            [dv_w, row_h],
            egui::DragValue::new(value).speed(speed),
        );

        changed = s.changed() || d.changed();
    });

    changed
}