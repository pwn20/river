# River - Rust Image ViewER -- A Paul-only feature replacement for Irfanview for Linux

## Features - Simple
[ ] Settings modal
[x] Fullscreen mode
[x] Fully borderless window mode with draggable edges/sides
[ ] About help menu and modal (very low priority)
[x] Adjustable selection box for zooming
[ ] Find out how to make Linux accept the app under "Open With".
[ ] Add "add_enabled()" stuff to *all* the app menu bar commands other than "Open File" and "Close"?

## Features - Complex
[x] Adjust R/G/B levels, contrast, brightness, saturation via modal with static thumbnail and live thumbnail preview
[x] Horizontal flip
[x] Vertical flip
[x] Arbitrary rotation via modal
[ ] fixed rotation via key bind/menu
[ ] Save/Save As modal and functionality
[x] Linux cross-compilation (build with "cargo linux")
[ ] Add/build out the custom view mode that sits between "100% zoom" and "fit to window". (or not?)
[ ] Undo (undo rotation, flip, RGB/contrast/bright/gamma/saturation, change size), multiple levels.
[ ] Triggering image adjusment window when a selection is drawn will only edit that selection
[ ] Actually implement (or fix if broken) the "drag and drop image onto app to load" feature.

## Changes
[ ] Currently egui adopts the system color scheme of light/dark. That's fine, but let's make the window background static dark gray or black. Full white is awful.
[ ] Center the Apply and Cancel buttons in the rotation modal; add decent spacing between them.
[ ] The clipboard copy really won't work now that we're doing GPU image flips instead of software.
[ ] The editor (R/G/B etc) modal lives inside the window, it needs to become its own window to escape size constraints.

## Bug fixes
[x] The image thumbnail preview in the rotation modal is stretched to fit the modal. We should fit it based on the longest axis and then apply its ratio and then center it based on the most narrow axis.
[x] Find out why copy-to-clipboard has code but doesn't work.
[ ] Entering fullscreen mode when the window is maximized causes a black bar to appear beneath the image where the Windows task bar used to be. This doesn't happen if you activate fullscreen when the window is not maximized.

## Settings Modal
[ ] Zoom factor stuff
[ ] Background color
[ ] Default view mode?
[ ] Key binds? This could be messy.

## Misc things
[x] Ask an LLM why Ifranview is faster at image flips than RIVER is when RIVER is supposedly using the GPU. [FIXED]
[ ] Have the LLM (which wrote the code in the first place) examine the initial image loading and display for potential optimizations like we got with the image flip.