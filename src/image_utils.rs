use std::borrow::Cow;
use eframe::egui;
use image::{DynamicImage, GenericImageView};
use crate::state::{AdjustState, RotateState};
use crate::ImageViewerApp;

/*
 * Regenerates the rotate dialog's live preview texture.
 *
 * Takes the cached small thumbnail (`rotate_state.thumbnail_base`), rotates
 * it about its center by the currently configured angle using bilinear
 * interpolation, and uploads the result as a new GPU texture so the dialog
 * can display it. Transparent (0,0,0,0) is used to fill the corners exposed
 * by rotation. No-op if no thumbnail has been captured yet.
 */
pub fn update_rotate_preview(ctx: &egui::Context, rotate_state: &mut RotateState)
{
    if let Some(ref thumb) = rotate_state.thumbnail_base
    {
        let radians = rotate_state.angle_degrees.to_radians();
        let rotated_thumb = imageproc::geometric_transformations::rotate_about_center(
            &thumb.to_rgba8(),
            radians,
            imageproc::geometric_transformations::Interpolation::Bilinear,
            image::Rgba([0, 0, 0, 0]),
        );

        let size = [rotated_thumb.width() as usize, rotated_thumb.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            rotated_thumb.as_flat_samples().as_slice(),
        );

        rotate_state.preview_texture = Some(ctx.load_texture(
            "rotate_preview",
            color_image,
            Default::default(),
        ));
    }
}

/*
 * Converts a screen-space point into a pixel coordinate within the source
 * image.
 *
 * `screen_pos` is normalized against `image_rect` (the on-screen rect the
 * image is currently drawn into) to get a 0..1 fraction, which is then
 * optionally mirrored if the image is being displayed flipped, and finally
 * scaled up to the image's actual pixel dimensions. Used to translate mouse
 * positions (e.g. selection box corners) into real image pixel coordinates.
 */
pub fn map_screen_to_image_pixels(
    screen_pos: egui::Pos2,
    image_rect: egui::Rect,
    img_dim: (u32, u32),
    flipped_h: bool,
    flipped_v: bool,
) -> (u32, u32)
{
    let mut nx = ((screen_pos.x - image_rect.min.x) / image_rect.width()).clamp(0.0, 1.0);
    let mut ny = ((screen_pos.y - image_rect.min.y) / image_rect.height()).clamp(0.0, 1.0);

    if flipped_h {
        nx = 1.0 - nx;
    }
    if flipped_v {
        ny = 1.0 - ny;
    }

    (
        (nx * img_dim.0 as f32) as u32,
        (ny * img_dim.1 as f32) as u32,
    )
}

/*
 * Maps a selection box's two (unsorted) drag corners into a normalized,
 * clamped full-resolution image pixel rectangle: `(x, y, width, height)`.
 *
 * Each corner is independently mapped to image pixel space (respecting any
 * active flip) before being sorted into a proper top-left/bottom-right
 * rectangle - this must happen in that order (map, then sort) rather than
 * sorting in screen space first, since flipping can swap which corner ends
 * up smaller. The result is clamped to the image bounds, and both
 * width/height are floored at 1px to avoid degenerate empty rects.
 *
 * Shared by the clipboard-copy path and the selection-scoped "Adjust
 * image" path so both agree on exactly which pixels a given on-screen
 * selection corresponds to.
 */
pub fn selection_to_image_rect(
    start: egui::Pos2,
    end: egui::Pos2,
    image_rect: egui::Rect,
    img_dim: (u32, u32),
    flipped_h: bool,
    flipped_v: bool,
) -> (u32, u32, u32, u32)
{
    let (img_w, img_h) = img_dim;

    // Map the raw drag corners directly — don't pre-sort in screen space.
    let (px_a_x, px_a_y) = map_screen_to_image_pixels(start, image_rect, img_dim, flipped_h, flipped_v);
    let (px_b_x, px_b_y) = map_screen_to_image_pixels(end, image_rect, img_dim, flipped_h, flipped_v);

    // Now sort in image space, after the flip has been applied.
    let mut px_x1 = px_a_x.min(px_b_x);
    let mut px_x2 = px_a_x.max(px_b_x);
    let mut px_y1 = px_a_y.min(px_b_y);
    let mut px_y2 = px_a_y.max(px_b_y);

    px_x1 = px_x1.min(img_w);
    px_y1 = px_y1.min(img_h);
    px_x2 = px_x2.min(img_w);
    px_y2 = px_y2.min(img_h);

    let width = px_x2.saturating_sub(px_x1).max(1);
    let height = px_y2.saturating_sub(px_y1).max(1);

    (px_x1, px_y1, width, height)
}

/*
 * Crops the current image to the user's selection box and pushes the
 * result onto the system clipboard as raw image data.
 *
 * The selection is mapped to image pixel space via `selection_to_image_rect`
 * (see that function for the corner/flip handling), then cropped and
 * copied out via `arboard`. Errors from clipboard access are logged to
 * stderr rather than surfaced to the UI.
 */
pub fn copy_selection_to_clipboard(
    img: &DynamicImage,
    start: egui::Pos2,
    end: egui::Pos2,
    image_rect: egui::Rect,
    flipped_h: bool,
    flipped_v: bool,
) {
    let (img_w, img_h) = img.dimensions();
    let (px_x1, px_y1, width, height) =
        selection_to_image_rect(start, end, image_rect, (img_w, img_h), flipped_h, flipped_v);

    let cropped = image::imageops::crop_imm(img, px_x1, px_y1, width, height).to_image();

    let (actual_w, actual_h) = cropped.dimensions();

    match arboard::Clipboard::new() 
    {
        Ok(mut clipboard) => 
        {
            let img_data = arboard::ImageData 
            {
                width: actual_w as usize,
                height: actual_h as usize,
                bytes: Cow::Owned(cropped.into_raw()),
            };
            if let Err(e) = clipboard.set_image(img_data) {
                eprintln!("clipboard set_image failed: {e:?}");
            }
        }
        Err(e) => eprintln!("clipboard init failed: {e:?}"),
    }
}

/*
 * Loads whatever is currently on the system clipboard into the viewer, if
 * possible.
 *
 * Tries two strategies in order:
 *   1. Raw image pixels (e.g. a screenshot or a browser "Copy Image"
 *      result) — read directly via `arboard` and loaded straight in.
 *   2. A file path as plain text (e.g. a file copied in Windows Explorer)
 *      — the clipboard text is cleaned of surrounding quotes/whitespace,
 *      checked to see if it points at a real file, and if so, decoded and
 *      loaded.
 *
 * If neither strategy yields a usable image, the function silently does
 * nothing. Errors opening the clipboard itself are logged to stderr.
 */
pub fn paste_from_clipboard(app: &mut ImageViewerApp, ctx: &egui::Context)
{
    let mut clipboard = match arboard::Clipboard::new()
    {
        Ok(cb) => cb,
        Err(err) =>
        {
            eprintln!("Failed to open clipboard: {err}");
            return;
        }
    };

    // 1. Try reading raw image pixels first (e.g. screenshots, browser "Copy Image")
    if let Ok(img_data) = clipboard.get_image()
    {
        if let Some(rgba) = image::RgbaImage::from_raw(
            img_data.width as u32,
            img_data.height as u32,
            img_data.bytes.into_owned(),
        )
        {
            let dyn_img = image::DynamicImage::ImageRgba8(rgba);
            app.load_dynamic_image(ctx, dyn_img);
            ctx.request_repaint();
            return;
        }
    }

    // 2. Fallback: Check if clipboard holds a file path (e.g. copied file in Explorer)
    if let Ok(text) = clipboard.get_text()
    {
        // Strip quotes or surrounding whitespace Windows Explorer might add
        let clean_path = text.trim().trim_matches('"');
        let path = std::path::Path::new(clean_path);

        if path.is_file()
        {
            if let Ok(loaded_img) = image::open(path)
            {
                app.image = Some(loaded_img);
                app.texture = None; // Signal egui to rebuild the GPU texture
                ctx.request_repaint();
            }
        }
    }
}

// ── Image Adjustments ────────────────────────────────────────────────────────

// Rebuilds the preview texture from the stored thumbnail using current AdjustState values.
pub fn update_adjust_preview(ctx: &egui::Context, state: &mut AdjustState)
{
    if let Some(ref base) = state.thumbnail_base
    {
        let result = apply_adjustments_to_buffer(&base.to_rgba8(), state);
        let size   = [result.width() as usize, result.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            size,
            result.as_flat_samples().as_slice(),
        );
        state.preview_texture = Some(ctx.load_texture(
            "adjust_preview",
            color_image,
            Default::default(),
        ));
    }
}

// Applies all adjustments from `state` to `img` destructively, returning the result.
pub fn apply_adjustments(img: &DynamicImage, state: &AdjustState) -> DynamicImage
{
    DynamicImage::ImageRgba8(apply_adjustments_to_buffer(&img.to_rgba8(), state))
}

/*
 * Applies all adjustments from `state` to only the `(x, y, width, height)`
 * sub-region of `img`, leaving every other pixel in the image untouched.
 *
 * Used for the "adjust selection" flow: the full image is cloned, just the
 * selected rectangle is cropped out and run through the same
 * `apply_adjustments_to_buffer` pipeline the whole-image path and the live
 * preview use, and the adjusted result is stamped back into the clone at
 * its original position. `rect` is expected to already be clamped to the
 * image bounds (see `selection_to_image_rect`).
 */
pub fn apply_adjustments_region(
    img:   &DynamicImage,
    state: &AdjustState,
    rect:  (u32, u32, u32, u32),
) -> DynamicImage
{
    let (x, y, w, h) = rect;

    let mut base = img.to_rgba8();
    let region = image::imageops::crop_imm(img, x, y, w, h).to_image();
    let adjusted_region = apply_adjustments_to_buffer(&region, state);

    image::imageops::replace(&mut base, &adjusted_region, x as i64, y as i64);

    DynamicImage::ImageRgba8(base)
}

/*
 * Core adjustment pipeline shared by the live preview and the full-resolution
 * "apply" path, so both always produce identical results.
 *
 * Phase 1: brightness and contrast are applied using the `image` crate's
 * built-in ops, each as its own full buffer pass.
 *
 * Phase 2: a single combined per-pixel loop applies, in order: additive
 * RGB channel offsets, a gamma curve (via a precomputed 256-entry lookup
 * table instead of calling `powf` per pixel/channel for performance), and
 * saturation, computed as a linear interpolation between the pixel's
 * luminance-weighted grayscale value and its own color (state.saturation
 * values >1 boost saturation, <1 desaturate, 0 = full grayscale). Alpha is
 * left untouched throughout.
 */
fn apply_adjustments_to_buffer(
    src:   &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    state: &AdjustState,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>>
{
    let brightened = image::imageops::brighten(src, state.brightness);
    let contrasted = image::imageops::contrast(&brightened, state.contrast);

    // Pre-compute gamma LUT to avoid per-pixel powf()
    let gamma_inv = 1.0_f32 / state.gamma.max(0.01);
    let gamma_lut: [u8; 256] = std::array::from_fn(|i|
    {
        ((i as f32 / 255.0).powf(gamma_inv) * 255.0)
            .clamp(0.0, 255.0) as u8
    });

    // Single pass: additive RGB offset + gamma + saturation
    let mut result = contrasted;
    for pixel in result.pixels_mut()
    {
        // Additive channel offsets
        let r = (pixel[0] as i32 + state.red  ).clamp(0, 255) as u8;
        let g = (pixel[1] as i32 + state.green).clamp(0, 255) as u8;
        let b = (pixel[2] as i32 + state.blue ).clamp(0, 255) as u8;

        // Gamma via LUT
        let r = gamma_lut[r as usize];
        let g = gamma_lut[g as usize];
        let b = gamma_lut[b as usize];

        // Saturation — luminance-weighted linear approximation
        let rf = r as f32;
        let gf = g as f32;
        let bf = b as f32;
        let gray = 0.299 * rf + 0.587 * gf + 0.114 * bf;
        let sat  = state.saturation;

        pixel[0] = (gray + sat * (rf - gray)).clamp(0.0, 255.0) as u8;
        pixel[1] = (gray + sat * (gf - gray)).clamp(0.0, 255.0) as u8;
        pixel[2] = (gray + sat * (bf - gray)).clamp(0.0, 255.0) as u8;
        // pixel[3] (alpha) intentionally preserved
    }

    result
}