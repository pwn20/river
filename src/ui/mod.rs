pub mod menu;
pub mod main_view;
pub mod dialogs;
pub mod adjust_dialog;

use eframe::egui;
use crate::app::ImageViewerApp;
use crate::image_utils;

pub fn update(app: &mut ImageViewerApp, ui: &mut egui::Ui, _frame: &mut eframe::Frame)
{
    if app.show_chrome
    {
        menu::show_menu(app, ui);
    }

    main_view::show_main_view(app, ui);
    handle_input(app, ui.ctx());
    
    if app.show_resize_dialog
    {
        dialogs::show_resize_dialog(app, ui.ctx());
    }

    if app.show_rotate_dialog
    {
        dialogs::show_rotate_dialog(app, ui.ctx());
    }

    if app.show_adjust_dialog
    {
        adjust_dialog::show_adjust_dialog(app, ui.ctx());
    }
}

// Handle keyboard shortcuts and other input events. Almost all shortcuts are here but a few still live
// in the main_view code (e.g. mouse wheel zoom, mouse drag for panning, etc.)
fn handle_input(app: &mut ImageViewerApp, ctx: &egui::Context)
{
    // Take pixels from the current selection and copy them to the clipboard. This is a "shift+C" shortcut.
    if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::C))
    {
        if let (Some(img), Some(start), Some(end), Some(rect)) = (&app.image, app.selection_start, app.selection_end, app.image_rect)
        {
            image_utils::copy_selection_to_clipboard(img, start, end, rect, app.flipped_h, app.flipped_v);
        }
        return;
    }

    // Paste pixels from the clipboard as a new image. This is a "shift+V" shortcut.
    if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::V))
    {
        image_utils::paste_from_clipboard(app, ctx);
        return;
    }

    // Shift-f is a shortcut for disabling all window decorations.
    if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::F))
    {
        app.show_chrome = !app.show_chrome;
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(app.show_chrome));
        return;
    }

    // F12 or "F" will disable all window decorations and maximize the UI to simulate fullscreen.
    if ctx.input(|i| i.key_pressed(egui::Key::F12) || i.key_pressed(egui::Key::F))
    {
        app.show_chrome = !app.show_chrome;
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(app.show_chrome));
        app.toggle_fullscreen(ctx);
        return;
    }

    // "Z" will buonce between original image size and fit-to-window.
    if ctx.input(|i| i.key_pressed(egui::Key::Z))
    {
        app.toggle_view_mode();
        return;
    }

    // "H" will flip the image horizontally on the GPU. Fucks up when trying to copy a selection.
    if ctx.input(|i| i.key_pressed(egui::Key::H))
    {
        app.flipped_h = !app.flipped_h;
        ctx.request_repaint();
        return;
    }

    // Vertically flip the image (does not *edit* the image this is a GPU trick)
    if ctx.input(|i| i.key_pressed(egui::Key::V))
    {
        app.flipped_v = !app.flipped_v;
        ctx.request_repaint();
        return;
    }

    // Open the image adjustment window for alterting R/G/B levels, brightness, contrast, gamma, and saturation.
    // This *does* edit the image.
    if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::G))
    {
        app.open_adjust_dialog(ctx);
        return;
    }

    // Open the resize image dialog.
    if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R))
    {
        app.show_resize_dialog = true;
        return;
    }
}