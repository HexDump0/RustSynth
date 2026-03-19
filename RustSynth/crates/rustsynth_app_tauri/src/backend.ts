import type { PipelineResult, BuildConfig } from "./types";

export interface Backend {
  runScript(source: string, config: BuildConfig): Promise<PipelineResult>;
  openFile(): Promise<{ path: string; content: string } | null>;
  saveFile(content: string, currentPath: string | null): Promise<string | null>;
  exportObj(source: string, config: BuildConfig): Promise<string | null>;
  exportTemplate(source: string, config: BuildConfig, templateXml?: string): Promise<string | null>;
}

let _backend: Backend | null = null;

export async function getBackend(): Promise<Backend> {
  if (_backend) return _backend;

  // Detect Tauri environment
  if ("__TAURI__" in window || "__TAURI_INTERNALS__" in window) {
    const mod = await import("./backend-tauri");
    _backend = mod.tauriBackend;
  } else {
    const mod = await import("./backend-wasm");
    _backend = await mod.createWasmBackend();
  }
  return _backend;
}
