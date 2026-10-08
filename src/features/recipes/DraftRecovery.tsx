import type { Draft } from '../../shared/contracts/recipe';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
export function DraftRecovery({
  drafts,
  busy,
  onRecover,
  onDiscard,
}: {
  drafts: Draft[];
  busy: boolean;
  onRecover: (id: string) => void;
  onDiscard: (draft: Draft) => void;
}) {
  if (!drafts.length) return null;
  return (
    <section className="panel recipe-form" aria-label={t.draftRecovery}>
      <h2>{t.draftRecovery}</h2>
      <p>{t.draftHelp}</p>
      <ul className="draft-list">
        {drafts.map((draft) => (
          <li key={draft.id}>
            <strong>{draft.input.title || t.untitled}</strong>
            <p>
              {draft.input.expectedRevision === null ? t.newDraft : t.editDraft}{' '}
              · {new Date(draft.updatedAt).toLocaleString('tr-TR')}
            </p>
            <div className="recipe-actions">
              <Button
                disabled={busy}
                variant="outline"
                onClick={() => onRecover(draft.id)}
              >
                {t.recover}
              </Button>
              <Button
                disabled={busy}
                variant="ghost"
                onClick={() => onDiscard(draft)}
              >
                {t.discardDraft}
              </Button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
