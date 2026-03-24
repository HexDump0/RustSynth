import { useMemo, useRef, useEffect, memo } from "react";
import { Canvas, useThree } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport } from "@react-three/drei";
import * as THREE from "three";
import type { OrbitControls as OrbitControlsImpl } from "three-stdlib";
import type { Scene, SceneObject, PrimitiveKind, CameraState } from "../types";

// ── Shared geometry singletons (avoid recreating per InstancedGroup) ──
const SHARED_GEO: Record<string, THREE.BufferGeometry> = {};
function getSharedGeometry(kind: string): THREE.BufferGeometry {
  if (SHARED_GEO[kind]) return SHARED_GEO[kind];
  let geo: THREE.BufferGeometry;
  switch (kind) {
    case "Box":
    case "Grid":
    case "Mesh":
    case "Template":
    case "Unknown":
      geo = new THREE.BoxGeometry(1, 1, 1);
      break;
    case "Sphere":
      geo = new THREE.SphereGeometry(0.5, 24, 16);
      break;
    case "Cylinder":
      geo = new THREE.CylinderGeometry(0.5, 0.5, 1, 24);
      break;
    case "Line":
      geo = new THREE.CylinderGeometry(0.02, 0.02, 1, 6);
      // Legacy line primitive is along local X, centered on the YZ plane.
      geo.rotateZ(Math.PI / 2);
      break;
    case "Dot":
      geo = new THREE.SphereGeometry(0.05, 8, 6);
      break;
    default:
      geo = new THREE.BoxGeometry(1, 1, 1);
      break;
  }
  SHARED_GEO[kind] = geo;
  return geo;
}

// ── Reusable temp objects (avoid allocations in hot loops) ──
const _tmpMatrix = new THREE.Matrix4();
const _tmpColor = new THREE.Color();

interface ViewportProps {
  scene: Scene | null;
  onInsertCameraToCode?: (camera: {
    eye: [number, number, number];
    target: [number, number, number];
    up: [number, number, number];
    fov?: number;
  }) => void;
}

export function Viewport({ scene, onInsertCameraToCode }: ViewportProps) {
  const controlsRef = useRef<OrbitControlsImpl | null>(null);
  const simpleCamera = useMemo(
    () => parseSimpleCameraSettings(scene?.raw_settings ?? []),
    [scene?.raw_settings],
  );

  const bgColor = useMemo(() => {
    if (scene?.background) {
      const { r, g, b } = scene.background;
      return new THREE.Color(r, g, b);
    }
    return new THREE.Color(0x111111);
  }, [scene?.background]);

  return (
    <div id="tour-viewport" className="relative w-full h-full">
      <Canvas
        camera={{ position: [3, 3, 5], fov: 45, near: 0.01, far: 1000 }}
        style={{ width: "100%", height: "100%" }}
        gl={{ antialias: true, powerPreference: "high-performance" }}
        frameloop="demand"
      >
        <color attach="background" args={[bgColor.r, bgColor.g, bgColor.b]} />
        <ambientLight intensity={0.4} />
        <directionalLight position={[5, 8, 5]} intensity={0.8} />
        <directionalLight position={[-3, 2, -4]} intensity={0.3} />

        <ApplySceneCamera settings={simpleCamera} />

        {scene && (
          <group matrixAutoUpdate={false} matrix={simpleCamera ? new THREE.Matrix4().identity() : cameraSettingsMatrix(scene.camera)}>
            <SceneRenderer objects={scene.objects} />
          </group>
        )}

        <OrbitControls ref={controlsRef} makeDefault enableDamping dampingFactor={0.1} />
        <GizmoHelper alignment="bottom-right" margin={[60, 60]}>
          <GizmoViewport labelColor="white" axisHeadScale={0.8} />
        </GizmoHelper>
      </Canvas>

      <div className="absolute bottom-3 left-3 flex items-center gap-2">
        <button
          onClick={() => {
            const controls = controlsRef.current;
            if (!controls || !onInsertCameraToCode) return;

            const eye: [number, number, number] = [
              controls.object.position.x,
              controls.object.position.y,
              controls.object.position.z,
            ];
            const target: [number, number, number] = [
              controls.target.x,
              controls.target.y,
              controls.target.z,
            ];
            const up: [number, number, number] = [
              controls.object.up.x,
              controls.object.up.y,
              controls.object.up.z,
            ];

            let fov: number | undefined;
            if ("fov" in controls.object && typeof controls.object.fov === "number") {
              fov = controls.object.fov;
            }

            onInsertCameraToCode({ eye, target, up, fov });
          }}
          title="Insert current camera into script"
          className="bg-ctp-crust p-2 transition-colors cursor-pointer text-ctp-subtext1 hover:text-ctp-mauve font-medium"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16"><path fill="#cdd6f4" d="M3.5 3A2.5 2.5 0 0 0 1 5.5v5A2.5 2.5 0 0 0 3.5 13h5a2.5 2.5 0 0 0 2.5-2.5v-.127l2.035 1.405a1.25 1.25 0 0 0 1.96-1.028V5.252a1.25 1.25 0 0 0-1.96-1.028L11 5.629V5.5A2.5 2.5 0 0 0 8.5 3zM11 6.844l2.604-1.798a.25.25 0 0 1 .392.206v5.498a.25.25 0 0 1-.392.205L11 9.158zM2 5.5A1.5 1.5 0 0 1 3.5 4h5A1.5 1.5 0 0 1 10 5.5v5A1.5 1.5 0 0 1 8.5 12h-5A1.5 1.5 0 0 1 2 10.5z"/></svg>
        </button>

        <button
          onClick={() => {
            controlsRef.current?.reset();
            controlsRef.current?.update();
          }}
          title="Reset camera"
          className="bg-ctp-crust p-2 transition-colors cursor-pointer text-ctp-subtext1 hover:text-ctp-mauve"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16"><path fill="currentColor" d="M14 8a5.98 5.98 0 0 1-1.759 4.243A5.98 5.98 0 0 1 8 14a6 6 0 0 1-4.243-1.758a6 6 0 0 1-1.285-1.905a.75.75 0 1 1 1.38-.586c.229.537.553 1.019.966 1.43c.412.414.894.738 1.431.967a4.55 4.55 0 0 0 3.503 0a4.5 4.5 0 0 0 1.429-.965A4.5 4.5 0 0 0 12.501 8a4.48 4.48 0 0 0-1.319-3.181a4.5 4.5 0 0 0-1.431-.967a4.55 4.55 0 0 0-3.503 0A4.5 4.5 0 0 0 4.257 5.5h1.992a.75.75 0 0 1 0 1.5h-3.5a.75.75 0 0 1-.75-.75v-3.5a.75.75 0 0 1 1.5 0v1.282q.124-.142.259-.275a6 6 0 0 1 1.904-1.285a6.04 6.04 0 0 1 4.673 0a6 6 0 0 1 1.907 1.286A6 6 0 0 1 13.999 8z"/></svg>
        </button>
      </div>
    </div>
  );
}

type SimpleCameraSettings = {
  eye: [number, number, number];
  target: [number, number, number];
  up?: [number, number, number];
  fov?: number;
};

function parseBracketVec(value: string): [number, number, number] | null {
  const inner = value.trim().match(/^\[(.*)\]$/)?.[1];
  if (!inner) return null;
  const nums = inner
    .trim()
    .split(/\s+/)
    .map(Number)
    .filter(n => Number.isFinite(n));
  if (nums.length < 3) return null;
  return [nums[0], nums[1], nums[2]];
}

function parseSimpleCameraSettings(rawSettings: [string, string][]): SimpleCameraSettings | null {
  let eye: [number, number, number] | null = null;
  let target: [number, number, number] | null = null;
  let up: [number, number, number] | undefined;
  let fov: number | undefined;

  for (const [rawKey, rawValue] of rawSettings) {
    const key = rawKey.toLowerCase();
    const value = rawValue.trim();

    if (key === "camera") {
      const inner = value.match(/^\[(.*)\]$/)?.[1];
      if (inner) {
        const nums = inner
          .trim()
          .split(/\s+/)
          .map(Number)
          .filter(n => Number.isFinite(n));
        if (nums.length >= 6) {
          eye = [nums[0], nums[1], nums[2]];
          target = [nums[3], nums[4], nums[5]];
        }
        if (nums.length >= 9) {
          up = [nums[6], nums[7], nums[8]];
        }
      }
      continue;
    }

    if (key === "camera_eye") {
      eye = parseBracketVec(value);
      continue;
    }
    if (key === "camera_target") {
      target = parseBracketVec(value);
      continue;
    }
    if (key === "camera_up") {
      up = parseBracketVec(value) ?? undefined;
      continue;
    }
    if (key === "camera_fov") {
      const n = Number(value);
      if (Number.isFinite(n) && n > 1 && n < 179) {
        fov = n;
      }
      continue;
    }
  }

  if (!eye || !target) return null;
  return { eye, target, up, fov };
}

function ApplySceneCamera({ settings }: { settings: SimpleCameraSettings | null }) {
  const camera = useThree(s => s.camera);
  const controls = useThree(s => s.controls as OrbitControlsImpl | undefined);
  const invalidate = useThree(s => s.invalidate);

  useEffect(() => {
    if (!settings) return;

    camera.position.set(settings.eye[0], settings.eye[1], settings.eye[2]);
    if (settings.up) {
      camera.up.set(settings.up[0], settings.up[1], settings.up[2]);
    }

    if ("fov" in camera && typeof settings.fov === "number") {
      camera.fov = settings.fov;
      camera.updateProjectionMatrix();
    }

    if (controls) {
      controls.target.set(settings.target[0], settings.target[1], settings.target[2]);
      controls.update();
      controls.saveState();
    } else {
      camera.lookAt(settings.target[0], settings.target[1], settings.target[2]);
    }

    invalidate();
  }, [settings, camera, controls, invalidate]);

  return null;
}

function cameraSettingsMatrix(camera: CameraState | null): THREE.Matrix4 {
  if (!camera) return new THREE.Matrix4().identity();

  const [tx, ty, tz] = camera.translation;
  const [px, py, pz] = camera.pivot;
  const scale = Number.isFinite(camera.scale) && camera.scale !== 0 ? camera.scale : 1;

  const t = new THREE.Matrix4().makeTranslation(tx, ty, tz);
  const p = new THREE.Matrix4().makeTranslation(px, py, pz);
  const pInv = new THREE.Matrix4().makeTranslation(-px, -py, -pz);
  const r = new THREE.Matrix4().fromArray(camera.rotation);
  const s = new THREE.Matrix4().makeScale(scale, scale, scale);

  // Legacy Structure Synth view transform convention:
  // T(translation) * T(pivot) * R(rotation) * S(scale) * T(-pivot)
  return t.multiply(p).multiply(r).multiply(s).multiply(pInv);
}

// ── Pre-computed instanced group data ──
interface InstancedGroupData {
  kind: string;
  opacity: number;
  // Flat Float32Arrays for direct buffer writes — no per-object class instantiation
  matrices: Float32Array;
  colors: Float32Array;
  count: number;
}

const SceneRenderer = memo(function SceneRenderer({ objects }: { objects: SceneObject[] }) {
  const { groups, nonInstanced } = useMemo(() => {
    // Phase 1: bucket objects by kind+opacity, count sizes
    const buckets = new Map<string, { kind: string; opacity: number; objs: SceneObject[] }>();
    const nonInstanced: SceneObject[] = [];

    for (let i = 0; i < objects.length; i++) {
      const obj = objects[i];
      const kind = normalizePrimitiveKind(obj.kind);
      if (kind === "Triangle") {
        nonInstanced.push(obj);
        continue;
      }

      const opacity = obj.alpha * obj.color.a;
      const opacityKey = opacity < 1.0 ? opacity.toFixed(3) : "1";
      const key = `${kind}_${opacityKey}`;

      let bucket = buckets.get(key);
      if (!bucket) {
        bucket = { kind, opacity, objs: [] };
        buckets.set(key, bucket);
      }
      bucket.objs.push(obj);
    }

    // Phase 2: build flat typed arrays for each group (single allocation, no per-object classes)
    const groups: InstancedGroupData[] = [];
    for (const [, bucket] of buckets) {
      const count = bucket.objs.length;
      const matrices = new Float32Array(count * 16);
      const colors = new Float32Array(count * 3);

      for (let i = 0; i < count; i++) {
        const obj = bucket.objs[i];
        // Copy 16 floats directly (already column-major from glam)
        const t = obj.transform;
        const mo = i * 16;
        matrices[mo]      = t[0];  matrices[mo + 1]  = t[1];  matrices[mo + 2]  = t[2];  matrices[mo + 3]  = t[3];
        matrices[mo + 4]  = t[4];  matrices[mo + 5]  = t[5];  matrices[mo + 6]  = t[6];  matrices[mo + 7]  = t[7];
        matrices[mo + 8]  = t[8];  matrices[mo + 9]  = t[9];  matrices[mo + 10] = t[10]; matrices[mo + 11] = t[11];
        matrices[mo + 12] = t[12]; matrices[mo + 13] = t[13]; matrices[mo + 14] = t[14]; matrices[mo + 15] = t[15];

        const co = i * 3;
        colors[co]     = obj.color.r;
        colors[co + 1] = obj.color.g;
        colors[co + 2] = obj.color.b;
      }

      groups.push({ kind: bucket.kind, opacity: bucket.opacity, matrices, colors, count });
    }

    return { groups, nonInstanced };
  }, [objects]);

  return (
    <>
      {groups.map((g, i) => (
        <InstancedGroup key={`${g.kind}_${g.opacity}_${i}`} data={g} />
      ))}
      {nonInstanced.map((obj, i) => (
        <TrianglePrimitive key={i} obj={obj} />
      ))}
    </>
  );
});

// ── Instanced group: writes pre-built typed arrays directly to GPU buffers ──

function InstancedGroup({ data }: { data: InstancedGroupData }) {
  const meshRef = useRef<THREE.InstancedMesh>(null);
  const invalidate = useThree(s => s.invalidate);
  const { kind, opacity, matrices, colors, count } = data;

  const transparent = opacity < 1.0;
  const isWireframe = kind === "Grid" || kind === "Mesh" || kind === "Template" || kind === "Unknown";
  const side = kind === "Grid" ? THREE.DoubleSide : THREE.FrontSide;

  const geometry = useMemo(() => getSharedGeometry(kind), [kind]);

  const material = useMemo(
    () =>
      new THREE.MeshStandardMaterial({
        opacity,
        transparent,
        wireframe: isWireframe,
        side,
      }),
    [opacity, transparent, isWireframe, side],
  );

  useEffect(() => {
    const mesh = meshRef.current;
    if (!mesh) return;

    // Write instance matrices directly from the flat Float32Array
    mesh.instanceMatrix = new THREE.InstancedBufferAttribute(matrices, 16);
    mesh.instanceMatrix.needsUpdate = true;

    // Write instance colors directly
    mesh.instanceColor = new THREE.InstancedBufferAttribute(colors, 3);
    mesh.instanceColor.needsUpdate = true;

    // Recompute bounding sphere for frustum culling
    mesh.computeBoundingSphere();

    invalidate();
  }, [matrices, colors, invalidate]);

  return (
    <instancedMesh
      ref={meshRef}
      args={[geometry, material, count]}
      matrixAutoUpdate={false}
      frustumCulled={false}
    />
  );
}

// ── Triangle primitive (non-instanced, typically rare) ──

function TrianglePrimitive({ obj }: { obj: SceneObject }) {
  const geometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    let vertexStr = "";
    if (typeof obj.kind === "object" && "Triangle" in obj.kind) {
      vertexStr = obj.kind.Triangle;
    }
    const cleaned = vertexStr.replace(/[\[\]]/g, "").trim();
    const parts = cleaned.split(";").map(s => s.trim().split(/\s+/).map(Number));
    if (parts.length === 3 && parts.every(p => p.length === 3)) {
      const verts = new Float32Array([...parts[0], ...parts[1], ...parts[2]]);
      geo.setAttribute("position", new THREE.BufferAttribute(verts, 3));
      geo.computeVertexNormals();
    }
    return geo;
  }, [obj.kind]);

  const matrix = useMemo(() => _tmpMatrix.clone().fromArray(obj.transform), [obj.transform]);
  const color = useMemo(() => new THREE.Color(obj.color.r, obj.color.g, obj.color.b), [obj.color]);
  const opacity = obj.alpha * obj.color.a;

  return (
    <mesh matrixAutoUpdate={false} matrix={matrix} geometry={geometry}>
      <meshStandardMaterial
        color={color}
        opacity={opacity}
        transparent={opacity < 1.0}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

function normalizePrimitiveKind(kind: PrimitiveKind): string {
  if (typeof kind === "string") return kind;
  if ("Triangle" in kind) return "Triangle";
  return "Unknown";
}
