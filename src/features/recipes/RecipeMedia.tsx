import { useEffect, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { mediaTr as t } from '../../shared/i18n/media';
import { Button } from '../../shared/ui/button';

export function CoverImage({ assetId, variant, client, alt, unavailableText }: {
  assetId?: string | null; variant: 'thumb' | 'preview'; client: RecipeClient; alt?: string; unavailableText?: string;
}) {
  const [failed, setFailed] = useState(false);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => { setFailed(false); }, [assetId]);
  if (!assetId) return null;
  if (failed) return <div className={`cover-unavailable cover-${variant}`}>
    <p role="status">{unavailableText ?? t.unavailable}</p>
    {variant === 'preview' && <Button type="button" variant="outline" onClick={() => {
      setFailed(false); setAttempt(n => n + 1);
    }}>{t.retry}</Button>}
  </div>;
  return <img key={`${assetId}:${attempt}`} className={`recipe-cover cover-${variant}`}
    src={client.coverUrl(assetId, variant)} alt={alt ?? (variant === 'thumb' ? '' : t.cover)}
    loading="lazy" decoding="async" onError={() => setFailed(true)} />;
}

/** One bounded native import; late completion never edits an unmounted component. */
export function useMediaImport(
  pick: RecipeClient['pickCover'], onChange: (id: string | null) => void, onBusy: (busy: boolean) => void,
) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const mounted = useRef(true), pending = useRef(false);
  const latest = useRef({ pick, onChange, onBusy });
  latest.current = { pick, onChange, onBusy };
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  async function choose() {
    if (pending.current) return;
    pending.current = true; setBusy(true); setError('');
    const report = latest.current.onBusy;
    report(true);
    try {
      const photo = await latest.current.pick();
      if (mounted.current && photo) latest.current.onChange(photo.id);
    } catch (e) {
      if (mounted.current) setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      pending.current = false;
      report(false);
      if (mounted.current) setBusy(false);
    }
  }
  return { busy, error, choose, clearError: () => setError('') };
}
