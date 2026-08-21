use eframe::egui;
use image::{DynamicImage, GenericImageView};
use crate::state::{AdjustState, ResizeState, RotateState, ViewMode, SelectionDragMode};
use crate::image_utils;
use crate::ui;
use std::path::PathBuf;

/* Top-level application state for the image viewer.
 *
 * This struct holds everything needed across frames: the currently loaded
 * image (both the raw `DynamicImage` and its GPU texture), viewport/zoom/pan
 * state, which dialogs are open and their associated state, selection-box
 * state (used for crop/resize interactions), and misc. flags used for
 * windowing behavior (fullscreen, custom chrome, first-frame centering). */
 
 pub struct ImageViewerApp
{
    // The currently loaded image in CPU memory (decoded pixels), if any.
    pub image: Option<DynamicImage>,
    // GPU texture handle for `image`, used by egui to actually paint it. Kept in sync with `image` whenever a new image is loaded.
    pub texture: Option<egui::TextureHandle>,

    // Viewport control
    // Whether the image should be auto-fit to the window or shown at its native/zoomed size (`ActualSize`).
    pub view_mode: ViewMode,
    // Current zoom multiplier applied when in `ActualSize` mode.
    pub zoom_factor: f32,
    // Current pan offset (in screen pixels) applied to the image when panning/dragging around a zoomed-in image.
    pub pan_offset: egui::Vec2,

    // Window states
    // Whether the OS window decorations/toolbar ("chrome") are shown. When `false`, the app draws its own resize border (see
    // `handle_window_resize_edges`).
    pub show_chrome: bool,
    // Whether the app window is currently in fullscreen mode.
    pub is_fullscreen: bool,

    // Dialog flags
    // Whether the "Resize image" dialog is currently open.
    pub show_resize_dialog: bool,
    // State backing the resize dialog (target width/height, aspect lock, etc).
    pub resize_state: ResizeState,
    // Tracks the current drag interaction (if any) on the selection box (e.g. dragging a corner/edge handle vs. idle).
    pub selection_drag: SelectionDragMode,
    // True only on the very first UI update; used to trigger one-time startup behavior such as centering the window on screen.
    pub first_frame: bool,

    // Whether the "Rotate image" dialog is currently open.
    pub show_rotate_dialog: bool,
    // State backing the rotate dialog (angle, cached thumbnail, live preview texture).
    pub rotate_state: RotateState,

    // Whether the "Adjust image" dialog (brightness/contrast/etc.) is currently open.
    pub show_adjust_dialog: bool,
    // State backing the adjust dialog (slider values, thumbnails, preview texture).
    pub adjust_state: AdjustState,

    // Selection Box
    // Start corner of an active selection box, in screen coordinates, if the user is currently drawing/holding one (e.g. for crop selection).
    pub selection_start: Option<egui::Pos2>,
    // End corner of an active selection box, in screen coordinates.
    pub selection_end: Option<egui::Pos2>,
    // The screen-space rect the image is currently being painted into; used to convert between screen coordinates and image-local coordinates for
    // selection/crop math.
    pub image_rect: Option<egui::Rect>,

    // zooming shit to be implemented later via GUI setting
    // Default per-scroll-tick zoom increment (not yet exposed in settings UI).
    pub default_zoom_step: f32,
    // Default zoom increment used when a modifier key is held (e.g. faster zoom).
    pub default_zoom_step_modified: f32,

    // New boolean flags for image flipping via GPU instead of CPU
    // Whether the image is currently flipped horizontally. This is applied at draw time (GPU/UV-flip) rather than mutating the pixel buffer, for performance.
    pub flipped_h: bool,
    // Whether the image is currently flipped vertically (drawn, not baked into pixels).
    pub flipped_v: bool,

    // If the app was launched indirectly by the OS (such as via file manager) then this will be the path to the file to open.
    // Consumed (via `Option::take`) the first time `open_image` runs so the file is only auto-opened once.
    pub initial_file: Option<PathBuf>,
}
//farts
impl ImageViewerApp
{
/*  Constructs the initial application state.
    
    Disables egui's built-in ctrl/cmd + scroll zoom-with-keyboard behavior (the app handles zoom itself), and initializes all fields to sensible
    defaults. `initialfile` is an optional path passed in from `main` (e.g. when the OS launches the app with a file to open, such as via
    "Open with..."). */

     pub fn new(cc: &eframe::CreationContext<'_>, initialfile: Option<PathBuf>) -> Self
    {
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);

        Self
        {
            image: None,
            texture: None,
            view_mode: ViewMode::FitToWindow,
            zoom_factor: 1.0,
            pan_offset: egui::Vec2::ZERO,
            show_chrome: true,
            is_fullscreen: false,

            show_resize_dialog: false,
            resize_state: ResizeState
            {
                width: 0,
                height: 0,
                aspect_ratio: 1.0,
                lock_ratio: true,
                percentage: 100.0,
            },

            first_frame: true,

            show_rotate_dialog: false,
            rotate_state: RotateState
            {
                angle_degrees: 0.0,
                thumbnail_base: None,
                preview_texture: None,
            },
            //active_resize_edge: None,
            selection_drag: SelectionDragMode::Idle,

            show_adjust_dialog: false,
            adjust_state: AdjustState::default(),

            selection_start: None,
            selection_end: None,
            image_rect: None,

            default_zoom_step: 0.01,
            default_zoom_step_modified: 0.04,

            flipped_h: false,
            flipped_v: false,

            initial_file: initialfile,
        }
    }

/*  Opens an image, either from `initial_file` (if the app was launched with a file to open) or, otherwise, by showing a native "pick file"
    dialog for the user to choose one. On success, resets flip flags, loads the image into GPU/CPU state via `load_dynamic_image`, switches to
    `FitToWindow` view mode, and resets panning. Silently does nothing if decoding fails or the user cancels the file dialog. */

    pub fn open_image(&mut self, ctx: &egui::Context)
    {
        // If we were launched with a specific file (e.g. double-clicked in a
        // file manager), consume and open that instead of prompting the user.
        if let Some(initial_file) = self.initial_file.take()
        {
            if let Ok(img) = image::open(initial_file)
            {
                self.flipped_h = false;
                self.flipped_v = false;
                self.load_dynamic_image(ctx, img);
                self.view_mode = ViewMode::FitToWindow;
                self.pan_offset = egui::Vec2::ZERO;
            }

            return;
        }

        // Otherwise, ask the user to pick a file via the native file dialog.
        if let Some(path) = rfd::FileDialog::new().pick_file()
        {
            if let Ok(img) = image::open(path)
            {
                self.flipped_h = false;
                self.flipped_v = false;
                self.load_dynamic_image(ctx, img);
                self.view_mode = ViewMode::FitToWindow;
                self.pan_offset = egui::Vec2::ZERO;
            }
        }
    }

/*   Loads a decoded `DynamicImage` into app state and uploads it to the GPU as an egui texture so it can be displayed. Also seeds
     the resize dialog's default width/height/aspect ratio from the newly loaded image's dimensions, so opening the resize dialog
     starts from the current image size. */
    pub fn load_dynamic_image(&mut self, ctx: &egui::Context, img: DynamicImage)
    {
        let dimensions = img.dimensions();
        self.resize_state.width = dimensions.0;
        self.resize_state.height = dimensions.1;
        self.resize_state.aspect_ratio = dimensions.0 as f32 / dimensions.1 as f32;

        // Convert to RGBA8 and hand the raw pixel buffer to egui as a ColorImage.
        let size = [img.width() as usize, img.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            img.to_rgba8().as_flat_samples().as_slice(),
        );

        self.texture = Some(ctx.load_texture("main_image", color_image, Default::default()));
        self.image = Some(img);
    }

/*  Prepares and opens the rotate dialog.
    Generates a small (max 400px on the longest side) thumbnail of the current image so rotation previews stay fast, resets the rotation
    angle to 0, and generates the initial preview texture via `image_utils::update_rotate_preview`. No-op if no image is loaded. */

    pub fn open_rotate_dialog(&mut self, ctx: &egui::Context)
    {
        if let Some(ref img) = self.image
        {
            const MAX_DIM: u32 = 400;
            let (orig_w, orig_h) = (img.width(), img.height());

            // Scale down proportionally so neither dimension exceeds MAX_DIM.
            let scale = (MAX_DIM as f64 / orig_w as f64).min(MAX_DIM as f64 / orig_h as f64);
            let new_w = (orig_w as f64 * scale).round() as u32;
            let new_h = (orig_h as f64 * scale).round() as u32;

            let thumb = img.thumbnail(new_w, new_h);

            self.rotate_state.thumbnail_base = Some(thumb.clone());
            self.rotate_state.angle_degrees = 0.0;
            image_utils::update_rotate_preview(ctx, &mut self.rotate_state);
            self.show_rotate_dialog = true;
        }
    }

/*  Prepares and opens the brightness/contrast/etc. "adjust" dialog.
    Builds a 512x512-max thumbnail of the current image to use as both the "original" reference preview (uploaded once as its own texture)
    and the base image for the live-adjusted preview. Resets adjustment sliders to defaults via `adjust_state.reset()` and generates an
    initial preview via `image_utils::update_adjust_preview`. No-op if no image is loaded. */

    pub fn open_adjust_dialog(&mut self, ctx: &egui::Context)
    {
        if let Some(ref img) = self.image
        {
            // If there's a large-enough active selection box, scope the
            // whole dialog to just that region of the full-res image.
            let source_rect = self.active_selection_image_rect(img);

            // Base the thumbnail/preview pipeline on the selected
            // sub-region when one exists, otherwise the whole image -
            // same as before this function knew about selections.
            let source_img = match source_rect
            {
                Some((x, y, w, h)) => DynamicImage::ImageRgba8(
                    image::imageops::crop_imm(img, x, y, w, h).to_image()
                ),
                None => img.clone(),
            };

            // Capture a thumbnail for the original pane and the processing pipeline
            let thumb = source_img.thumbnail(512, 512);
            let rgba  = thumb.to_rgba8();
            let size  = [rgba.width() as usize, rgba.height() as usize];
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                size,
                rgba.as_flat_samples().as_slice(),
            );
            // Static "before" texture shown alongside the live-updating preview.
            self.adjust_state.original_texture = Some(ctx.load_texture(
                "adjust_original",
                color_image,
                Default::default(),
            ));
            self.adjust_state.thumbnail_base = Some(thumb);
            self.adjust_state.source_rect = source_rect;
            self.adjust_state.reset();
            image_utils::update_adjust_preview(ctx, &mut self.adjust_state);
            self.show_adjust_dialog = true;
        }
    }

    /* If a large-enough selection box is currently active (same 5x5px
     * screen-space threshold `main_view.rs` uses to decide whether a
     * selection "counts"), maps it into full-resolution image pixel
     * coordinates as (x, y, width, height) via
     * `image_utils::selection_to_image_rect`. Returns `None` if there's no
     * selection, it's too small, or there's no known on-screen image rect
     * to map it from (e.g. before the first frame has drawn the image). */
    fn active_selection_image_rect(&self, img: &DynamicImage) -> Option<(u32, u32, u32, u32)>
    {
        let start = self.selection_start?;
        let end   = self.selection_end?;
        let rect  = self.image_rect?;

        let screen_sel = egui::Rect::from_two_pos(start, end);
        if screen_sel.width() <= 5.0 || screen_sel.height() <= 5.0
        {
            return None;
        }

        let (img_w, img_h) = img.dimensions();
        Some(image_utils::selection_to_image_rect(
            start, end, rect, (img_w, img_h), self.flipped_h, self.flipped_v,
        ))
    }

/*  Toggles between `FitToWindow` and `ActualSize` view modes. Switching to `ActualSize` resets zoom to 1.0 (100%)
    and clears any pan offset; switching back to `FitToWindow` also clears pan offset (since fit-to-window
    recomputes scale/positioning automatically). */

    pub fn toggle_view_mode(&mut self)
    {
        if self.view_mode == ViewMode::FitToWindow
        {
            self.view_mode = ViewMode::ActualSize;
            self.zoom_factor = 1.0;
            self.pan_offset = egui::Vec2::ZERO;
        }
        else
        {
            self.view_mode = ViewMode::FitToWindow;
            self.pan_offset = egui::Vec2::ZERO;
        }
    }

    // Toggles OS-level fullscreen mode for the window, flipping `is_fullscreen` and sending the corresponding viewport command to eframe/winit.
    pub fn toggle_fullscreen(&mut self, ctx: &egui::Context)
    {
        self.is_fullscreen = !self.is_fullscreen;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
    }

/*  Implements manual window-edge resizing for borderless/chrome-less windows (used when `show_chrome` is false
    and the OS titlebar/border is hidden, so there's no native resize handle).

    Each frame, checks how close the pointer is to each of the four window edges (within `stroke_width` points).
    If it's near an edge or corner, sets an appropriate resize cursor icon and, if the primary mouse button is
    pressed there, asks the OS/windowing system to begin a native interactive resize in that direction via
    `ViewportCommand::BeginResize`. */

    fn handle_window_resize_edges(&self, ctx: &egui::Context)
    {
        use egui::{ResizeDirection, ViewportCommand};

        let stroke_width = 4.0; // Width of the invisible resize border in points
        let screen_rect = ctx.viewport_rect();

        // Check mouse interactions around the 4 edges and corners
        // Egui provides helper logic or manual rect checks for border hit-testing:

        // Or simple pointer position checking:
        if let Some(pos) = ctx.pointer_latest_pos()
        {
            if screen_rect.contains(pos)
            {
                // Distance from the pointer to each of the four edges.
                let dist_left = pos.x - screen_rect.min.x;
                let dist_right = screen_rect.max.x - pos.x;
                let dist_top = pos.y - screen_rect.min.y;
                let dist_bottom = screen_rect.max.y - pos.y;

                let on_left = dist_left <= stroke_width;
                let on_right = dist_right <= stroke_width;
                let on_top = dist_top <= stroke_width;
                let on_bottom = dist_bottom <= stroke_width;

                // Combine edge flags into a single resize direction, favoring
                // corners (two edges at once) over single-edge directions.
                let direction = match (on_left, on_right, on_top, on_bottom)
                {
                    (true, _, true, _) => Some(ResizeDirection::NorthWest),
                    (_, true, true, _) => Some(ResizeDirection::NorthEast),
                    (true, _, _, true) => Some(ResizeDirection::SouthWest),
                    (_, true, _, true) => Some(ResizeDirection::SouthEast),
                    (true, _, _, _) => Some(ResizeDirection::West),
                    (_, true, _, _) => Some(ResizeDirection::East),
                    (_, _, true, _) => Some(ResizeDirection::North),
                    (_, _, _, true) => Some(ResizeDirection::South),
                    _ => None,
                };

                if let Some(dir) = direction
                {
                    // Manually map the direction to the correct cursor icon
                    let cursor_icon = match dir
                    {
                        ResizeDirection::North => egui::CursorIcon::ResizeNorth,
                        ResizeDirection::South => egui::CursorIcon::ResizeSouth,
                        ResizeDirection::East => egui::CursorIcon::ResizeEast,
                        ResizeDirection::West => egui::CursorIcon::ResizeWest,
                        ResizeDirection::NorthWest => egui::CursorIcon::ResizeNorthWest,
                        ResizeDirection::NorthEast => egui::CursorIcon::ResizeNorthEast,
                        ResizeDirection::SouthWest => egui::CursorIcon::ResizeSouthWest,
                        ResizeDirection::SouthEast => egui::CursorIcon::ResizeSouthEast,
                    };

                    // Set appropriate resize cursor
                    ctx.set_cursor_icon(cursor_icon);

                    // If user clicks and drags the edge, delegate native resize to OS
                    if ctx.input(|i| i.pointer.primary_pressed())
                    {
                        ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
                    }
                }
            }
        }
    }
}

impl eframe::App for ImageViewerApp
{
/*  Main per-frame update/draw entry point, called by eframe every frame.

    Order of operations:
    1. If custom (chrome-less) window mode is active, run manual edge resize hit-testing/cursor logic.
    2. On Windows builds with the `win7` feature, force dark visuals (workaround for lack of native dark-mode detection on Win7).
    3. Delegate all actual widget layout/drawing to `ui::update`.
    4. If an `initial_file` is still pending, open it now (handles the "launched via file manager" case).
    5. On the very first frame only, center the window on the primary monitor based on its reported size. */

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)
    {
        // Some junk to do with turning decorations off but enabling adjustable window handles.
        if !self.show_chrome
        {
            self.handle_window_resize_edges(ui.ctx());
        }

        // Sets dark mode, needs to be wrapped in win7-only thingy
        #[cfg(all(target_os = "windows", feature = "win7"))]
        {
            ui.ctx().set_visuals(egui::Visuals::dark());
        }

        ui::update(self, ui, frame);

        // calls open_image() if the user tried to open an image via a file manager
        if self.initial_file.is_some()
        {
            self.open_image(ui.ctx());
        }

        // Center the main window on first update then never again.
        if self.first_frame
        {
            self.first_frame = false;

            if let Some(monitor_size) = ui.ctx().input(|i| i.viewport().monitor_size)
            {
                let main_window_size = egui::vec2(800.0, 400.0); // Your default main window size

                let center_x = (monitor_size.x - main_window_size.x) / 2.0;
                let center_y = (monitor_size.y - main_window_size.y) / 2.0;

                ui.ctx().send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                    egui::pos2(center_x, center_y)
                ));
            }
        }
    }
}