# River — Rust Image Viewer

River is a high-performance, lightweight image viewer written in Rust. It utilizes [`egui` / `eframe`](file:///d:/Documents/Programming/Rust/river/Cargo.toml) with the `glow` OpenGL backend to achieve fast, responsive rendering, and utilizes the [`image`](file:///d:/Documents/Programming/Rust/river/Cargo.toml) and [`imageproc`](file:///d:/Documents/Programming/Rust/river/Cargo.toml) crates for image decoding, manipulation, and transformations.

This document provides a thorough overview of River's codebase architecture, module responsibilities, core state models, image processing mechanics, and Windows 7 compatibility subsystem.

For the project roadmap and current tasks, please refer to the main [`todo.md`](file:///d:/Documents/Programming/Rust/river/docs/todo.md) file in the `docs` directory.

---

## Codebase Architecture

The application is structured into the main application manager, helper utilities for image transformations, a Windows 7 compatibility thunking layer, and modular UI components under the `ui` directory:

```
river/
├── Cargo.toml                  # Dependencies & target settings
├── build.rs                    # Delays combase.dll loading & bundles Mesa3D on Win7
├── docs/
│   └── todo.md                 # Current feature list, bugs, and roadmap
├── mesa3d_dlls/                # Bundled OpenGL drivers for Win7 software rendering
└── src/
    ├── main.rs                 # Entry point, crash catcher, and eframe initialization
    ├── app.rs                  # Core app state & lifecycle methods
    ├── state.rs                # State containers and layout modes
    ├── image_utils.rs          # Pixel mapping, clipboard, and color adjustments
    ├── win7.rs                 # Win7 compatibility hooks (delay-load error handler)
    └── ui/
        ├── mod.rs              # Main UI assembly, keyboard inputs
        ├── menu.rs             # Top menu bar layout
        ├── main_view.rs        # Central viewport: canvas, pan/zoom, selections
        ├── dialogs.rs          # Image resizing and arbitrary rotation modals
        └── adjust_dialog.rs    # RGB, contrast, brightness, saturation editor viewport
```

---

## File Breakdown and Component Roles

### 1. Main Entry Point: [`main.rs`](file:///d:/Documents/Programming/Rust/river/src/main.rs)

The main function initializes the program:
- **Panic Catcher**: Configures a global panic hook via `std::panic::set_hook` that captures backtraces and writes them to `river_crash.log` in the executable's directory.
- **Initial Arguments**: Captures the first command-line argument as an optional [`PathBuf`](file:///d:/Documents/Programming/Rust/river/src/main.rs#L32) to load an image passed by the OS (e.g., when clicking "Open With" in a file manager).
- **Viewport Setup**: Configures default native options for the `egui` viewport, such as an initial size of 800x400.
- **Launcher**: Launches `eframe::run_native` initializing [`ImageViewerApp`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L16).

---

### 2. Application Logic: [`app.rs`](file:///d:/Documents/Programming/Rust/river/src/app.rs)

Contains [`ImageViewerApp`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L16) which represents the top-level application state, driving the rendering frame and holding references to:
- **Image Cache**: The current CPU pixel buffer (`image: Option<DynamicImage>`) and the corresponding GPU texture handle (`texture: Option<egui::TextureHandle>`).
- **Viewport Control**: The active [`ViewMode`](file:///d:/Documents/Programming/Rust/river/src/state.rs#L13), `zoom_factor`, and `pan_offset`.
- **Windowing Chrome**: Boolean states indicating whether window decoration is shown (`show_chrome`) or fullscreen mode is active (`is_fullscreen`).
- **Flipping States**: `flipped_h` and `flipped_v` flags used to mirror the image inside the shader without altering the CPU pixel buffer.
- **Sub-Dialog states**: Backing states for modal dialogs (resize, rotate, image adjustments).
- **Selection Marquee**: `selection_start` and `selection_end` vectors, mapped inside the visible `image_rect` boundary.

#### Key Functions
- [`open_image`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L149): Opens the native OS file picker via the `rfd` library or consumes the `initial_file` argument.
- [`load_dynamic_image`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L184): Uploads decoded `DynamicImage` pixels onto the GPU texture space.
- [`open_rotate_dialog`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L206): Scales down the active image to generate a lightweight thumbnail (max 400px) for quick preview updates in the rotation dialog.
- [`open_adjust_dialog`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L232): Initializes the color adjustment state, checking for selection limits to scope modifications.
- [`handle_window_resize_edges`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L332): Implements custom borders resizing support when `show_chrome` is disabled, manually checking cursor proximity to viewport edges and triggering `ViewportCommand::BeginResize`.

---

### 3. Application State: [`state.rs`](file:///d:/Documents/Programming/Rust/river/src/state.rs)

Defines data models that group specific configurations for dialogs or viewer settings:
- [`ViewMode`](file:///d:/Documents/Programming/Rust/river/src/state.rs#L13): Enumeration of sizing modes:
  - `FitToWindow`: Recomputes scale automatically to fit the pane.
  - `ActualSize`: Fixed 100% scale.
  - `Custom`: Set dynamically via scroll wheel or selection zooms.
- [`ResizeState`](file:///d:/Documents/Programming/Rust/river/src/state.rs#L24): Struct tracking dimensions, aspect locks, and scale percentages.
- [`SelectionDragMode`](file:///d:/Documents/Programming/Rust/river/src/state.rs#L45): Enumerates the active dragging context (`Idle`, `Drawing`, `Resizing(ResizeEdge)`).
- [`RotateState`](file:///d:/Documents/Programming/Rust/river/src/state.rs#L65): Holds rotation angle details and temporary GPU texture handles for fast thumbnail rotations.
- [`AdjustState`](file:///d:/Documents/Programming/Rust/river/src/state.rs#L75): Tracks color values: brightness, red/green/blue offsets, contrast, gamma, and saturation. It also maintains references to original and live-preview GPU textures, along with a `source_rect` specifying whether adjustments apply to a regional cropped selection.

---

### 4. Image Utilities: [`image_utils.rs`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs)

Performs pixel coordinates mapping, clipboard interaction, and color rendering pipelines:
- [`map_screen_to_image_pixels`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L52): Maps screen space mouse inputs onto source image dimensions, reversing coordinates based on `flipped_h` and `flipped_v` GPU status.
- [`selection_to_image_rect`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L91): Calculates normalized and sorted coordinates of a selection box to represent accurate top-left and bottom-right image coordinates.
- [`copy_selection_to_clipboard`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L132): Extracts pixels matching the selection box, constructs an RGBA buffer, and exports it using the `arboard` clipboard manager.
- [`paste_from_clipboard`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L181): Attempts to load images from the clipboard. It checks for raw pixel arrays (e.g. screenshots) first, and falls back to resolving plain-text file paths (explorer file transfers) as a secondary method.
- **Adjustment Pipeline**:
  - [`update_adjust_preview`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L231): Spawns worker threads/calls to refresh preview textures.
  - [`apply_adjustments_to_buffer`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L298): Sequentially runs contrast and brightness passes using standard `imageops`, then runs a single-pass loop applying RGB offsets, a Gamma Look-Up Table (LUT) to avoid `powf` overhead, and a luminance-weighted saturation formula.
  - [`apply_adjustments_region`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L266): Clones the image, crops out the selection, runs adjustments on that sub-region, and stamps the modified section back onto the original using `imageops::replace`.

---

### 5. UI Assembly: [`ui/mod.rs`](file:///d:/Documents/Programming/Rust/river/src/ui/mod.rs)

The manager that structures the UI frame layout:
- Drives execution order: first checks `show_chrome` to paint top menus, renders the `main_view`, calls the keyboard helper, and draws open dialog views.
- **Shortcuts & Inputs**: Manages global keybinds:
  - `Shift+C`: Copies selection.
  - `Shift+V`: Pastes image.
  - `Shift+F`: Toggles window chrome (decorations).
  - `F` / `F12`: Toggles borderless fullscreen.
  - `Z`: Toggles zoom view modes.
  - `H` / `V`: Toggles horizontal/vertical GPU flips.
  - `Shift+G`: Opens image adjustments.
  - `Ctrl+R`: Opens the resize dialog.

---

### 6. Viewport Menu: [`ui/menu.rs`](file:///d:/Documents/Programming/Rust/river/src/ui/menu.rs)

Wires top-panel option hooks:
- **File**: Open Image and Close commands.
- **Edit**: Copy and Paste actions.
- **Image**: Adjust, Resize, Rotate, Flip Horizontal, and Flip Vertical commands. The buttons are conditionally grayed out using `add_enabled(app.image.is_some(), ...)` when no image is loaded.
- **View**: Toggle view sizing modes, and trigger fullscreen.

---

### 7. Central Canvas: [`ui/main_view.rs`](file:///d:/Documents/Programming/Rust/river/src/ui/main_view.rs)

Builds the center canvas rendering space:
- **Pan and Zoom**:
  - Handles mouse scrollwheel events and pinch gestures. It anchors zoom relative to the mouse pointer by recalculating the `pan_offset`.
  - Keyboard `+`, `-`, and `=` trigger zoom centered on the viewport middle.
  - Right-click dragging modifies `pan_offset` for smooth scrolling.
- **GPU Flip Mechanics**: Swaps UV texture coordinates (min/max bounds) when drawing the image to toggle horizontal or vertical mirrors instantly:
  ```rust
  let u_min = if app.flipped_h { 1.0 } else { 0.0 };
  let u_max = if app.flipped_h { 0.0 } else { 1.0 };
  let v_min = if app.flipped_v { 1.0 } else { 0.0 };
  let v_max = if app.flipped_v { 0.0 } else { 1.0 };
  ```
- **Selection Marquee**:
  - Left-clicking and dragging draws a dotted selection marquee. The border is rendered using a dual-stroke pattern (a white line inside a black border) to preserve visibility against both light and dark backgrounds.
  - Supports resizing individual edges of an active selection box. Crucially, the target interaction mode (`Drawing`, `Resizing(Edge)`, or `Idle`) is locked in `SelectionDragMode` when the drag starts, preventing edges from shifting unexpectedly if the cursor drifts during adjustment.
  - **Zoom to Selection**: A single primary click inside an active selection box adjusts `zoom_factor` and `pan_offset` to zoom directly into the selected region, then clears the box boundaries.

---

### 8. Dialog Modals: [`ui/dialogs.rs`](file:///d:/Documents/Programming/Rust/river/src/ui/dialogs.rs)

Renders modal windows:
- **Resize Dialog**: Adjusts target pixels using numeric input fields or percentages, supporting aspect ratio locking. Applying changes performs a `Lanczos3` filter scaling operation.
- **Rotate Dialog**: Provides an angle slider (-180 to +180 degrees) with real-time thumbnail updates using bilinear interpolation. Clicking Apply transforms the full-resolution image.

---

### 9. Image Adjustments Dialog: [`ui/adjust_dialog.rs`](file:///d:/Documents/Programming/Rust/river/src/ui/adjust_dialog.rs)

Manages a dedicated viewport containing advanced editing options:
- **Multi-Window Layout**: Created as an independent immediate-mode viewport (`egui::ViewportId`) so it is not clipped by the boundaries of the primary application frame.
- **Deferred State Updates**: Since immediate-mode viewports run in separate rendering pipelines, all mutations modifying `ImageViewerApp` are stored in local variables inside the closure and executed afterward to prevent borrowing conflicts.
- **Controls**: Renders a side-by-side comparison (Original vs. Preview), and displays two columns of adjustment sliders (Brightness, Color Balance offsets, Contrast, Gamma, and Saturation).
- **Regional Application**: Adjustments are automatically applied only to the selected sub-region if a marquee selection box is active when the dialog is opened.

---

## Windows 7 Compatibility Subsystem

River features a custom Windows 7 compatibility layer to support older environments:

### 1. Linker Configuration: [`build.rs`](file:///d:/Documents/Programming/Rust/river/build.rs)
When compiled with the `win7` feature enabled, the build script:
- Invokes the `thunk-rs` build dependency to inject YY-Thunks hooks, providing modern Win32 compatibility stubs.
- Injects delay-loading options `/DELAYLOAD:combase.dll` and `/INCLUDE:__pfnDliFailureHook2` to defer dll linking.
- Bundles custom OpenGL drivers (Mesa3D software rasterizer DLLs) into the compilation directory so the GPU/OpenGL pipeline works even without newer system display drivers.

### 2. Failure Handling Hooks: [`win7.rs`](file:///d:/Documents/Programming/Rust/river/src/win7.rs)
- Intercepts delay-load failures through [`__pfnDliFailureHook2`](file:///d:/Documents/Programming/Rust/river/src/win7.rs#L123).
- If a specific function export is missing (`dliFailGetProc`), it returns a zeroed-out no-op function stub [`stub_fn`](file:///d:/Documents/Programming/Rust/river/src/win7.rs#L53) instead of raising a loading exception.
- If a dependency library fails to load (`dliFailLoadLib`), it loads `kernel32.dll` to acquire a valid system handler. This directs future API requests through the delay-load handler to resolve stubs rather than crashing during startup.

---

## Keyboard Shortcuts Reference

| Shortcut | Description | Target Component |
|---|---|---|
| **`Shift + C`** | Copy current selection to clipboard | [`image_utils::copy_selection_to_clipboard`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L132) |
| **`Shift + V`** | Paste image or path from clipboard | [`image_utils::paste_from_clipboard`](file:///d:/Documents/Programming/Rust/river/src/image_utils.rs#L181) |
| **`Shift + F`** | Toggle window decorations | [`ImageViewerApp::show_chrome`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L34) |
| **`F` / `F12`** | Toggle borderless fullscreen | [`ImageViewerApp::toggle_fullscreen`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L318) |
| **`Z`** | Toggle zoom (Actual Size vs. Fit to Window) | [`ImageViewerApp::toggle_view_mode`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L302) |
| **`H`** | Flip image horizontally (GPU-only UV swap) | [`ImageViewerApp::flipped_h`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L75) |
| **`V`** | Flip image vertically (GPU-only UV swap) | [`ImageViewerApp::flipped_v`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L77) |
| **`Shift + G`** | Open Image Adjustments Dialog | [`ImageViewerApp::open_adjust_dialog`](file:///d:/Documents/Programming/Rust/river/src/app.rs#L232) |
| **`Ctrl + R`** | Open Resize Image Dialog | [`ui::dialogs::show_resize_dialog`](file:///d:/Documents/Programming/Rust/river/src/ui/dialogs.rs#L15) |

---

## Roadmap & Outstanding Tasks

For detailed tasks and planned features, refer to [`todo.md`](file:///d:/Documents/Programming/Rust/river/docs/todo.md). Key items include:
1. **Window Sizing Adjustments**: Stop following system light/dark scheme for window background, and center buttons in the rotation modal.
2. **GPU Flip & Clipboard Integration**: Resolving conflicts where copying a selection fails when GPU flips are active.
3. **Adjustment Window UI**: Wider sliders and separate adjust modal placement.
4. **Bug Fixes**: Fixing fullscreen window black bar when entering from maximized state, and fixing saturation slider behavior below `0.00`.
