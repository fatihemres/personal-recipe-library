import type { RecipeClient } from '../../shared/api/recipes';
import { mediaTr as t } from '../../shared/i18n/media';
import { Button } from '../../shared/ui/button';
import { CoverImage, useMediaImport } from './RecipeMedia';
export { CoverImage } from './RecipeMedia';

export function RecipeCoverEditor({ assetId, client, onChange, onBusy }: {
  assetId?: string | null; client: RecipeClient;
  onChange: (id: string | null) => void; onBusy: (busy: boolean) => void;
}) {
  const { busy, error, choose, clearError } = useMediaImport(() => client.pickCover(), onChange, onBusy);
  return <section className="panel recipe-form cover-editor" aria-label={t.cover} aria-busy={busy}>
    <h2>{t.cover}</h2><p>{t.help}</p>
    <details><summary>{t.privacy}</summary><p>{t.retention}</p></details>
    <CoverImage assetId={assetId} variant="preview" client={client} />
    <div className="recipe-actions">
      <Button type="button" variant="outline" disabled={busy} onClick={() => void choose()}>{assetId ? t.replace : t.add}</Button>
      {assetId && <Button type="button" variant="ghost" disabled={busy} onClick={() => {
        clearError(); onChange(null);
      }}>{t.remove}</Button>}
    </div>
    {busy && <p role="status">{t.importing}</p>}{error && <p role="alert">{error}</p>}
    <p>{t.saveHelp}</p>
  </section>;
}
