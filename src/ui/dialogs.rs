use eframe::egui;
use image::DynamicImage;
use crate::app::ImageViewerApp;
use crate::image_utils;

/* Draws the "Resize Image" modal window and handles its interactions.
 *
 * Lets the user set a target width/height directly (optionally locked to
 * the original aspect ratio) or via a scale percentage, which recomputes
 * width/height from the currently loaded image's dimensions. On "OK", the
 * image is resized with Lanczos3 filtering to the chosen exact dimensions
 * and loaded as the new working image; "Cancel" just closes the dialog
 * without applying anything.
 */
pub fn show_resize_dialog(app: &mut ImageViewerApp, ctx: &egui::Context)
{
    egui::Window::new("Resize Image")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui|
        {
            ui.horizontal(|ui|
            {
                ui.label("Width:");
                if ui.add(egui::DragValue::new(&mut app.resize_state.width)).changed()
                {
                    // Keep height in sync when the aspect ratio is locked.
                    if app.resize_state.lock_ratio && app.resize_state.aspect_ratio > 0.0
                    {
                        app.resize_state.height = (app.resize_state.width as f32 / app.resize_state.aspect_ratio) as u32;
                    }
                }
                ui.label("px");
            });

            ui.horizontal(|ui|
            {
                ui.label("Height:");
                if ui.add(egui::DragValue::new(&mut app.resize_state.height)).changed()
                {
                    // Keep width in sync when the aspect ratio is locked.
                    if app.resize_state.lock_ratio && app.resize_state.aspect_ratio > 0.0
                    {
                        app.resize_state.width = (app.resize_state.height as f32 * app.resize_state.aspect_ratio) as u32;
                    }
                }
                ui.label("px");
            });

            ui.checkbox(&mut app.resize_state.lock_ratio, "Lock Aspect Ratio");

            ui.separator();

            ui.horizontal(|ui|
            {
                ui.label("Scale (%):");
                if ui.add(egui::DragValue::new(&mut app.resize_state.percentage).speed(1.0)).changed()
                {
                    /* Recompute absolute width/height from the original
                     * image dimensions each time the percentage changes,
                     * rather than compounding off the current width/height. */

                     if let Some(ref img) = app.image
                    {
                        let factor = app.resize_state.percentage / 100.0;
                        app.resize_state.width = (img.width() as f32 * factor) as u32;
                        app.resize_state.height = (img.height() as f32 * factor) as u32;
                    }
                }
            });

            ui.separator();

            ui.horizontal(|ui|
            {
                if ui.button("OK").clicked()
                {
                    if let Some(ref img) = app.image
                    {
                        let resized = img.resize_exact(
                            app.resize_state.width,
                            app.resize_state.height,
                            image::imageops::FilterType::Lanczos3,
                        );
                        app.load_dynamic_image(ctx, resized);
                    }
                    app.show_resize_dialog = false;
                }

                if ui.button("Cancel").clicked()
                {
                    app.show_resize_dialog = false;
                }
            });
        });
}

/* Draws the "Arbitrary Rotation" modal window and handles its interactions.
 *
 * Shows a live preview texture (generated from the small cached thumbnail,
 * scaled down further to fit within a 300px display box) alongside a
 * slider for the rotation angle in degrees. Moving the slider regenerates
 * the thumbnail-based preview via `image_utils::update_rotate_preview` so
 * the change is visible immediately without touching the full-resolution
 * image.
 *
 * On "Apply", the rotation is redone at full resolution against the real
 * loaded image (not the thumbnail) and the result becomes the new working
 * image; "Cancel" closes the dialog without applying anything.
 */
pub fn show_rotate_dialog(app: &mut ImageViewerApp, ctx: &egui::Context)
{
    egui::Window::new("Arbitrary Rotation")
        .collapsible(false)
        .show(ctx, |ui|
        {
            if let Some(ref preview) = app.rotate_state.preview_texture
            {
                let tex_size = preview.size_vec2(); // actual texture dimensions
                const MAX_DISPLAY: f32 = 300.0;

                // Scale down (never up) to fit within the display box while
                // preserving aspect ratio.
                let scale = (MAX_DISPLAY / tex_size.x).min(MAX_DISPLAY / tex_size.y);
                let display_size = tex_size * scale;

                ui.image((preview.id(), display_size));
            }

            if ui.add(egui::Slider::new(&mut app.rotate_state.angle_degrees, -180.0..=180.0).text("Angle")).changed()
            {
                image_utils::update_rotate_preview(ctx, &mut app.rotate_state);
            }

            ui.horizontal(|ui|
            {
                if ui.button("Apply").clicked()
                {
                    // Re-run the same rotation used for the preview, but
                    // against the full-resolution source image instead of
                    // the small thumbnail.
                    if let Some(ref img) = app.image
                    {
                        let radians = app.rotate_state.angle_degrees.to_radians();
                        let rotated = imageproc::geometric_transformations::rotate_about_center(
                            &img.to_rgba8(),
                            radians,
                            imageproc::geometric_transformations::Interpolation::Bilinear,
                            image::Rgba([0, 0, 0, 0]),
                        );
                        app.load_dynamic_image(ctx, DynamicImage::ImageRgba8(rotated));
                    }
                    app.show_rotate_dialog = false;
                }

                if ui.button("Cancel").clicked()
                {
                    app.show_rotate_dialog = false;
                }
            });
        });
}