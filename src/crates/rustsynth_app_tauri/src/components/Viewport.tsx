import { useMemo, useRef, useEffect, memo } from "react";
import { Canvas, useThree } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport } from "@react-three/drei";
import * as THREE from "three";
import type { OrbitControls as OrbitControlsImpl } from "three-stdlib";
import type { Scene, SceneObject, PrimitiveKind } from "../types";

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
}

export function Viewport({ scene }: ViewportProps) {
  const controlsRef = useRef<OrbitControlsImpl | null>(null);

  const bgColor = useMemo(() => {
    if (scene?.background) {
      const { r, g, b } = scene.background;
      return new THREE.Color(r, g, b);
    }
    return new THREE.Color(0x111111);
  }, [scene?.background]);

  return (
    <div className="relative w-full h-full">
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

        {scene && <SceneRenderer objects={scene.objects} />}

        <OrbitControls ref={controlsRef} makeDefault enableDamping dampingFactor={0.1} />
        <GizmoHelper alignment="bottom-right" margin={[60, 60]}>
          <GizmoViewport labelColor="white" axisHeadScale={0.8} />
        </GizmoHelper>
      </Canvas>

      <button
        onClick={() => {
          controlsRef.current?.reset();
          controlsRef.current?.update();
        }}
        className="absolute bottom-3 left-3 bg-ctp-crust p-2 transition-colors cursor-pointer text-ctp-subtext1 hover:text-ctp-mauve"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16"><path fill="currentColor" d="M14 8a5.98 5.98 0 0 1-1.759 4.243A5.98 5.98 0 0 1 8 14a6 6 0 0 1-4.243-1.758a6 6 0 0 1-1.285-1.905a.75.75 0 1 1 1.38-.586c.229.537.553 1.019.966 1.43c.412.414.894.738 1.431.967a4.55 4.55 0 0 0 3.503 0a4.5 4.5 0 0 0 1.429-.965A4.5 4.5 0 0 0 12.501 8a4.48 4.48 0 0 0-1.319-3.181a4.5 4.5 0 0 0-1.431-.967a4.55 4.55 0 0 0-3.503 0A4.5 4.5 0 0 0 4.257 5.5h1.992a.75.75 0 0 1 0 1.5h-3.5a.75.75 0 0 1-.75-.75v-3.5a.75.75 0 0 1 1.5 0v1.282q.124-.142.259-.275a6 6 0 0 1 1.904-1.285a6.04 6.04 0 0 1 4.673 0a6 6 0 0 1 1.907 1.286A6 6 0 0 1 13.999 8z"/></svg>
      </button>
    </div>
  );
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
