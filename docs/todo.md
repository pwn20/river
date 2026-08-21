# River — Rust Image Viewer

*A Paul-only feature replacement for IrfanView on Linux*

---

## Simple Features

- [x] Fullscreen mode
- [x] Fully borderless window mode with draggable edges/sides
- [x] Adjustable selection box for zooming
- [ ] Settings modal
- [ ] Add `add_enabled()` guards to all menu bar commands other than "Open File" and "Close"
- [ ] Figure out how to make Linux recognize the app under "Open With"
- [ ] About / Help menu and modal *(very low priority)*

## Complex Features

- [x] Adjust R/G/B levels, contrast, brightness, and saturation via a modal with a static "before" thumbnail and a live preview
- [x] Horizontal flip
- [x] Vertical flip
- [x] Arbitrary rotation via modal
- [x] Linux cross-compilation (`cargo linux`)
- [ ] Fixed-increment rotation via keybind/menu
- [ ] Save / Save As modal and functionality
- [ ] Custom view mode between "100% zoom" and "fit to window" *(TBD — may not be worth it)*
- [ ] Undo support (rotation, flip, RGB/contrast/brightness/gamma/saturation, resize), with multiple undo levels
- [ ] Restrict the adjustment modal to the active selection, if one is drawn
- [ ] Implement (or fix, if broken) drag-and-drop image loading

## Changes

- [ ] Stop following the system light/dark color scheme for the window background — use a static dark gray or black instead (pure white is jarring).
- [ ] Center the Apply/Cancel buttons in the rotation modal and add proper spacing between them.
- [ ] Fix clipboard copy, which no longer works now that flips are done on the GPU instead of in software.
- [ ] Break the R/G/B adjustment modal out into its own window so it isn't constrained by the main window's size.
- [ ] Make the sliders in the image adjustment window wider.

## Bug Fixes

- [x] Rotation modal thumbnail preview was stretched to fit — now scaled to the longest axis, with aspect ratio preserved and centered on the shorter axis
- [x] Diagnosed and fixed copy-to-clipboard, which had code in place but wasn't working
- [ ] Entering fullscreen while the window is maximized leaves a black bar where the taskbar used to be; doesn't occur when entering fullscreen from a non-maximized state
- [ ] Dragging the saturation control to the left until the text control reads "0.00" should *stop* adjustments, but instead you can continue dragging to the left and the image resaturates.

## Settings Modal — Planned Contents

- [ ] Zoom factor behavior
- [ ] Background color
- [ ] Default view mode
- [ ] Keybinds *(could get messy)*

## Misc / Research

- [x] Asked an LLM why IrfanView flips images faster than River, despite River supposedly using the GPU — **resolved**
- [ ] Have the LLM that wrote River's code review the initial image loading/display path for optimizations, similar to the image-flip fix