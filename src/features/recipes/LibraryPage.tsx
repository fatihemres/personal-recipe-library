import { useCallback, useEffect, useRef, useState } from 'react';
import { recipeClient, type RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import type { Draft, Recipe } from '../../shared/contracts/recipe';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { RecipeDetails } from './RecipeDetails';
import { DraftRecovery } from './DraftRecovery';
import { RecipeEditor } from './RecipeEditor';
export function LibraryPage({
  client = recipeClient,
}: {
  client?: RecipeClient;
}) {
  const [items, setItems] = useState<Recipe[]>([]);
  const [selected, setSelected] = useState<Recipe | null>(null);
  const [editing, setEditing] = useState(false);
  const [kind, setKind] = useState<'food' | 'beverage' | null>(null);
  const [scope, setScope] = useState<'active' | 'archived' | 'trash'>('active');
  const trash = scope === 'trash';
  const [drafts, setDrafts] = useState<Draft[]>([]);
  const [recovered, setRecovered] = useState<Draft>();
  const [discardTarget, setDiscardTarget] = useState<Draft>();
  const [purgeConfirm, setPurgeConfirm] = useState(false);
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [confirm, setConfirm] = useState(false);
  const generation = useRef(0);
  const operation = useRef(false);
  const load = useCallback(async () => {
    const current = ++generation.current;
    setLoading(true);
    setError('');
    try {
      const [recipes, recovery] = await Promise.all([
        client.scope(scope, kind),
        client.drafts(),
      ]);
      if (current === generation.current) {
        setItems(recipes);
        setDrafts(recovery);
      }
    } catch (e) {
      if (current === generation.current)
        setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      if (current === generation.current) setLoading(false);
    }
  }, [client, scope, kind]);
  const invalidate = useCallback(() => {
    generation.current++;
  }, []);
  useEffect(() => {
    void load();
    return invalidate;
  }, [load, invalidate]);
  async function open(id: string) {
    if (operation.current) return;
    operation.current = true;
    setBusy(true);
    setError('');
    try {
      setSelected(await client.get(id));
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      setBusy(false);
      operation.current = false;
    }
  }
  async function deletion(deleted: boolean) {
    if (!selected || operation.current) return;
    operation.current = true;
    setBusy(true);
    setConfirm(false);
    try {
      await client.setDeleted(selected.id, selected.revision, deleted);
      setSelected(null);
      setNotice(deleted ? t.deleted : t.restored);
      await load();
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      operation.current = false;
      setBusy(false);
    }
  }
  async function management(
    action: 'duplicate' | 'archive' | 'unarchive' | 'purge',
  ) {
    if (!selected || operation.current) return;
    operation.current = true;
    setBusy(true);
    setError('');
    setPurgeConfirm(false);
    try {
      if (action === 'purge') {
        await client.purge(selected.id, selected.revision);
        setSelected(null);
        setNotice(t.purged);
      } else if (action === 'duplicate') {
        setSelected(await client.duplicate(selected.id, selected.revision));
        setScope('active');
        setNotice(t.duplicated);
      } else {
        setSelected(
          await client.archive(
            selected.id,
            selected.revision,
            action === 'archive',
          ),
        );
      }
      await load();
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      operation.current = false;
      setBusy(false);
    }
  }
  async function recover(id: string) {
    if (operation.current) return;
    operation.current = true;
    setBusy(true);
    setError('');
    try {
      const draft = await client.getDraft(id);
      let original: Recipe | null = null;
      if (draft.recipeId) {
        try {
          original = await client.get(draft.recipeId);
        } catch (e) {
          if (normalizeError(e).code !== 'NOT_FOUND') throw e;
        }
      }
      setSelected(original);
      setRecovered(draft);
      setEditing(true);
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      operation.current = false;
      setBusy(false);
    }
  }
  async function discardRecovery() {
    if (!discardTarget || operation.current) return;
    operation.current = true;
    setBusy(true);
    try {
      await client.discardDraft(discardTarget.id, discardTarget.revision);
      setDiscardTarget(undefined);
      await load();
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
      setDiscardTarget(undefined);
    } finally {
      operation.current = false;
      setBusy(false);
    }
  }
  function saved(recipe: Recipe) {
    setEditing(false);
    setRecovered(undefined);
    setScope('active');
    setSelected(recipe);
    setNotice(t.saved);
    void load();
  }
  if (editing)
    return (
      <RecipeEditor
        key={recovered?.id ?? selected?.id ?? 'new'}
        recovered={recovered}
        recipe={selected}
        client={client}
        onSaved={saved}
        onCancel={() => {
          setEditing(false);
          setRecovered(undefined);
          void load();
        }}
      />
    );
  const normalize = (s: string) => s.normalize('NFC').toLocaleLowerCase('tr');
  const shown = items.filter((r) =>
    normalize(r.title).includes(normalize(query)),
  );
  return (
    <section>
      <div className="page-heading">
        <div>
          <p className="eyebrow">{t.brand}</p>
          <h1>{selected ? selected.title : t.heading}</h1>
          <p>{selected ? selected.description : t.intro}</p>
        </div>
        <Button
          disabled={busy}
          onClick={() => {
            setSelected(null);
            setRecovered(undefined);
            setEditing(true);
          }}
        >
          {t.newRecipe}
        </Button>
      </div>
      {error && (
        <div role="alert" className="recipe-error">
          <p>{error}</p>
          <Button variant="outline" onClick={() => void load()}>
            {t.retry}
          </Button>
        </div>
      )}
      {notice && <p role="status">{notice}</p>}
      {!selected && (
        <DraftRecovery
          drafts={drafts}
          busy={busy}
          onRecover={(id) => void recover(id)}
          onDiscard={setDiscardTarget}
        />
      )}
      {selected ? (
        <>
          <div className="recipe-actions">
            <Button variant="outline" onClick={() => setSelected(null)}>
              {t.back}
            </Button>
            {selected.deletedAt ? (
              <>
                <Button disabled={busy} onClick={() => void deletion(false)}>
                  {t.restore}
                </Button>
                <Button
                  disabled={busy}
                  variant="outline"
                  onClick={() => setPurgeConfirm(true)}
                >
                  {t.purge}
                </Button>
              </>
            ) : (
              <>
                {!selected.archivedAt && (
                  <Button
                    disabled={busy}
                    onClick={() => {
                      setRecovered(undefined);
                      setEditing(true);
                    }}
                  >
                    {t.edit}
                  </Button>
                )}
                <Button
                  disabled={busy}
                  variant="outline"
                  onClick={() => void management('duplicate')}
                >
                  {t.duplicate}
                </Button>
                <Button
                  disabled={busy}
                  variant="outline"
                  onClick={() =>
                    void management(
                      selected.archivedAt ? 'unarchive' : 'archive',
                    )
                  }
                >
                  {selected.archivedAt ? t.unarchive : t.archive}
                </Button>
                <Button
                  disabled={busy}
                  variant="outline"
                  onClick={() => setConfirm(true)}
                >
                  {t.delete}
                </Button>
              </>
            )}
          </div>
          <RecipeDetails recipe={selected} />
        </>
      ) : (
        <>
          <div className="recipe-toolbar">
            <div className="recipe-actions">
              <Button
                variant={scope === 'active' ? 'default' : 'outline'}
                onClick={() => setScope('active')}
              >
                {t.active}
              </Button>
              <Button
                variant={scope === 'archived' ? 'default' : 'outline'}
                onClick={() => setScope('archived')}
              >
                {t.archived}
              </Button>
              <Button
                variant={trash ? 'default' : 'outline'}
                onClick={() => setScope('trash')}
              >
                {t.trash}
              </Button>
            </div>
            <label>
              {t.kind}
              <select
                value={kind ?? ''}
                onChange={(e) =>
                  setKind(
                    e.target.value === ''
                      ? null
                      : (e.target.value as 'food' | 'beverage'),
                  )
                }
              >
                <option value="">{t.all}</option>
                <option value="food">{t.food}</option>
                <option value="beverage">{t.beverage}</option>
              </select>
            </label>
            <label>
              {t.search}
              <input value={query} onChange={(e) => setQuery(e.target.value)} />
            </label>
          </div>
          {loading ? (
            <p role="status">{t.loading}</p>
          ) : !shown.length ? (
            <div className="panel empty-state">
              <h2>
                {query || kind
                  ? t.noMatches
                  : trash
                    ? t.emptyTrash
                    : scope === 'archived'
                      ? t.emptyArchive
                      : t.empty}
              </h2>
              {!trash && !query && !kind && <p>{t.emptyBody}</p>}
            </div>
          ) : (
            <div className="recipe-grid">
              {shown.map((recipe) => (
                <button
                  className="panel recipe-card"
                  key={recipe.id}
                  disabled={busy}
                  onClick={() => void open(recipe.id)}
                >
                  <span className="badge">
                    {recipe.kind === 'food' ? t.food : t.beverage}
                  </span>
                  <h2>{recipe.title}</h2>
                  <p>{recipe.description}</p>
                  <span>
                    {recipe.servings} {t.portions}
                  </span>
                </button>
              ))}
            </div>
          )}
        </>
      )}
      {discardTarget && (
        <ConfirmDialog
          title={t.confirmDraftDiscard}
          description={t.draftDiscardHelp}
          confirm={t.discardDraft}
          cancel={t.cancel}
          onConfirm={() => void discardRecovery()}
          onCancel={() => setDiscardTarget(undefined)}
        />
      )}
      {purgeConfirm && (
        <ConfirmDialog
          title={t.confirmPurge}
          description={t.purgeHelp}
          confirm={t.purge}
          cancel={t.cancel}
          onConfirm={() => void management('purge')}
          onCancel={() => setPurgeConfirm(false)}
        />
      )}
      {confirm && (
        <ConfirmDialog
          title={t.confirmDelete}
          description={t.deleteHelp}
          confirm={t.delete}
          cancel={t.cancel}
          onCancel={() => setConfirm(false)}
          onConfirm={() => void deletion(true)}
        />
      )}
    </section>
  );
}
