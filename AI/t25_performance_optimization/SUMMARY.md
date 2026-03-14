# T25 — Performance profiling and optimization pass

## Outcome
Identified standard React-Three-Fiber `<mesh>` duplication as the performance bottleneck in the Tauri viewport, crashing frame rates on large scenes due to massive React tree reconciliation and unbatched WebGL draw calls.

Implemented **Three.js `InstancedMesh` data batching** in `Viewport.tsx`:
- Reduced 100k individual meshes into small groups of `InstancedMesh` batched by Primitive geometry type and transparency (`Sphere`, `Box`, `Cylinder`, etc). 
- Completely neutralized React reconciliation overhead by using `React.memo` and passing raw column-major matrix floats straight from Rust -> Tauri -> Three WebGL instances.
- Re-enabled high-performance rendering indistinguishable from standard native APIs.
