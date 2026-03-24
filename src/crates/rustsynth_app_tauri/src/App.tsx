import { useState, useCallback, useEffect, useRef } from "react";
import { getBackend, type Backend } from "./backend";
import type { BuildConfig, GuiParam, Scene } from "./types";
import { Viewport } from "./components/Viewport";
import { InfoBar } from "./components/InfoBar";
import { MenuBar } from "./components/MenuBar";
import { Editor } from "./components/Editor";
import { StatusBar } from "./components/StatusBar";
import { useRustSynthOnboarding } from "./onboarding";

type ScriptCameraInsert = {
  eye: [number, number, number];
  target: [number, number, number];
  up: [number, number, number];
  fov?: number;
};

const EXAMPLES = Object.entries(
  import.meta.glob("./examples/**/*.es", {
    eager: true,
    query: "?raw",
    import: "default",
  }) as Record<string, string>
)
  .map(([path, content]) => {
    const relative = path.replace("./examples/", "");
    return { path: relative, content };
  })
  .sort((a, b) => a.path.localeCompare(b.path));


const DEFAULT_SCRIPT = `set camera_eye [-21.906634 25.703905 64.490958]
set camera_target [13.53033 3.935075 4.414238]
set camera_up [0 1 0]
set camera_fov 45

set maxobjects 16000
10 * { y 1 } 10 * { z 1 }  1 * { a 0.8  sat 0.9  } r1 
set background #fff


rule r1   {
  { x 1  ry 4 } r1
  xbox
}

rule r1   {
{ x 1  ry -4  } r1
xbox
}

rule r1   {
{ x 1  rz -8  s 0.95 } r1
xbox
}

rule r1   {
{ x 1  rz 8  s 0.95   } r1
xbox
}



rule r2 maxdepth 36 {
{ ry 1  ry -13 x  1.2 b 0.99 h 12  } r2 
xbox
}

rule xbox {
  { s 1.1   color #000   } grid
  { b 0.7  color #000    }  box
}

rule xbox {
 { s 1.1   color #000     } grid
 { b 0.7  color #fff      } box
}`;

function App() {
  const [source, setSource] = useState(DEFAULT_SCRIPT);
  const [scene, setScene] = useState<Scene | null>(null);
  const [status, setStatus] = useState("Ready");
  const [objectCount, setObjectCount] = useState(0);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [guiParams, setGuiParams] = useState<GuiParam[]>([]);
  const [showConsole, setShowConsole] = useState(false);
  const [seed, setSeed] = useState(0);
  const [maxObjects, setMaxObjects] = useState(100000);
  const [selectedExampleLabel, setSelectedExampleLabel] = useState("EXAMPLES");
  const [filePath, setFilePath] = useState<string | null>(null);
  const [backend, setBackend] = useState<Backend | null>(null);
  const [editorWidthPct, setEditorWidthPct] = useState(50);
  const [consoleHeight, setConsoleHeight] = useState(128);
  const splitContainerRef = useRef<HTMLDivElement | null>(null);
  const editorContainerRef = useRef<HTMLDivElement | null>(null);
  const isResizingRef = useRef(false);
  const isResizingConsoleRef = useRef(false);
  const consoleResizeStartY = useRef(0);
  const consoleResizeStartHeight = useRef(0);

  const buildConfig = useCallback((): BuildConfig => ({
    max_generations: 1000,
    max_objects: maxObjects,
    min_dim: 0,
    max_dim: 0,
    sync_random: false,
    mode: "BreadthFirst",
    seed,
  }), [seed, maxObjects]);

  const runScript = useCallback(async () => {
    if (!backend) return;
    try {
      setStatus("BUILDING");
      const result = await backend.runScript(source, buildConfig());
      setScene(result.scene);
      setObjectCount(result.scene.objects.length);
      setWarnings(result.warnings);
      setGuiParams(result.gui_params);
      setStatus(
        `READY`
      );
    } catch (e) {
      setStatus(`ERROR: ${e}`);
      setWarnings([`${e}`]);
    }
  }, [backend, source, buildConfig]);

  useEffect(() => {
    getBackend().then(setBackend);
  }, []);

  useEffect(() => {
    if (backend) runScript();
  }, [backend]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const handlePointerMove = (e: PointerEvent) => {
      // Handle horizontal editor/viewport resize
      if (isResizingRef.current) {
        const container = splitContainerRef.current;
        if (container) {
          const rect = container.getBoundingClientRect();
          if (rect.width > 0) {
            const nextPct = ((e.clientX - rect.left) / rect.width) * 100;
            const clamped = Math.max(20, Math.min(80, nextPct));
            setEditorWidthPct(clamped);
          }
        }
      }
      
      // Handle vertical console resize
      if (isResizingConsoleRef.current) {
        const deltaY = consoleResizeStartY.current - e.clientY;
        const newHeight = consoleResizeStartHeight.current + deltaY;
        const clamped = Math.max(32, Math.min(400, newHeight));
        setConsoleHeight(clamped);
      }
    };

    const handlePointerUp = () => {
      if (isResizingRef.current) {
        isResizingRef.current = false;
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      }
      if (isResizingConsoleRef.current) {
        isResizingConsoleRef.current = false;
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      }
    };

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, []);

  const handleParamChange = useCallback((name: string, value: string) => {
    setSource(prev => {
      const prefix = `#define ${name} `;
      return prev
        .split("\n")
        .map(line => {
          if (line.startsWith(prefix)) {
            const rest = line.slice(prefix.length);
            const metaIdx = rest.indexOf(" (");
            const meta = metaIdx >= 0 ? rest.slice(metaIdx) : "";
            return `${prefix}${value}${meta}`;
          }
          return line;
        })
        .join("\n");
    });
  }, []);

  const handleNewFile = useCallback(() => {
    setSource("");
    setFilePath(null);
    setSelectedExampleLabel("EXAMPLES");
  }, []);

  const handleOpenFile = useCallback(async () => {
    if (!backend) return;
    try {
      const result = await backend.openFile();
      if (!result) return;
      setSource(result.content);
      setFilePath(result.path);
      setSelectedExampleLabel("EXAMPLES");
      setStatus(`Opened: ${result.path.split("/").pop()}`);
    } catch (e) {
      setStatus(`Open error: ${e}`);
    }
  }, [backend]);

  const handleSelectExample = useCallback((examplePath: string) => {
    const selected = EXAMPLES.find(example => example.path === examplePath);
    if (!selected) return;
    setSource(selected.content);
    setFilePath(`examples/${selected.path}`);
    setSelectedExampleLabel(selected.path);
  }, []);

  const handleSaveFile = useCallback(async () => {
    if (!backend) return;
    try {
      const path = await backend.saveFile(source, filePath);
      if (!path) return;
      setFilePath(path);
      setStatus(`Saved: ${path.split("/").pop()}`);
    } catch (e) {
      setStatus(`Save error: ${e}`);
    }
  }, [backend, source, filePath]);

  const handleExportObj = useCallback(async () => {
    if (!backend) return;
    try {
      const result = await backend.exportObj(source, buildConfig());
      if (!result) return;
      setStatus(`OBJ exported: ${result}`);
    } catch (e) {
      setStatus(`Export error: ${e}`);
    }
  }, [backend, source, buildConfig]);

  const handleExportTemplate = useCallback(async () => {
    if (!backend) return;
    try {
      const result = await backend.exportTemplate(source, buildConfig());
      if (!result) return;
      setStatus(`Template exported: ${result}`);
    } catch (e) {
      setStatus(`Export error: ${e}`);
    }
  }, [backend, source, buildConfig]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      runScript();
    }
    if (e.key === "Tab" && !e.shiftKey && e.currentTarget instanceof HTMLTextAreaElement) {
      e.preventDefault();
      const ta = e.currentTarget;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      const val = ta.value;
      const next = val.substring(0, start) + "  " + val.substring(end);
      setSource(next);
      requestAnimationFrame(() => {
        ta.selectionStart = ta.selectionEnd = start + 2;
      });
    }
  }, [runScript]);

  const handleInsertCameraToCode = useCallback((camera: ScriptCameraInsert) => {
    setSource(prev => insertOrReplaceCameraBlock(prev, camera));
  }, []);
  const { startOnboarding } = useRustSynthOnboarding();

  const fileName = filePath ? filePath.split("/").pop() : "unsaved";

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-ctp-base text-ctp-text font-sans">
      <InfoBar github="https://github.com/HexDump0/RustSynth" docs="https://github.com/HexDump0/RustSynth/blob/main/DOCS.md"/>

      <MenuBar
        seed={seed}
        maxObjects={maxObjects}
        examples={EXAMPLES.map(example => example.path)}
        selectedExampleLabel={selectedExampleLabel}
        onSeedChange={setSeed}
        onMaxObjectsChange={setMaxObjects}
        onExampleSelect={handleSelectExample}
        onNewFile={handleNewFile}
        onOpenFile={handleOpenFile}
        onSaveFile={handleSaveFile}
        onRun={runScript}
        onExportObj={handleExportObj}
        onStartOnboarding={startOnboarding}
      />

      <div ref={splitContainerRef} className="flex flex-1 min-h-0">
        <div ref={editorContainerRef} className="min-w-0 flex flex-col" style={{ width: `${editorWidthPct}%` }}>
          <Editor
            fileName={fileName}
            source={source}
            onSourceChange={setSource}
            onKeyDown={handleKeyDown}
            showConsole={showConsole}
            warnings={warnings}
            consoleHeight={consoleHeight}
            onConsoleResizeStart={(startY, startHeight) => {
              consoleResizeStartY.current = startY;
              consoleResizeStartHeight.current = startHeight;
              isResizingConsoleRef.current = true;
              document.body.style.cursor = "row-resize";
              document.body.style.userSelect = "none";
            }}
          />
        </div>

        <div
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize editor and viewport"
          className="w-1 shrink-0 cursor-col-resize bg-ctp-mantle hover:bg-ctp-surface0 transition-colors"
          onPointerDown={e => {
            e.preventDefault();
            isResizingRef.current = true;
            document.body.style.cursor = "col-resize";
            document.body.style.userSelect = "none";
          }}
        />

        <div className="bg-black flex-1 min-w-0">
          <Viewport scene={scene} onInsertCameraToCode={handleInsertCameraToCode} />
        </div>
      </div>

      <StatusBar
        showConsole={showConsole}
        warningsCount={warnings.length}
        status={status}
        objectCount={objectCount}
        fileName={fileName}
        onToggleConsole={() => setShowConsole(v => !v)}
      />

    </div>
  );
}

function fmtNum(v: number): string {
  if (!Number.isFinite(v)) return "0";
  const rounded = Math.round(v * 1_000_000) / 1_000_000;
  const text = rounded.toString();
  return text === "-0" ? "0" : text;
}

function fmtVec3(v: [number, number, number]): string {
  return `[${fmtNum(v[0])} ${fmtNum(v[1])} ${fmtNum(v[2])}]`;
}

function insertOrReplaceCameraBlock(source: string, camera: ScriptCameraInsert): string {
  const lines = source.split("\n");
  const filtered = lines.filter(line => {
    const trimmed = line.trim().toLowerCase();
    return !(
      trimmed.startsWith("set camera ") ||
      trimmed.startsWith("set camera_eye ") ||
      trimmed.startsWith("set camera_target ") ||
      trimmed.startsWith("set camera_up ") ||
      trimmed.startsWith("set camera_fov ")
    );
  });

  const block: string[] = [
    `set camera_eye ${fmtVec3(camera.eye)}`,
    `set camera_target ${fmtVec3(camera.target)}`,
    `set camera_up ${fmtVec3(camera.up)}`,
  ];
  if (typeof camera.fov === "number" && Number.isFinite(camera.fov)) {
    block.push(`set camera_fov ${fmtNum(camera.fov)}`);
  }
  block.push("");

  return [...block, ...filtered].join("\n");
}

export default App;
