declare global {
  interface Window {
    __TAURI__: {
      core: {
        invoke: (name: string, args?: Record<string, unknown>) => Promise<any>;
      };
    };
  }
}
export const invoke = (name: string, args?: Record<string, unknown>) =>
  window.__TAURI__.core.invoke(name, args);
