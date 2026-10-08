import { isTauri, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
let guard: (() => Promise<void>) | undefined;
export function registerDraftExitGuard(callback: () => Promise<void>) {
  guard = callback;
  return () => {
    if (guard === callback) guard = undefined;
  };
}
export async function installNativeExitHandler(
  onError: (error: unknown) => void,
) {
  if (!isTauri()) return () => {};
  let exiting = false;
  return listen('request-exit', () => {
    if (exiting) return;
    exiting = true;
    void (async () => {
      try {
        await guard?.();
        await invoke('finish_exit');
      } catch (error) {
        onError(error);
      } finally {
        exiting = false;
      }
    })();
  });
}
