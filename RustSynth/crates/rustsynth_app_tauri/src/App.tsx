import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PipelineResult, BuildConfig, GuiParam, Scene } from "./types";
import { Viewport } from "./components/Viewport";
import { VariablePanel } from "./components/VariablePanel";

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
  const [recursionMode, setRecursionMode] = useState<"BreadthFirst" | "DepthFirst">("BreadthFirst");
  const [filePath, setFilePath] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const buildConfig = useCallback((): BuildConfig => ({
    max_generations: 1000,
    max_objects: maxObjects,
    min_dim: 0,
    max_dim: 0,
    sync_random: false,
    mode: recursionMode,
    seed,
  }), [seed, maxObjects, recursionMode]);

  const runScript = useCallback(async () => {
    try {
      setStatus("Building...");
      const result = await invoke<PipelineResult>("run_script", {
        source,
        config: buildConfig(),
      });
      setScene(result.scene);
      setObjectCount(result.scene.objects.length);
      setWarnings(result.warnings);
      setGuiParams(result.gui_params);
      const warnCount = result.warnings.length;
      setStatus(
        `${result.scene.objects.length} objects` +
        (warnCount > 0 ? ` · ${warnCount} warning${warnCount > 1 ? "s" : ""}` : "")
      );
    } catch (e) {
      setStatus(`Error: ${e}`);
      setWarnings([`${e}`]);
    }
  }, [source, buildConfig]);

  // Run on first load
  useEffect(() => {
    runScript();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

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
    try {
      const result = await invoke<{ path: string; content: string }>("open_file_dialog");
      setSource(result.content);
      setFilePath(result.path);
      setStatus(`Opened: ${result.path.split("/").pop()}`);
    } catch (e) {
      if (e !== "cancelled") setStatus(`Open error: ${e}`);
    }
  }, []);

  const handleSaveFile = useCallback(async () => {
    try {
      const path = await invoke<string>("save_file_dialog", {
        content: source,
        currentPath: filePath,
      });
      setFilePath(path);
      setStatus(`Saved: ${path.split("/").pop()}`);
    } catch (e) {
      if (e !== "cancelled") setStatus(`Save error: ${e}`);
    }
  }, [source, filePath]);

  const handleExportObj = useCallback(async () => {
    try {
      const result = await invoke<string>("export_obj", {
        source,
        config: buildConfig(),
      });
      setStatus(`OBJ exported: ${result}`);
    } catch (e) {
      if (e !== "cancelled") setStatus(`Export error: ${e}`);
    }
  }, [source, buildConfig]);

  const handleExportTemplate = useCallback(async () => {
    try {
      const result = await invoke<string>("export_template", {
        source,
        config: buildConfig(),
        templatePath: null,
      });
      setStatus(`Template exported: ${result}`);
    } catch (e) {
      if (e !== "cancelled") setStatus(`Export error: ${e}`);
    }
  }, [source, buildConfig]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    // Ctrl/Cmd+Enter to run
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      runScript();
    }
    // Tab inserts 2 spaces
    if (e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      const ta = textareaRef.current;
      if (ta) {
        const start = ta.selectionStart;
        const end = ta.selectionEnd;
        const val = ta.value;
        ta.value = val.substring(0, start) + "  " + val.substring(end);
        ta.selectionStart = ta.selectionEnd = start + 2;
        setSource(ta.value);
      }
    }
  }, [runScript]);

  return (
    <div className="app-container">
      {/* Toolbar */}
      <div className="toolbar">
        <button onClick={handleNewFile}>New</button>
        <button onClick={handleOpenFile}>Open</button>
        <button onClick={handleSaveFile}>Save</button>
        <div style={{ width: 1, height: 20, background: "var(--border)" }} />
        <button className="primary" onClick={runScript}>
          ▶ Run
        </button>
        <div style={{ width: 1, height: 20, background: "var(--border)" }} />
        <label>Seed:</label>
        <input
          type="number"
          value={seed}
          min={0}
          onChange={e => setSeed(parseInt(e.target.value) || 0)}
        />
        <label>Max:</label>
        <input
          type="number"
          value={maxObjects}
          min={1}
          step={1000}
          onChange={e => setMaxObjects(parseInt(e.target.value) || 100000)}
        />
        <select
          value={recursionMode}
          onChange={e => setRecursionMode(e.target.value as "BreadthFirst" | "DepthFirst")}
        >
          <option value="BreadthFirst">BFS</option>
          <option value="DepthFirst">DFS</option>
        </select>
        <div style={{ flex: 1 }} />
        <button onClick={handleExportObj}>Export OBJ</button>
        <button onClick={handleExportTemplate}>Export Template</button>
        <button onClick={() => setShowConsole(v => !v)}>
          Console {warnings.length > 0 ? `(${warnings.length})` : ""}
        </button>
      </div>

      {/* Main content */}
      <div className="main-panel">
        {/* Editor */}
        <div className="editor-panel">
          <div className="panel-header">
            <span>Editor</span>
            <span style={{ fontSize: 10, color: "var(--text-muted)" }}>
              Ctrl+Enter to run
            </span>
          </div>
          <textarea
            ref={textareaRef}
            className="editor-textarea"
            value={source}
            onChange={e => setSource(e.target.value)}
            onKeyDown={handleKeyDown}
            spellCheck={false}
          />
          {guiParams.length > 0 && (
            <VariablePanel
              params={guiParams}
              onChange={handleParamChange}
              onChangeComplete={runScript}
            />
          )}
          {showConsole && (
            <div className="console-panel">
              {warnings.length === 0
                ? "No warnings."
                : warnings.map((w, i) => <div key={i}>{w}</div>)}
            </div>
          )}
        </div>

        {/* Viewport */}
        <div className="viewport-panel">
          <div className="panel-header">
            <span>Viewport</span>
            <span style={{ fontSize: 10, color: "var(--text-muted)" }}>
              {objectCount} objects
            </span>
          </div>
          <div className="viewport-canvas">
            <Viewport scene={scene} />
          </div>
        </div>
      </div>

      {/* Status bar */}
      <div className="status-bar">
        <div className="status-left">
          <span className={status.startsWith("Error") ? "status-error" : ""}>
            {status}
          </span>
        </div>
        <span>{filePath ? filePath.split("/").pop() : "unsaved"}</span>
      </div>
    </div>
  );
}

export default App;
