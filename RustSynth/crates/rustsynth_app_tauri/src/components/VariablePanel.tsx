import { useCallback } from "react";
import type { GuiParam } from "../types";

interface VariablePanelProps {
  params: GuiParam[];
  onChange: (name: string, value: string) => void;
  onChangeComplete: () => void;
}

export function VariablePanel({ params, onChange, onChangeComplete }: VariablePanelProps) {
  return (
    <div className="var-panel">
      <div className="var-header">Variables ({params.length})</div>
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
    <div className="var-row">
      <label>{name}</label>
      <input
        type="range"
        min={min}
        max={max}
        step={isFloat ? (max - min) / 200 : 1}
        defaultValue={defaultVal}
        onChange={handleChange}
        onMouseUp={handleMouseUp}
      />
      <span className="var-value">
        {isFloat ? defaultVal.toFixed(2) : defaultVal}
      </span>
    </div>
  );
}
