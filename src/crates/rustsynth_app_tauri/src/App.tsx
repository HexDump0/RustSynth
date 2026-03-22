import { useState, useCallback, useRef, useEffect } from "react";
import { getBackend, type Backend } from "./backend";
import type { BuildConfig, GuiParam, Scene } from "./types";
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
  const [backend, setBackend] = useState<Backend | null>(null);
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
    if (!backend) return;
    try {
      setStatus("Building...");
      const result = await backend.runScript(source, buildConfig());
      setScene(result.scene);
      setObjectCount(result.scene.objects.length);
      setWarnings(result.warnings);
      setGuiParams(result.gui_params);
      const warnCount = result.warnings.length;
      setStatus(
        `READY`
      );
    } catch (e) {
      setStatus(`Error: ${e}`);
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

  const fileName = filePath ? filePath.split("/").pop() : "unsaved";

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-ctp-base text-ctp-text font-sans">
      <div className="bg-ctp-crust py-4 px-6 flex justify-between items-center shrink-0">
        <div className="flex gap-2 items-end">
          <h1 className="font-bold text-2xl text-ctp-mauve uppercase">RustSynth</h1>
          <span className="text-xs text-ctp-subtext0 font-mono pb-1">v0.1.0</span>
        </div>
        <div className="flex justify-center items-center gap-8">
          <a
            href="https://github.com/HexDump0/RustSynth"
            target="_blank"
            rel="noreferrer"
            className="flex gap-2 font-bold justify-center items-center text-ctp-text hover:text-ctp-mauve transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16">
              <path fill="currentColor" fillRule="evenodd"
                d="M7.976 0A7.977 7.977 0 0 0 0 7.976c0 3.523 2.3 6.507 5.431 7.584c.392.049.538-.196.538-.392v-1.37c-2.201.49-2.69-1.076-2.69-1.076c-.343-.93-.881-1.175-.881-1.175c-.734-.489.048-.489.048-.489c.783.049 1.224.832 1.224.832c.734 1.223 1.859.88 2.3.685c.048-.538.293-.88.489-1.076c-1.762-.196-3.621-.881-3.621-3.964c0-.88.293-1.566.832-2.153c-.05-.147-.343-.978.098-2.055c0 0 .685-.195 2.201.832c.636-.196 1.322-.245 2.007-.245s1.37.098 2.006.245c1.517-1.027 2.202-.832 2.202-.832c.44 1.077.146 1.908.097 2.104a3.16 3.16 0 0 1 .832 2.153c0 3.083-1.86 3.719-3.62 3.915c.293.244.538.733.538 1.467v2.202c0 .196.146.44.538.392A7.98 7.98 0 0 0 16 7.976C15.951 3.572 12.38 0 7.976 0"
                clipRule="evenodd" />
            </svg>
            GITHUB
          </a>
          <a
            href="https://github.com/HexDump0/RustSynth"
            target="_blank"
            rel="noreferrer"
            className="flex gap-1 font-bold justify-center items-center text-ctp-text hover:text-ctp-mauve transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16">
              <path fill="currentColor"
                d="M3 3a2 2 0 0 1 2-2h3.586a1.5 1.5 0 0 1 1.06.44l2.915 2.914A1.5 1.5 0 0 1 13 5.414V13a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zm2-1a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V6H9.5A1.5 1.5 0 0 1 8 4.5V2zm4.5 3h2.293L9 2.207V4.5a.5.5 0 0 0 .5.5m-4 3a.5.5 0 0 0 0 1h5a.5.5 0 0 0 0-1zM5 10.5a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m.5 1.5a.5.5 0 0 0 0 1h5a.5.5 0 0 0 0-1z" />
            </svg>
            DOCS
          </a>
        </div>
      </div>

      <div className="h-12 bg-ctp-base px-6 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-6">
          <div className="flex items-center justify-center gap-6">
            <button
              onClick={handleNewFile}
              className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase hover:text-ctp-mauve transition-colors cursor-pointer"
            >
              NEW
            </button>
            <button
              onClick={handleOpenFile}
              className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase hover:text-ctp-mauve transition-colors cursor-pointer"
            >
              OPEN
            </button>
            <button
            onClick={handleSaveFile}
            className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase hover:text-ctp-mauve transition-colors cursor-pointer"
          >
            SAVE
          </button>
          </div>
          <div className="h-6 w-px bg-ctp-surface1" />
          <div className="flex items-center justify-center gap-6">
            <div className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase w-24 flex justify-between">
              SEED
              <input
                type="number"
                value={seed}
                min={0}
                onChange={e => setSeed(parseInt(e.target.value) || 0)}
                className="bg-transparent text-ctp-mauve w-12 text-right outline-none font-mono text-sm [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none pr-4"
              />
            </div>
            <div className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide flex justify-between gap-6 min-w-36">
              MAX
              <input
                type="number"
                value={maxObjects}
                min={1}
                step={1000}
                onChange={e => setMaxObjects(parseInt(e.target.value) || 100000)}
                className="bg-transparent text-ctp-mauve w-16 text-right outline-none font-mono text-sm [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
              />
            </div>
          </div>
        </div>
        <div className="flex items-center justify-center gap-6">
          <button
            onClick={runScript}
            className="bg-ctp-mauve px-4 py-1 text-sm font-mono tracking-wide flex justify-between gap-2 text-ctp-base items-center hover:opacity-90 transition-opacity cursor-pointer font-semibold"
          >
            RUN
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
              <path fill="currentColor"
                d="M3 3.5a1.5 1.5 0 0 1 2.235-1.307l8 4.5a1.5 1.5 0 0 1 0 2.615l-8 4.5A1.5 1.5 0 0 1 3 12.5z" />
            </svg>
          </button>
          <button
            onClick={handleExportObj}
            className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide flex justify-between gap-2 items-center hover:text-ctp-mauve transition-colors cursor-pointer"
          >
            EXPORT OBJ
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
              <path fill="currentColor"
                d="M11.5 7a4.5 4.5 0 1 0 0 9a4.5 4.5 0 0 0 0-9m2.354 4.854l-2 2a.5.5 0 0 1-.35.147h-.006a.5.5 0 0 1-.348-.144l-.003-.003l-2-2a.5.5 0 0 1 .707-.707L11 12.294V9.001a.5.5 0 0 1 1 0v3.293l1.146-1.147a.5.5 0 0 1 .707.707zM4.25 12H6v1H4.25a3.25 3.25 0 0 1-.22-6.493A4 4 0 0 1 8 3a3.99 3.99 0 0 1 3.857 3h-1.046A2.99 2.99 0 0 0 8 4a3 3 0 0 0-3 3a.5.5 0 0 1-.5.5h-.25a2.25 2.25 0 1 0 0 4.5" />
            </svg>
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 flex-1 min-h-0">
        <div className="flex flex-col h-full">
          <div className="w-full h-8 bg-ctp-mantle flex justify-between items-center px-6">
            <p className="text-ctp-mauve uppercase text-xs font-semibold">{fileName}</p>
            <p className="text-ctp-overlay1 text-xs">Ctrl+Enter to run</p>
          </div>
          <textarea
            ref={textareaRef}
            value={source}
            onChange={e => setSource(e.target.value)}
            onKeyDown={handleKeyDown}
            spellCheck={false}
            className="bg-ctp-crust p-2.5 text-sm leading-relaxed overflow-auto whitespace-pre font-mono flex-1 min-h-0 resize-none outline-none text-ctp-text"
          />
          {guiParams.length > 0 && (
            <VariablePanel
              params={guiParams}
              onChange={handleParamChange}
              onChangeComplete={runScript}
            />
          )}
          {showConsole && (
            <div className="border-t border-ctp-surface1 max-h-40 overflow-y-auto bg-ctp-crust px-3 py-2 text-sm text-ctp-subtext0 whitespace-pre-wrap font-mono shrink-0">
              {warnings.length === 0
                ? "No warnings."
                : warnings.map((w, i) => <div key={i}>{w}</div>)}
            </div>
          )}
        </div>

        <div className="bg-black flex-1">
          <Viewport scene={scene} />
        </div>
      </div>

      <div className="bg-ctp-mantle px-6 py-2 w-full flex justify-between shrink-0">
        <div className="flex gap-7">
          <button
            onClick={() => setShowConsole(v => !v)}
            className={`text-xs uppercase font-medium transition-colors cursor-pointer ${warnings.length > 0 ? "text-ctp-red hover:text-ctp-red" : "text-ctp-subtext1 hover:text-ctp-mauve"}`}
          >
            {showConsole ? "HIDE CONSOLE" : "SHOW CONSOLE" } {(warnings.length > 0 ? ` (${warnings.length})` : "")}
          </button>
                    <div className="h-4 w-px bg-ctp-surface1" />

          <p className={`text-xs font-medium uppercase`}>
            {status}
          </p>
          <div className="h-4 w-px bg-ctp-surface1" />
          <p className="text-xs text-ctp-subtext0 font-medium uppercase">{objectCount} OBJECTS</p>
        </div>
        <p className="text-xs text-ctp-subtext0 font-medium uppercase">{fileName}</p>
      </div>
    </div>
  );
}

export default App;
