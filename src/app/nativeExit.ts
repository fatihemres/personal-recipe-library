import { isTauri, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
let guard: (() => Promise<void>) | undefined;
let privateNoteGuard: (() => Promise<void>) | undefined;
export function registerPrivateNoteExitGuard(callback: () => Promise<void>) {
  privateNoteGuard = callback;
  return () => { if (privateNoteGuard === callback) privateNoteGuard = undefined; };
}
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
        await privateNoteGuard?.();
        await invoke('finish_exit');
      } catch (error) {
        onError(error);
      } finally {
        exiting = false;
      }
    })();
  });
}
