import { useMemo, useRef } from "react";

type HighlightKind = "plain" | "comment" | "preprocessor" | "keyword" | "operator" | "number" | "bracket" | "symbol";

type HighlightToken = {
  text: string;
  kind: HighlightKind;
};

const KEYWORDS = new Set(["rule", "set"]);

const OPERATORS = new Set([
  "c",
  "reflect",
  "color",
  "blend",
  "a",
  "alpha",
  "matrix",
  "h",
  "hue",
  "sat",
  "b",
  "brightness",
  "v",
  "x",
  "y",
  "z",
  "rx",
  "ry",
  "rz",
  "s",
  "fx",
  "fy",
  "fz",
  "maxdepth",
  "weight",
  "md",
  "w",
]);

const BRACKETS = new Set(["{", "}", "[", "]", "(", ")", "*", ">"]);

function readWhile(source: string, start: number, predicate: (ch: string) => boolean): number {
  let i = start;
  while (i < source.length && predicate(source[i])) i += 1;
  return i;
}

function highlightEisenScript(source: string): HighlightToken[] {
  const out: HighlightToken[] = [];
  let i = 0;

  while (i < source.length) {
    const ch = source[i];

    if (ch === " " || ch === "\t" || ch === "\n" || ch === "\r") {
      const end = readWhile(source, i, c => c === " " || c === "\t" || c === "\n" || c === "\r");
      out.push({ text: source.slice(i, end), kind: "plain" });
      i = end;
      continue;
    }

    if ((i === 0 || source[i - 1] === "\n") && ch === "#") {
      const end = readWhile(source, i, c => c !== "\n");
      out.push({ text: source.slice(i, end), kind: "preprocessor" });
      i = end;
      continue;
    }

    if (ch === "/" && source[i + 1] === "/") {
      const end = readWhile(source, i, c => c !== "\n");
      out.push({ text: source.slice(i, end), kind: "comment" });
      i = end;
      continue;
    }

    if (ch === "/" && source[i + 1] === "*") {
      const close = source.indexOf("*/", i + 2);
      const end = close >= 0 ? close + 2 : source.length;
      out.push({ text: source.slice(i, end), kind: "comment" });
      i = end;
      continue;
    }

    const hexColor = source.slice(i).match(/^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b/);
    if (hexColor) {
      out.push({ text: hexColor[0], kind: "number" });
      i += hexColor[0].length;
      continue;
    }

    if (BRACKETS.has(ch)) {
      out.push({ text: ch, kind: "bracket" });
      i += 1;
      continue;
    }

    const number = source.slice(i).match(/^[+-]?(?:\d+\/\d+|\d*\.\d+|\d+)(?:[eE][+-]?\d+)?/);
    if (number) {
      out.push({ text: number[0], kind: "number" });
      i += number[0].length;
      continue;
    }

    const word = source.slice(i).match(/^[A-Za-z_][A-Za-z0-9_:]*/);
    if (word) {
      const text = word[0];
      const lower = text.toLowerCase();

      if (KEYWORDS.has(lower)) {
        out.push({ text, kind: "keyword" });
      } else if (OPERATORS.has(lower)) {
        out.push({ text, kind: "operator" });
      } else {
        out.push({ text, kind: "symbol" });
      }

      i += text.length;
      continue;
    }

    out.push({ text: ch, kind: "plain" });
    i += 1;
  }

  return out;
}

function tokenClass(kind: HighlightKind): string {
  switch (kind) {
    case "comment":
      return "text-ctp-overlay0";
    case "preprocessor":
      return "text-ctp-peach";
    case "keyword":
      return "text-ctp-mauve";
    case "operator":
      return "text-ctp-teal";
    case "number":
      return "text-ctp-yellow";
    case "bracket":
      return "text-ctp-sky";
    case "symbol":
      return "text-ctp-text";
    case "plain":
    default:
      return "text-ctp-text";
  }
}

type EditorParams = {
  fileName: string;
  source: string;
  onSourceChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  showConsole: boolean;
  warnings: string[];
  consoleHeight: number;
  onConsoleResizeStart: (startY: number, startHeight: number) => void;
};

export function Editor(params: EditorParams) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const gutterRef = useRef<HTMLDivElement>(null);
  const highlightContentRef = useRef<HTMLPreElement>(null);
  const lineCount = params.source.split("\n").length;
  const highlighted = useMemo(() => highlightEisenScript(params.source), [params.source]);

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
        <div className="relative flex-1 min-h-0 bg-ctp-crust overflow-hidden">
          <div className="pointer-events-none absolute inset-0 overflow-hidden">
            <pre
              ref={highlightContentRef}
              aria-hidden
              className="m-0 py-2.5 text-sm leading-relaxed whitespace-pre font-mono text-ctp-text pl-2"
            >
              {highlighted.map((token, index) => (
                <span key={index} className={tokenClass(token.kind)}>
                  {token.text}
                </span>
              ))}
            </pre>
          </div>
          <textarea
            ref={textareaRef}
            value={params.source}
            onChange={e => params.onSourceChange(e.target.value)}
            onKeyDown={params.onKeyDown}
            onScroll={e => {
              const { scrollTop, scrollLeft } = e.currentTarget;
              if (gutterRef.current) {
                gutterRef.current.scrollTop = scrollTop;
              }
              if (highlightContentRef.current) {
                highlightContentRef.current.style.transform = `translate(${-scrollLeft}px, ${-scrollTop}px)`;
              }
            }}
            spellCheck={false}
            className="absolute inset-0 bg-transparent py-2.5 text-sm leading-relaxed overflow-y-auto overflow-x-auto whitespace-pre font-mono flex-1 min-h-0 resize-none outline-none text-transparent caret-ctp-text pl-2"
          />
        </div>
      </div>
      {params.showConsole && (
        <>
          <div
            role="separator"
            aria-orientation="horizontal"
            aria-label="Resize console"
            className="h-1 shrink-0 cursor-row-resize bg-ctp-surface hover:bg-ctp-surface0 transition-colors"
            onPointerDown={e => {
              e.preventDefault();
              params.onConsoleResizeStart(e.clientY, params.consoleHeight);
            }}
          />
          <div
            className="overflow-y-auto bg-ctp-crust px-3 py-2 text-sm text-ctp-subtext0 whitespace-pre-wrap font-mono shrink-0 "
            style={{ height: params.consoleHeight }}
          >
            {params.warnings.length === 0
              ? "No warnings."
              : params.warnings.map((w, i) => <div key={i}>{w}</div>)}
          </div>
        </>
      )}
    </div>
  );
}