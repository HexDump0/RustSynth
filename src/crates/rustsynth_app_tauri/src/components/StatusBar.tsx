type StatusBarParams = {
  showConsole: boolean;
  warningsCount: number;
  status: string;
  objectCount: number;
  fileName: string;
  onToggleConsole: () => void;
};

export function StatusBar(params: StatusBarParams) {
  return (
    <div id="tour-status" className="bg-ctp-mantle px-6 py-2 w-full flex justify-between shrink-0">
      <div className="flex gap-7">
        <button
          onClick={params.onToggleConsole}
          className={`text-xs uppercase font-medium transition-colors cursor-pointer ${params.warningsCount > 0 ? "text-ctp-red hover:text-ctp-red" : "text-ctp-subtext1 hover:text-ctp-mauve"}`}
        >
          {params.showConsole ? "HIDE CONSOLE" : "SHOW CONSOLE"} {params.warningsCount > 0 ? ` (${params.warningsCount})` : ""}
        </button>
        <div className="h-4 w-px bg-ctp-surface1" />

        <p className="text-xs font-medium uppercase">{params.status}</p>
        <div className="h-4 w-px bg-ctp-surface1" />
        <p className="text-xs text-ctp-subtext0 font-medium uppercase">{params.objectCount} OBJECTS</p>
      </div>
      <p className="text-xs text-ctp-subtext0 font-medium uppercase">{params.fileName}</p>
    </div>
  );
}