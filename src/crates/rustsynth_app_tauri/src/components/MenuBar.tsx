import { useEffect, useRef, useState } from "react";

type MenuBarParams = {
  seed: number;
  maxObjects: number;
  examples: string[];
  selectedExampleLabel: string;
  onSeedChange: (value: number) => void;
  onMaxObjectsChange: (value: number) => void;
  onExampleSelect: (examplePath: string) => void;
  onNewFile: () => void;
  onOpenFile: () => void;
  onSaveFile: () => void;
  onRun: () => void;
  onExportObj: () => void;
  onStartOnboarding: () => void;
};

export function MenuBar(params: MenuBarParams) {
  const [examplesOpen, setExamplesOpen] = useState(false);
  const examplesRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handlePointerDown = (event: MouseEvent) => {
      if (!examplesRef.current) return;
      if (!examplesRef.current.contains(event.target as Node)) {
        setExamplesOpen(false);
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setExamplesOpen(false);
      }
    };

    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleEscape);
    };
  }, []);

  return (
    <div className="h-12 bg-ctp-base px-6 flex items-center justify-between shrink-0">
      <div className="flex items-center gap-6">
        <div className="flex items-center justify-center gap-6">
          <button
            onClick={params.onNewFile}
            className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase hover:text-ctp-mauve transition-colors cursor-pointer"
          >
            NEW
          </button>
          <button
            onClick={params.onOpenFile}
            className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase hover:text-ctp-mauve transition-colors cursor-pointer"
          >
            OPEN
          </button>
          <button
            onClick={params.onSaveFile}
            className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase hover:text-ctp-mauve transition-colors cursor-pointer"
          >
            SAVE
          </button>
        </div>
        <div className="h-6 w-px bg-ctp-surface1" />
        <div id="tour-examples" ref={examplesRef} className="relative">
          <button
            onClick={() => setExamplesOpen(v => !v)}
            className="bg-ctp-crust px-3 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase hover:text-ctp-mauve transition-colors cursor-pointer min-w-40 flex items-center justify-between gap-2"
            type="button"
            title={params.selectedExampleLabel}
          >
            <span className="truncate max-w-48 text-left">{params.selectedExampleLabel}</span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="12"
              height="12"
              viewBox="0 0 16 16"
              className={`transition-transform ${examplesOpen ? "rotate-180" : "rotate-0"}`}
            >
              <path fill="currentColor" d="M3.2 5.5L8 10.3l4.8-4.8l1 1L8 12.3L2.2 6.5z" />
            </svg>
          </button>
          {examplesOpen && (
            <div className="absolute left-0 top-full mt-1 z-20 w-72 max-h-72 overflow-y-auto border border-ctp-surface1 bg-ctp-crust shadow-xl">
              {params.examples.map(example => (
                <button
                  key={example}
                  type="button"
                  onClick={() => {
                    params.onExampleSelect(example);
                    setExamplesOpen(false);
                  }}
                  className="w-full text-left px-3 py-2 text-sm font-mono text-ctp-subtext1 hover:text-ctp-mauve hover:bg-ctp-surface0 transition-colors"
                  title={example}
                >
                  {example}
                </button>
              ))}
            </div>
          )}
        </div>
        <div className="h-6 w-px bg-ctp-surface1" />
        <div className="flex items-center justify-center gap-6">
          <div className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide uppercase w-24 flex justify-between">
            SEED
            <input
              type="number"
              value={params.seed}
              min={0}
              onChange={e => params.onSeedChange(parseInt(e.target.value, 10) || 0)}
              className="bg-transparent text-ctp-mauve w-12 text-right outline-none font-mono text-sm [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none pr-4"
            />
          </div>
          <div className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide flex justify-between gap-6 min-w-36">
            MAX
            <input
              type="number"
              value={params.maxObjects}
              min={1}
              step={1000}
              onChange={e => params.onMaxObjectsChange(parseInt(e.target.value, 10) || 100000)}
              className="bg-transparent text-ctp-mauve w-16 text-right outline-none font-mono text-sm [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
            />
          </div>
        </div>
      </div>
      <div className="flex items-center justify-center gap-6">
        <button
          id="tour-run"
          onClick={params.onRun}
          className="bg-ctp-mauve px-4 py-1 text-sm font-mono tracking-wide flex justify-between gap-2 text-ctp-base items-center hover:opacity-90 transition-opacity cursor-pointer font-semibold"
        >
          RUN
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
            <path
              fill="currentColor"
              d="M3 3.5a1.5 1.5 0 0 1 2.235-1.307l8 4.5a1.5 1.5 0 0 1 0 2.615l-8 4.5A1.5 1.5 0 0 1 3 12.5z"
            />
          </svg>
        </button>
        <button
          onClick={params.onExportObj}
          className="bg-ctp-crust px-4 py-1 text-sm font-mono text-ctp-subtext1 tracking-wide flex justify-between gap-2 items-center hover:text-ctp-mauve transition-colors cursor-pointer"
        >
          EXPORT OBJ
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
            <path
              fill="currentColor"
              d="M11.5 7a4.5 4.5 0 1 0 0 9a4.5 4.5 0 0 0 0-9m2.354 4.854l-2 2a.5.5 0 0 1-.35.147h-.006a.5.5 0 0 1-.348-.144l-.003-.003l-2-2a.5.5 0 0 1 .707-.707L11 12.294V9.001a.5.5 0 0 1 1 0v3.293l1.146-1.147a.5.5 0 0 1 .707.707zM4.25 12H6v1H4.25a3.25 3.25 0 0 1-.22-6.493A4 4 0 0 1 8 3a3.99 3.99 0 0 1 3.857 3h-1.046A2.99 2.99 0 0 0 8 4a3 3 0 0 0-3 3a.5.5 0 0 1-.5.5h-.25a2.25 2.25 0 1 0 0 4.5"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}