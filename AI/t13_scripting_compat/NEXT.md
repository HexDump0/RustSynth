# T13 Next

## Recommended next task: T14 — GTK4 + Relm4 desktop app shell

T10A is complete, which unblocks both T14 and T17.

**Start with T14** (app shell) before T17 (wgpu viewport) because:
- The app shell provides the window, GTK `GtkGLArea` surface, and Relm4 component tree that T17 depends on.
- T14 can be scaffolded and tested with a placeholder viewport widget while T17 is in progress.
- T17's wgpu renderer needs an EGL surface handle that only exists once T14 creates the `GtkGLArea`.

## After T14

T17 — `wgpu` viewport backend (rendering into the `GtkGLArea` created by T14).

## Parallel work

T18, T19, T20, T21 can all be prototyped once T14's component skeleton exists, even before T17 is complete.
