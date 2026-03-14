import { useMemo } from "react";
import { Canvas } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport } from "@react-three/drei";
import * as THREE from "three";
import type { Scene, SceneObject, PrimitiveKind } from "../types";

interface ViewportProps {
  scene: Scene | null;
}

export function Viewport({ scene }: ViewportProps) {
  const bgColor = useMemo(() => {
    if (scene?.background) {
      const { r, g, b } = scene.background;
      return new THREE.Color(r, g, b);
    }
    return new THREE.Color(0x111111);
  }, [scene?.background]);

  return (
    <Canvas
      camera={{ position: [3, 3, 5], fov: 45, near: 0.01, far: 1000 }}
      style={{ width: "100%", height: "100%" }}
      gl={{ antialias: true }}
    >
      <color attach="background" args={[bgColor.r, bgColor.g, bgColor.b]} />
      <ambientLight intensity={0.4} />
      <directionalLight position={[5, 8, 5]} intensity={0.8} />
      <directionalLight position={[-3, 2, -4]} intensity={0.3} />

      {scene && <SceneObjects objects={scene.objects} />}

      <OrbitControls makeDefault enableDamping dampingFactor={0.1} />
      <GizmoHelper alignment="bottom-right" margin={[60, 60]}>
        <GizmoViewport labelColor="white" axisHeadScale={0.8} />
      </GizmoHelper>
    </Canvas>
  );
}

function SceneObjects({ objects }: { objects: SceneObject[] }) {
  return (
    <>
      {objects.map((obj, i) => (
        <ScenePrimitive key={i} obj={obj} />
      ))}
    </>
  );
}

function ScenePrimitive({ obj }: { obj: SceneObject }) {
  const matrix = useMemo(() => {
    const m = new THREE.Matrix4();
    // glam uses column-major, Three.js fromArray expects column-major too
    m.fromArray(obj.transform);
    return m;
  }, [obj.transform]);

  const color = useMemo(
    () => new THREE.Color(obj.color.r, obj.color.g, obj.color.b),
    [obj.color],
  );

  const opacity = obj.alpha * obj.color.a;
  const transparent = opacity < 1.0;

  const kind = normalizePrimitiveKind(obj.kind);

  switch (kind) {
    case "Box":
      return (
        <mesh matrixAutoUpdate={false} matrix={matrix}>
          <boxGeometry args={[1, 1, 1]} />
          <meshStandardMaterial
            color={color}
            opacity={opacity}
            transparent={transparent}
          />
        </mesh>
      );

    case "Sphere":
      return (
        <mesh matrixAutoUpdate={false} matrix={matrix}>
          <sphereGeometry args={[0.5, 24, 16]} />
          <meshStandardMaterial
            color={color}
            opacity={opacity}
            transparent={transparent}
          />
        </mesh>
      );

    case "Cylinder":
      return (
        <mesh matrixAutoUpdate={false} matrix={matrix}>
          <cylinderGeometry args={[0.5, 0.5, 1, 24]} />
          <meshStandardMaterial
            color={color}
            opacity={opacity}
            transparent={transparent}
          />
        </mesh>
      );

    case "Line":
      return (
        <mesh matrixAutoUpdate={false} matrix={matrix}>
          <cylinderGeometry args={[0.02, 0.02, 1, 6]} />
          <meshStandardMaterial
            color={color}
            opacity={opacity}
            transparent={transparent}
          />
        </mesh>
      );

    case "Dot":
      return (
        <mesh matrixAutoUpdate={false} matrix={matrix}>
          <sphereGeometry args={[0.05, 8, 6]} />
          <meshStandardMaterial
            color={color}
            opacity={opacity}
            transparent={transparent}
          />
        </mesh>
      );

    case "Grid":
      return (
        <mesh matrixAutoUpdate={false} matrix={matrix}>
          <boxGeometry args={[1, 0.01, 1]} />
          <meshStandardMaterial
            color={color}
            opacity={opacity}
            transparent={transparent}
            wireframe
          />
        </mesh>
      );

    case "Triangle":
      return <TrianglePrimitive matrix={matrix} color={color} opacity={opacity} kind={obj.kind} />;

    default:
      // Mesh, Template, or unknown — render as wireframe box
      return (
        <mesh matrixAutoUpdate={false} matrix={matrix}>
          <boxGeometry args={[1, 1, 1]} />
          <meshStandardMaterial
            color={color}
            opacity={opacity}
            transparent={transparent}
            wireframe
          />
        </mesh>
      );
  }
}

function TrianglePrimitive({
  matrix,
  color,
  opacity,
  kind,
}: {
  matrix: THREE.Matrix4;
  color: THREE.Color;
  opacity: number;
  kind: PrimitiveKind;
}) {
  const geometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    // Parse triangle vertices from the string payload
    let vertexStr = "";
    if (typeof kind === "object" && "Triangle" in kind) {
      vertexStr = kind.Triangle;
    }
    // Format: "[x0 y0 z0; x1 y1 z1; x2 y2 z2]"
    const cleaned = vertexStr.replace(/[\[\]]/g, "").trim();
    const parts = cleaned.split(";").map(s => s.trim().split(/\s+/).map(Number));
    if (parts.length === 3 && parts.every(p => p.length === 3)) {
      const verts = new Float32Array([
        ...parts[0], ...parts[1], ...parts[2],
      ]);
      geo.setAttribute("position", new THREE.BufferAttribute(verts, 3));
      geo.computeVertexNormals();
    }
    return geo;
  }, [kind]);

  const transparent = opacity < 1.0;

  return (
    <mesh matrixAutoUpdate={false} matrix={matrix} geometry={geometry}>
      <meshStandardMaterial
        color={color}
        opacity={opacity}
        transparent={transparent}
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
