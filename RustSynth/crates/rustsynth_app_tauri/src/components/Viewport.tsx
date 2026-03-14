import { useMemo, useRef, useEffect, memo } from "react";
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
      gl={{ antialias: true, powerPreference: "high-performance" }}
      frameloop="demand"
    >
      <color attach="background" args={[bgColor.r, bgColor.g, bgColor.b]} />
      <ambientLight intensity={0.4} />
      <directionalLight position={[5, 8, 5]} intensity={0.8} />
      <directionalLight position={[-3, 2, -4]} intensity={0.3} />

      {scene && <OptimizedSceneObjects objects={scene.objects} />}

      <OrbitControls makeDefault enableDamping dampingFactor={0.1} />
      <GizmoHelper alignment="bottom-right" margin={[60, 60]}>
        <GizmoViewport labelColor="white" axisHeadScale={0.8} />
      </GizmoHelper>
    </Canvas>
  );
}

const OptimizedSceneObjects = memo(function OptimizedSceneObjects({ objects }: { objects: SceneObject[] }) {
  const { instancedGroups, nonInstanced } = useMemo(() => {
    const groups = new Map<string, SceneObject[]>();
    const nonInstanced: SceneObject[] = [];

    for (const obj of objects) {
      const kind = normalizePrimitiveKind(obj.kind);
      if (kind === "Triangle") {
        nonInstanced.push(obj);
        continue;
      }

      const opacity = obj.alpha * obj.color.a;
      const opacityKey = opacity < 1.0 ? opacity.toFixed(3) : "1";
      const key = `${kind}_${opacityKey}`;

      let list = groups.get(key);
      if (!list) {
        list = [];
        groups.set(key, list);
      }
      list.push(obj);
    }

    return { instancedGroups: groups, nonInstanced };
  }, [objects]);

  return (
    <>
      {Array.from(instancedGroups.entries()).map(([key, objs]) => {
        const [kind, opacityStr] = key.split("_");
        return (
          <InstancedGroup
            key={key}
            kind={kind}
            opacity={parseFloat(opacityStr)}
            objects={objs}
          />
        );
      })}
      {nonInstanced.map((obj, i) => (
        <ScenePrimitive key={i} obj={obj} />
      ))}
    </>
  );
});

function InstancedGroup({
  kind,
  opacity,
  objects,
}: {
  kind: string;
  opacity: number;
  objects: SceneObject[];
}) {
  const meshRef = useRef<THREE.InstancedMesh>(null);

  const transparent = opacity < 1.0;
  const isWireframe = kind === "Grid" || kind === "Mesh" || kind === "Template" || kind === "Unknown";
  const side = kind === "Grid" ? THREE.DoubleSide : THREE.FrontSide;

  useEffect(() => {
    if (!meshRef.current) return;
    const mesh = meshRef.current;
    
    // Use raw Float32Arrays instead of instantiating Matrix4 and Color classes inside the loop
    // THREE.Matrix4 backing array is elements string
    
    for (let i = 0; i < objects.length; i++) {
      const obj = objects[i];
      // Object transform array is already column-major 16-element
      mesh.setMatrixAt(i, new THREE.Matrix4().fromArray(obj.transform));
      mesh.setColorAt(i, new THREE.Color(obj.color.r, obj.color.g, obj.color.b));
    }

    mesh.instanceMatrix.needsUpdate = true;
    if (mesh.instanceColor) {
      mesh.instanceColor.needsUpdate = true;
    }
  }, [objects]);

  // Generate correct geometry purely by JSX children declaration
  return (
    <instancedMesh ref={meshRef} args={[null as any, null as any, objects.length]} matrixAutoUpdate={false}>
      {kind === "Box" && <boxGeometry args={[1, 1, 1]} />}
      {kind === "Sphere" && <sphereGeometry args={[0.5, 24, 16]} />}
      {kind === "Cylinder" && <cylinderGeometry args={[0.5, 0.5, 1, 24]} />}
      {kind === "Line" && <cylinderGeometry args={[0.02, 0.02, 1, 6]} />}
      {kind === "Dot" && <sphereGeometry args={[0.05, 8, 6]} />}
      {kind === "Grid" && <boxGeometry args={[1, 0.01, 1]} />}
      {(kind === "Mesh" || kind === "Template" || kind === "Unknown") && <boxGeometry args={[1, 1, 1]} />}

      <meshStandardMaterial
        opacity={opacity}
        transparent={transparent}
        wireframe={isWireframe}
        side={side}
      />
    </instancedMesh>
  );
}

function ScenePrimitive({ obj }: { obj: SceneObject }) {
  const matrix = useMemo(() => {
    const m = new THREE.Matrix4();
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

  if (kind === "Triangle") {
    return <TrianglePrimitive matrix={matrix} color={color} opacity={opacity} kind={obj.kind} />;
  }

  // Fallback, should rarely be hit since most fallbacks go into InstancedGroup
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
    let vertexStr = "";
    if (typeof kind === "object" && "Triangle" in kind) {
      vertexStr = kind.Triangle;
    }
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
