use eframe::egui;
use crate::app::ImageViewerApp;
use crate::state::ViewMode;
use crate::image_utils;

/* Draws the top menu bar (File / Edit / Image / View) and wires up each
 * menu item to the corresponding app action.
 *
 * This function only handles menu-triggered actions; the same operations
 * are also reachable via keyboard shortcuts elsewhere (see the shortcut
 * hints baked into a few of the button labels below, e.g. "shift-c",
 * "Ctr-R"). Most Image-menu items are disabled (`add_enabled(false, ...)`)
 * whenever no image is currently loaded.
 */
pub fn show_menu(app: &mut ImageViewerApp, ui: &mut egui::Ui)
{
    egui::Panel::top("top_panel").show(ui, |ui|
    {
        egui::MenuBar::new().ui(ui, |ui|
        {
            ui.menu_button("File", |ui|
            {
                if ui.button("Open Image").clicked()
                {
                    app.open_image(ui.ctx());
                }

                ui.separator();

                if ui.button("Close").clicked()
                {
                    // Ask the OS/windowing system to close the app window.
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Edit", |ui|
            {
                if ui.button("Copy (shift-c)").clicked()
                {
                    //image_utils::paste_from_clipboard(app, ui.ctx());

                    /* Copy only makes sense if there's a loaded image, an
                     * active selection (start + end corners), and a known
                     * on-screen image rect to map those corners back into
                     * image pixel space — hence the combined `if let`.
                     */
                    if let (Some(img), Some(start), Some(end), Some(rect)) = (&app.image, app.selection_start, app.selection_end, app.image_rect)
                    {
                        image_utils::copy_selection_to_clipboard(img, start, end, rect, app.flipped_h, app.flipped_v);
                    }
                }

                if ui.button("Paste (shift-v)").clicked()
                {
                    image_utils::paste_from_clipboard(app, ui.ctx());
                }
            });

            ui.menu_button("Image", |ui|
            {
                if ui.add_enabled(app.image.is_some(), egui::Button::new("Adjust (Shift+G)")).clicked()
                {
                    app.open_adjust_dialog(ui.ctx());
                }

                ui.separator();

                if ui.add_enabled(app.image.is_some(), egui::Button::new("Resize (Ctr-R)")).clicked()
                {
                    app.show_resize_dialog = true;
                }
                if ui.add_enabled(app.image.is_some(), egui::Button::new("Rotate...")).clicked()
                {
                    app.open_rotate_dialog(ui.ctx());
                }
                if ui.add_enabled(app.image.is_some(), egui::Button::new("Flip horizontal (H)")).clicked()
                {
                    // Flips are drawn on the GPU (see ImageViewerApp::flipped_h),
                    // not baked into the pixel buffer, so just toggle the flag.
                    app.flipped_h = !app.flipped_h;
                    ui.ctx().request_repaint();
                }
                if ui.add_enabled(app.image.is_some(), egui::Button::new("Flip vertical (V)")).clicked()
                {
                    app.flipped_v = !app.flipped_v;
                    ui.ctx().request_repaint();
                }
            });

            ui.menu_button("View", |ui|
            {
                // Label reflects the mode we'd switch TO, not the current mode.
                let toggle_label = if app.view_mode == ViewMode::FitToWindow { "Actual size (100%)" } else { "Fit to window" };

                if ui.add_enabled(app.image.is_some(), egui::Button::new(toggle_label)).clicked()
                {
                    app.toggle_view_mode();
                }

                if ui.button("Fullscreen").clicked()
                {
                    app.toggle_fullscreen(ui.ctx());
                }                
            });
        });
    });
}