import { useState, useCallback, useEffect } from "react";
import { getBackend, type Backend } from "./backend";
import type { BuildConfig, GuiParam, Scene } from "./types";
import { Viewport } from "./components/Viewport";
import { InfoBar } from "./components/InfoBar";
import { MenuBar } from "./components/MenuBar";
import { Editor } from "./components/Editor";
import { StatusBar } from "./components/StatusBar";


const DEFAULT_SCRIPT = `set background #111
set maxdepth 200

r0

rule r0 {
  3 * { rz 120 } R1
  3 * { rz 120 } R2
}

rule R1 {
  { x 1.3 rx 1.57 rz 6 ry 3 s 0.99 hue 0.4 sat 0.99 } R1
  { s 4 } sphere
}

rule R2 {
  { x -1.3 rz 6 ry 3 s 0.99 hue 0.4 sat 0.99 } R2
  { s 4 } box
}
`;

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
  const [filePath, setFilePath] = useState<string | null>(null);
  const [backend, setBackend] = useState<Backend | null>(null);

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
    setSource(DEFAULT_SCRIPT);
    setFilePath(null);
    setStatus("New file");
  }, []);

  const handleOpenFile = useCallback(async () => {
    if (!backend) return;
    try {
      const result = await backend.openFile();
      if (!result) return;
      setSource(result.content);
      setFilePath(result.path);
      setStatus(`Opened: ${result.path.split("/").pop()}`);
    } catch (e) {
      setStatus(`Open error: ${e}`);
    }
  }, [backend]);

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

  const fileName = filePath ? filePath.split("/").pop() : "unsaved";

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-ctp-base text-ctp-text font-sans">
      <InfoBar github="https://github.com/HexDump0/RustSynth" docs="https://github.com/HexDump0/RustSynth/blob/main/DOCS.md"/>

      <MenuBar
        seed={seed}
        maxObjects={maxObjects}
        onSeedChange={setSeed}
        onMaxObjectsChange={setMaxObjects}
        onNewFile={handleNewFile}
        onOpenFile={handleOpenFile}
        onSaveFile={handleSaveFile}
        onRun={runScript}
        onExportObj={handleExportObj}
      />

      <div className="grid grid-cols-2 flex-1 min-h-0">
        <Editor
          fileName={fileName}
          source={source}
          onSourceChange={setSource}
          onKeyDown={handleKeyDown}
          showConsole={showConsole}
          warnings={warnings}
        />
        <div className="bg-black flex-1">
          <Viewport scene={scene} />
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

export default App;
