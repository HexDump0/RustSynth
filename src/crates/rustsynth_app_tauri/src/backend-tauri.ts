import { invoke } from "@tauri-apps/api/core";
import type { Backend } from "./backend";
import type { PipelineResult, BuildConfig } from "./types";

export const tauriBackend: Backend = {
  async runScript(source, config) {
    return invoke<PipelineResult>("run_script", { source, config });
  },
  async openFile() {
    try {
      return await invoke<{ path: string; content: string }>("open_file_dialog");
    } catch (e) {
      if (e === "cancelled") return null;
      throw e;
    }
  },
  async saveFile(content, currentPath) {
    try {
      return await invoke<string>("save_file_dialog", { content, currentPath });
    } catch (e) {
      if (e === "cancelled") return null;
      throw e;
    }
  },
  async exportObj(source, config) {
    try {
      return await invoke<string>("export_obj", { source, config });
    } catch (e) {
      if (e === "cancelled") return null;
      throw e;
    }
  },
  async exportTemplate(source, config) {
    try {
      return await invoke<string>("export_template", {
        source,
        config,
        templatePath: null,
      });
    } catch (e) {
      if (e === "cancelled") return null;
      throw e;
    }
  },
};
