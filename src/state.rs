use eframe::egui;
use image::DynamicImage;

/* Which sizing mode the main image viewport is currently using.
 *
 * FitToWindow: scale is recomputed every frame to fit the available panel.
 * ActualSize: shown at zoom_factor == 1.0 (100%), no auto-fit.
 * Custom: any other zoom_factor set by scroll/keyboard zoom or
 * click-to-zoom-selection - once the user manually zooms, the mode
 * switches here so FitToWindow's auto-scaling doesn't override it.
 */
#[derive(PartialEq)]
pub enum ViewMode {
    FitToWindow,
    ActualSize,
    Custom,
}

/* Backing state for the "Resize Image" dialog: the target dimensions the
 * user is configuring, whether width/height changes should stay locked to
 * the original aspect ratio, and the equivalent scale percentage (kept in
 * sync with width/height, not authoritative on its own).
 */
pub struct ResizeState {
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: f32,
    pub lock_ratio: bool,
    pub percentage: f32,
}

/* Tracks what a click-and-drag on the main view is currently doing to the
 * selection box:
 *
 * Idle: no drag in progress, or a drag started inside the existing box
 * (intentionally ignored rather than treated as a move).
 * Drawing: dragging out a brand new selection box from scratch.
 * Resizing: dragging one specific edge of an existing selection box.
 *
 * This is decided once at drag-start and then read back each frame for the
 * remainder of the drag, rather than being re-derived every frame - see the
 * comment in main_view.rs for why that matters.
 */
#[derive(Clone, Copy, PartialEq)]
pub enum SelectionDragMode {
    Idle,
    Drawing,
    Resizing(ResizeEdge),
}

/// Which edge of a selection box is being hovered/dragged.
#[derive(Clone, Copy, PartialEq)]
pub enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
}

/* Backing state for the "Arbitrary Rotation" dialog: the angle the user has
 * currently dialed in, a small cached thumbnail of the source image used as
 * the base for fast re-rotation, and the resulting rotated preview texture
 * currently uploaded to the GPU.
 */
pub struct RotateState {
    pub angle_degrees: f32,
    pub thumbnail_base: Option<DynamicImage>,
    pub preview_texture: Option<egui::TextureHandle>,
}

/* Backing state for the "Image Adjustments" dialog - both the slider values
 * themselves and the cached data needed to render the before/after preview
 * panes without touching the full-resolution image on every change.
 */
pub struct AdjustState
{
    // Left-pane controls
    pub brightness: i32,    // −255 to +255, additive
    pub red:        i32,    // −128 to +128, additive channel offset
    pub green:      i32,    // −128 to +128
    pub blue:       i32,    // −128 to +128

    // Right-pane controls
    pub contrast:   f32,    // −100.0 to +100.0
    pub gamma:      f32,    //    0.1 to   5.0, default 1.0
    pub saturation: f32,    //    0.0 to   2.0, default 1.0

    // Preview data
    pub thumbnail_base:   Option<DynamicImage>,
    pub original_texture: Option<egui::TextureHandle>,
    pub preview_texture:  Option<egui::TextureHandle>,

    /* If the dialog was opened with an active selection box, this holds
     * that selection mapped into full-resolution image pixel coordinates
     * as (x, y, width, height). `thumbnail_base` and `original_texture`
     * are then built from just that sub-region rather than the whole
     * image, and "Apply" writes the adjusted result back into only that
     * region of the full image instead of replacing it entirely.
     * `None` means the dialog is operating on the whole image, same as
     * before this field existed. */
    pub source_rect: Option<(u32, u32, u32, u32)>,
}

impl AdjustState
{
    /* Resets all adjustment sliders back to their neutral/identity values
     * (i.e. the values that would leave the image visually unchanged).
     * Deliberately leaves the thumbnail/texture fields untouched, since
     * those are cache data unrelated to the slider values themselves.
     */
    pub fn reset(&mut self)
    {
        self.brightness = 0;
        self.red        = 0;
        self.green      = 0;
        self.blue       = 0;
        self.contrast   = 0.0;
        self.gamma      = 1.0;
        self.saturation = 1.0;
    }
}

impl Default for AdjustState
{
    /* Same neutral values as `reset()`, but also initializes the cache
     * fields to empty - used when constructing a brand new AdjustState
     * rather than resetting an existing one.
     */
    fn default() -> Self
    {
        Self
        {
            brightness: 0,
            red:        0,
            green:      0,
            blue:       0,
            contrast:   0.0,
            gamma:      1.0,
            saturation: 1.0,
            thumbnail_base:   None,
            original_texture: None,
            preview_texture:  None,
            source_rect:      None,
        }
    }
}