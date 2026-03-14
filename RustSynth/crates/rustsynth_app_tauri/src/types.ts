// Types mirroring the Rust scene representation.
// These match the serde JSON output from the Tauri backend.

export interface Rgba {
  r: number;
  g: number;
  b: number;
  a: number;
}

export type PrimitiveKind =
  | "Box"
  | "Sphere"
  | "Cylinder"
  | "Mesh"
  | "Line"
  | "Dot"
  | "Grid"
  | "Template"
  | { Triangle: string };

export interface SceneObject {
  kind: PrimitiveKind;
  // Column-major 4x4 matrix as [c0x,c0y,c0z,c0w, c1x,...] (glam convention)
  transform: number[];
  color: Rgba;
  alpha: number;
  tag: string | null;
}

export interface CameraState {
  translation: [number, number, number];
  rotation: number[]; // 16 floats, column-major
  pivot: [number, number, number];
  scale: number;
}

export interface Scene {
  objects: SceneObject[];
  camera: CameraState | null;
  background: Rgba | null;
  raw_settings: [string, string][];
}

export interface GuiParam {
  Float?: { name: string; default: number; min: number; max: number };
  Int?: { name: string; default: number; min: number; max: number };
}

export interface BuildConfig {
  max_generations: number;
  max_objects: number;
  min_dim: number;
  max_dim: number;
  sync_random: boolean;
  mode: "BreadthFirst" | "DepthFirst";
  seed: number;
}

export interface PipelineResult {
  scene: Scene;
  warnings: string[];
  gui_params: GuiParam[];
}
