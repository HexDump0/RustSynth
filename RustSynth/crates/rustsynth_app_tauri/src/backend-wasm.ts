import type { Backend } from "./backend";
import type { BuildConfig } from "./types";

export async function createWasmBackend(): Promise<Backend> {
  // @ts-ignore — resolved by vite alias to the wasm-pack pkg directory
  const wasm = await import("rustsynth-wasm");
  // wasm-pack --target web requires explicit init before calling exports
  if (typeof wasm.default === "function") {
    await wasm.default();
  }

  return {
    async runScript(source, config) {
      const configJson = JSON.stringify(config);
      return wasm.run_script(source, configJson);
    },
    async openFile() {
      const input = document.createElement("input");
      input.type = "file";
      input.accept = ".es,.eisenscript,.txt";
      return new Promise((resolve) => {
        input.onchange = async () => {
          const file = input.files?.[0];
          if (!file) { resolve(null); return; }
          const content = await file.text();
          resolve({ path: file.name, content });
        };
        input.oncancel = () => resolve(null);
        input.click();
      });
    },
    async saveFile(content, currentPath) {
      const name = currentPath?.split("/").pop() ?? "script.es";
      const blob = new Blob([content], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = name;
      a.click();
      URL.revokeObjectURL(url);
      return name;
    },
    async exportObj(source, config) {
      const configJson = JSON.stringify(config);
      const result = wasm.export_obj(source, configJson);
      const blob = new Blob([result.obj], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "scene.obj";
      a.click();
      URL.revokeObjectURL(url);
      if (result.mtl) {
        const mtlBlob = new Blob([result.mtl], { type: "text/plain" });
        const mtlUrl = URL.createObjectURL(mtlBlob);
        const mtlA = document.createElement("a");
        mtlA.href = mtlUrl;
        mtlA.download = "scene.mtl";
        mtlA.click();
        URL.revokeObjectURL(mtlUrl);
      }
      return "scene.obj";
    },
    async exportTemplate(source, config, templateXml) {
      const configJson = JSON.stringify(config);
      const text = wasm.export_template(source, configJson, templateXml ?? "");
      const blob = new Blob([text], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "export.txt";
      a.click();
      URL.revokeObjectURL(url);
      return "export.txt";
    },
  };
}
