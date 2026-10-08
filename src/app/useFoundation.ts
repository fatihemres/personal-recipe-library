import { useCallback, useEffect, useRef, useState } from 'react';
import { normalizeError, type FoundationClient } from '../shared/api/client';
import type { AppError, Bootstrap, Theme } from '../shared/contracts/foundation';

export function useFoundation(client: FoundationClient) {
  const [data, setData] = useState<Bootstrap>();
  const [error, setError] = useState<AppError>();
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const savingRef = useRef(false);
  const generation = useRef(0);

  const load = useCallback(async () => {
    const current = ++generation.current;
    setLoading(true);
    setError(undefined);
    try {
      const result = await client.bootstrap();
      if (current === generation.current) setData(result);
    } catch (err) {
      if (current === generation.current) setError(normalizeError(err));
    } finally {
      if (current === generation.current) setLoading(false);
    }
  }, [client]);

  useEffect(() => {
    const lifecycle = generation;
    void load();
    return () => { lifecycle.current++; };
  }, [load]);

  async function saveTheme(theme: Theme) {
    if (!data || savingRef.current) return;
    savingRef.current = true;
    setSaving(true);
    setError(undefined);
    try {
      const preferences = await client.savePreferences({ ...data.preferences, theme });
      setData((previous) => previous ? { ...previous, preferences } : previous);
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      savingRef.current = false;
      setSaving(false);
    }
  }

  return { data, error, loading, saving, load, saveTheme };
}
