let ready: Promise<any> | null = null;
export function loadEngine() {
  if (!ready) {
    ready = (async () => {
      const url = "/wasm/strucal_wasm.js";
      const mod = await import(/* webpackIgnore: true */ url);
      await mod.default("/wasm/strucal_wasm_bg.wasm");
      return mod;
    })();
  }
  return ready;
}
