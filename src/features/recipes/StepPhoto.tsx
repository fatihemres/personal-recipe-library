import { useEffect, useId, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import { mediaTr } from '../../shared/i18n/media';
import { stepPhotosTr as t } from '../../shared/i18n/stepPhotos';
import { Button } from '../../shared/ui/button';
import { CoverImage, useMediaImport } from './RecipeMedia';

function PhotoPreview({ assetId, position, client, onClose }: {
  assetId: string; position: number; client: RecipeClient; onClose: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null), close = useRef<HTMLButtonElement>(null);
  const heading = useId();
  useEffect(() => {
    const previous = document.activeElement;
    const element = dialog.current;
    element?.showModal(); close.current?.focus();
    return () => { element?.close(); if (previous instanceof HTMLElement && previous.isConnected) previous.focus(); };
  }, []);
  return <dialog ref={dialog} className="confirm-dialog step-photo-dialog" aria-labelledby={heading}
    onCancel={e => { e.preventDefault(); onClose(); }}>
    <h2 id={heading}>{t.label(position)}</h2>
    <CoverImage assetId={assetId} variant="preview" client={client} alt={t.label(position)} unavailableText={t.unavailable} />
    <div className="recipe-actions"><Button ref={close} type="button" onClick={onClose}>{t.close}</Button></div>
  </dialog>;
}

export function StepPhoto({ assetId, position, client }: {
  assetId?: string | null; position: number; client: RecipeClient;
}) {
  const [preview, setPreview] = useState(false);
  if (!assetId) return null;
  return <div className="step-photo-preview">
    <CoverImage assetId={assetId} variant="thumb" client={client} alt={t.label(position)} unavailableText={t.unavailable} />
    <Button type="button" variant="outline" aria-label={t.previewLabel(position)} onClick={() => setPreview(true)}>{t.preview}</Button>
    {preview && <PhotoPreview assetId={assetId} position={position} client={client} onClose={() => setPreview(false)} />}
  </div>;
}

export function StepPhotoEditor({ assetId, position, client, onChange, onBusy }: {
  assetId?: string | null; position: number; client: RecipeClient;
  onChange: (id: string | null) => void; onBusy: (busy: boolean) => void;
}) {
  const { busy, error, choose, clearError } = useMediaImport(() => client.pickStepPhoto(), onChange, onBusy);
  return <section className="step-photo-editor" aria-label={t.label(position)} aria-busy={busy}>
    <h3>{t.heading}</h3><p className="muted">{t.help}</p>
    <StepPhoto assetId={assetId} position={position} client={client} />
    <div className="recipe-actions">
      <Button type="button" variant="outline" disabled={busy} onClick={() => void choose()}>{assetId ? t.replace : t.add}</Button>
      {assetId && <Button type="button" variant="ghost" disabled={busy} onClick={() => { clearError(); onChange(null); }}>{t.remove}</Button>}
    </div>
    {busy && <p role="status">{mediaTr.importing}</p>}{error && <p role="alert">{error}</p>}
  </section>;
}
