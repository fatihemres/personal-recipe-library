import { getVersion } from '@tauri-apps/api/app';
import { useEffect, useRef, useState } from 'react';

// Tauri's package metadata is the only runtime version source. Keep this
// independent of SQLite bootstrap so diagnostics remain visible on failure.
export function useAppVersion() {
  const [version, setVersion] = useState<string | null>(null);
  const request = useRef<Promise<string> | null>(null);
  useEffect(() => {
    let active = true;
    request.current ??= getVersion();
    void request.current.then(
      value => { if (active) setVersion(value); },
      () => { if (active) setVersion(null); },
    );
    return () => { active = false; };
  }, []);
  return version;
}
