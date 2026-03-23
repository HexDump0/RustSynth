import { useRef } from "react";

type EditorParams = {
  fileName: string;
  source: string;
  onSourceChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  showConsole: boolean;
  warnings: string[];
};

export function Editor(params: EditorParams) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const gutterRef = useRef<HTMLDivElement>(null);
  const lineCount = params.source.split("\n").length;

  return (
    <div className="flex flex-col h-full min-h-0">
      <div className="w-full h-8 bg-ctp-mantle flex justify-between items-center px-6">
        <p className="text-ctp-mauve uppercase text-xs font-semibold">{params.fileName}</p>
        <p className="text-ctp-overlay1 text-xs">CTRL+ENTER</p>
      </div>
      <div className="flex flex-1 min-h-0 bg-ctp-crust overflow-hidden">
        <div
          ref={gutterRef}
          className="w-10 shrink-0 overflow-hidden text-ctp-overlay0 text-right text-sm leading-relaxed font-mono select-none py-2.5 px-3"
        >
          {Array.from({ length: lineCount }, (_, i) => (
            <div key={i + 1}>{i + 1}</div>
          ))}
        </div>
        <textarea
          ref={textareaRef}
          value={params.source}
          onChange={e => params.onSourceChange(e.target.value)}
          onKeyDown={params.onKeyDown}
          onScroll={e => {
            if (gutterRef.current) {
              gutterRef.current.scrollTop = e.currentTarget.scrollTop;
            }
          }}
          spellCheck={false}
          className="bg-ctp-crust py-2.5 text-sm leading-relaxed overflow-y-auto overflow-x-auto whitespace-pre font-mono flex-1 min-h-0 resize-none outline-none text-ctp-text pl-2"
        />
      </div>
      {params.showConsole && (
        <div className="border-t border-ctp-surface1 h-32 overflow-y-auto bg-ctp-crust px-3 py-2 text-sm text-ctp-subtext0 whitespace-pre-wrap font-mono shrink-0">
          {params.warnings.length === 0
            ? "No warnings."
            : params.warnings.map((w, i) => <div key={i}>{w}</div>)}
        </div>
      )}
    </div>
  );
}