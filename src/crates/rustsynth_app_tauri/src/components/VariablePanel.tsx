import { useCallback } from "react";
import type { GuiParam } from "../types";

interface VariablePanelProps {
  params: GuiParam[];
  onChange: (name: string, value: string) => void;
  onChangeComplete: () => void;
}

export function VariablePanel({ params, onChange, onChangeComplete }: VariablePanelProps) {
  return (
    <div className="border-t border-ctp-surface1 px-3 py-2 bg-ctp-mantle max-h-[200px] overflow-y-auto">
      <div className="text-xs text-ctp-subtext0 uppercase tracking-wider mb-1.5">
        Variables ({params.length})
      </div>
      {params.map((param, i) => (
        <ParamRow
          key={i}
          param={param}
          onChange={onChange}
          onChangeComplete={onChangeComplete}
        />
      ))}
    </div>
  );
}

function ParamRow({
  param,
  onChange,
  onChangeComplete,
}: {
  param: GuiParam;
  onChange: (name: string, value: string) => void;
  onChangeComplete: () => void;
}) {
  const info = param.Float || param.Int;
  if (!info) return null;

  const isFloat = !!param.Float;
  const { name, min, max } = info;
  const defaultVal = info.default;

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = isFloat
        ? parseFloat(e.target.value).toFixed(6)
        : `${Math.round(parseFloat(e.target.value))}`;
      onChange(name, val);
    },
    [name, isFloat, onChange],
  );

  const handleMouseUp = useCallback(() => {
    onChangeComplete();
  }, [onChangeComplete]);

  return (
    <div className="flex items-center gap-2 mb-1">
      <label className="min-w-[100px] text-ctp-subtext0 text-xs">{name}</label>
      <input
        type="range"
        min={min}
        max={max}
        step={isFloat ? (max - min) / 200 : 1}
        defaultValue={defaultVal}
        onChange={handleChange}
        onMouseUp={handleMouseUp}
        className="flex-1 accent-ctp-mauve"
      />
      <span className="min-w-[50px] text-right text-xs text-ctp-overlay1">
        {isFloat ? defaultVal.toFixed(2) : defaultVal}
      </span>
    </div>
  );
}
