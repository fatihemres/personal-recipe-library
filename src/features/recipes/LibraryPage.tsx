import { useCallback, useEffect, useRef, useState } from 'react';
import { recipeClient, type RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import type { Recipe } from '../../shared/contracts/recipe';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
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
  const [trash, setTrash] = useState(false);
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
      const recipes = await client.list(trash, kind);
      if (current === generation.current) setItems(recipes);
    } catch (e) {
      if (current === generation.current)
        setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      if (current === generation.current) setLoading(false);
    }
  }, [client, trash, kind]);
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
  function saved(recipe: Recipe) {
    setEditing(false);
    setSelected(recipe);
    setNotice(t.saved);
    void load();
  }
  if (editing)
    return (
      <RecipeEditor
        key={selected?.id ?? 'new'}
        recipe={selected}
        client={client}
        onSaved={saved}
        onCancel={() => setEditing(false)}
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
      {selected ? (
        <>
          <div className="recipe-actions">
            <Button variant="outline" onClick={() => setSelected(null)}>
              {t.back}
            </Button>
            {selected.deletedAt ? (
              <Button disabled={busy} onClick={() => void deletion(false)}>
                {t.restore}
              </Button>
            ) : (
              <>
                <Button disabled={busy} onClick={() => setEditing(true)}>
                  {t.edit}
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
          <div className="panel recipe-form">
            <p>
              {selected.kind === 'food' ? t.food : t.beverage} ·{' '}
              {selected.servings} {t.portions}
            </p>
            {selected.prepMinutes !== null && (
              <p>
                {t.prep}: {selected.prepMinutes} {t.minute}
              </p>
            )}
            {selected.cookMinutes !== null && (
              <p>
                {t.cook}: {selected.cookMinutes} {t.minute}
              </p>
            )}
            <h2>{t.ingredients}</h2>
            {selected.ingredients.length ? (
              <ul className="detail-list">
                {selected.ingredients.map((i) => (
                  <li key={i.id}>
                    <strong>{selected.ingredientNames[i.ingredientId]}</strong>{' '}
                    —{' '}
                    {i.quantity === null
                      ? t.unknown
                      : `${i.quantity} ${i.unitCode}`}
                    {i.note && <p>{i.note}</p>}
                  </li>
                ))}
              </ul>
            ) : (
              <p>{t.noIngredients}</p>
            )}
            <h2>{t.steps}</h2>
            {selected.steps.length ? (
              <ol className="detail-list">
                {selected.steps.map((s) => (
                  <li key={s.id}>{s.instructions}</li>
                ))}
              </ol>
            ) : (
              <p>{t.noSteps}</p>
            )}
            {selected.notes && (
              <>
                <h2>{t.notes}</h2>
                <p className="preserve-text">{selected.notes}</p>
              </>
            )}
          </div>
        </>
      ) : (
        <>
          <div className="recipe-toolbar">
            <div className="recipe-actions">
              <Button
                variant={!trash ? 'default' : 'outline'}
                onClick={() => setTrash(false)}
              >
                {t.active}
              </Button>
              <Button
                variant={trash ? 'default' : 'outline'}
                onClick={() => setTrash(true)}
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
                {query || kind ? t.noMatches : trash ? t.emptyTrash : t.empty}
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
