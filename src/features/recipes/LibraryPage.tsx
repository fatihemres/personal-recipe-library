import { useCallback, useEffect, useRef, useState } from 'react';
import { recipeClient, type RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import type { Draft, Recipe } from '../../shared/contracts/recipe';
import type { RecipeSearchItem } from '../../shared/contracts/recipeSearch';
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
  const [items, setItems] = useState<RecipeSearchItem[]>([]);
  const [total, setTotal] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [suggestions, setSuggestions] = useState<string[]>([]);
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
  const [searchText, setSearchText] = useState('');
  const [ingredientId, setIngredientId] = useState<string | null>(null);
  const [ingredientName, setIngredientName] = useState('');
  const [ingredientQuery, setIngredientQuery] = useState('');
  const [ingredientMatches, setIngredientMatches] = useState<{id:string;name:string}[]>([]);
  const [ingredientSearchError, setIngredientSearchError] = useState(false);
  const [createdFrom, setCreatedFrom] = useState('');
  const [createdTo, setCreatedTo] = useState('');
  const [modifiedFrom, setModifiedFrom] = useState('');
  const [modifiedTo, setModifiedTo] = useState('');
  const [sort, setSort] = useState<'modified'|'created'|'title_asc'|'title_desc'|'relevance'>('modified');
  const [view, setView] = useState<'card'|'list'>('card');
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [confirm, setConfirm] = useState(false);
  const generation = useRef(0);
  const operation = useRef(false);
  const searchRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const timer = window.setTimeout(() => setSearchText(query), 180);
    return () => window.clearTimeout(timer);
  }, [query]);
  useEffect(() => {
    if (!ingredientQuery.trim() || ingredientId || !client.searchAvailable) {
      setIngredientMatches([]);
      setIngredientSearchError(false);
      return;
    }
    let alive = true;
    const timer = window.setTimeout(() => {
      void client.searchAvailable!(ingredientQuery).then(
        (result) => { if (alive) { setIngredientMatches(result.items.slice(0, 8)); setIngredientSearchError(false); } },
        () => { if (alive) { setIngredientMatches([]); setIngredientSearchError(true); } },
      );
    }, 150);
    return () => { alive = false; window.clearTimeout(timer); };
  }, [client, ingredientQuery, ingredientId]);
  const filters = {text:searchText,scope,kind,ingredientId,
    createdFrom:createdFrom || null,createdTo:createdTo || null,
    modifiedFrom:modifiedFrom || null,modifiedTo:modifiedTo || null,
    sort,offset:page*24,limit:24} as const;
  const load = useCallback(async () => {
    const current = ++generation.current;
    setLoading(true);
    setError('');
    try {
      const [recipes, recovery] = await Promise.all([
        client.searchRecipes(filters),
        client.drafts(),
      ]);
      if (current === generation.current) {
        setItems(recipes.items);
        setTotal(recipes.total);
        setHasMore(recipes.hasMore);
        setSuggestions(recipes.suggestions);
        setDrafts(recovery);
      }
    } catch (e) {
      if (current === generation.current) {
        setItems([]);
        setTotal(0);
        setHasMore(false);
        setSuggestions([]);
        setError(errorMessage(normalizeError(e).messageKey));
      }
    } finally {
      if (current === generation.current) setLoading(false);
    }
  // Primitive dependencies keep the client request stable across ordinary renders.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [client, searchText, scope, kind, ingredientId, createdFrom, createdTo, modifiedFrom, modifiedTo, sort, page]);
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
  function resetFilters() {
    setQuery(''); setSearchText(''); setKind(null); setIngredientId(null);
    setIngredientName(''); setIngredientQuery(''); setCreatedFrom('');
    setCreatedTo(''); setModifiedFrom(''); setModifiedTo('');
    setSort('modified'); setPage(0);
  }
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
            <Button variant="outline" onClick={() => { setSelected(null); window.setTimeout(() => searchRef.current?.focus(), 0); }}>
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
                onClick={() => { setScope('active'); setPage(0); }}
              >
                {t.active}
              </Button>
              <Button
                variant={scope === 'archived' ? 'default' : 'outline'}
                onClick={() => { setScope('archived'); setPage(0); }}
              >
                {t.archived}
              </Button>
              <Button
                variant={trash ? 'default' : 'outline'}
                onClick={() => { setScope('trash'); setPage(0); }}
              >
                {t.trash}
              </Button>
            </div>
            <label>
              {t.kind}
              <select
                value={kind ?? ''}
                onChange={(e) =>
                  { setPage(0); setKind(
                    e.target.value === ''
                      ? null
                      : (e.target.value as 'food' | 'beverage'),
                  ); }
                }
              >
                <option value="">{t.all}</option>
                <option value="food">{t.food}</option>
                <option value="beverage">{t.beverage}</option>
              </select>
            </label>
            <label>
              {t.search}
              <input ref={searchRef} type="search" maxLength={200} value={query}
                onChange={(e) => { setQuery(e.target.value); setSort(e.target.value.trim() ? 'relevance' : 'modified'); setPage(0); }}
                onKeyDown={(event) => {
                  if (event.key === 'Escape') { setQuery(''); setPage(0); }
                  if (event.key === 'ArrowDown' && items.length) {
                    event.preventDefault(); document.getElementById('recipe-result-0')?.focus();
                  }
                }}
                aria-describedby="recipe-search-help" />
            </label>
          </div>
          <p id="recipe-search-help" className="muted">{t.searchHelp}</p>
          <div className="recipe-search-filters panel">
            <label>{t.ingredientFilter}
              <input type="search" value={ingredientId ? ingredientName : ingredientQuery}
                onChange={(event) => { setIngredientId(null); setIngredientName(''); setIngredientQuery(event.target.value); setPage(0); }} />
            </label>
            {ingredientMatches.length > 0 && !ingredientId && <div className="recipe-ingredient-options" role="group" aria-label={t.ingredientMatches}>
              {ingredientMatches.map((ingredient) => <Button variant="outline" key={ingredient.id}
                onClick={() => { setIngredientId(ingredient.id); setIngredientName(ingredient.name); setIngredientMatches([]); setPage(0); }}>
                {ingredient.name}</Button>)}
            </div>}
            {ingredientSearchError && <p role="alert">{t.ingredientSearchFailed}</p>}
            {ingredientQuery.trim() && !ingredientId && !ingredientSearchError && ingredientMatches.length === 0 && <p className="muted">{t.selectIngredientFilter}</p>}
            <label>{t.createdFrom}<input type="date" value={createdFrom} onChange={e => {setCreatedFrom(e.target.value);setPage(0);}} /></label>
            <label>{t.createdTo}<input type="date" value={createdTo} onChange={e => {setCreatedTo(e.target.value);setPage(0);}} /></label>
            <label>{t.modifiedFrom}<input type="date" value={modifiedFrom} onChange={e => {setModifiedFrom(e.target.value);setPage(0);}} /></label>
            <label>{t.modifiedTo}<input type="date" value={modifiedTo} onChange={e => {setModifiedTo(e.target.value);setPage(0);}} /></label>
            <label>{t.sort}<select value={sort} onChange={e => {setSort(e.target.value as typeof sort);setPage(0);}}>
              <option value="modified">{t.sortModified}</option><option value="created">{t.sortCreated}</option>
              <option value="title_asc">{t.sortTitleAsc}</option><option value="title_desc">{t.sortTitleDesc}</option>
              <option value="relevance" disabled={!query.trim()}>{t.sortRelevance}</option>
            </select></label>
            <Button variant="outline" onClick={resetFilters}>{t.resetFilters}</Button>
          </div>
          <div className="recipe-results-heading">
            <p role="status">{t.resultCount.replace('{count}', String(total))}</p>
            <div role="group" aria-label={t.viewMode} className="recipe-actions">
              <Button variant={view === 'card' ? 'default' : 'outline'} aria-pressed={view === 'card'} onClick={() => setView('card')}>{t.cardView}</Button>
              <Button variant={view === 'list' ? 'default' : 'outline'} aria-pressed={view === 'list'} onClick={() => setView('list')}>{t.listView}</Button>
            </div>
          </div>
          {loading ? (
            <p role="status">{t.loading}</p>
          ) : error ? null : !items.length ? (
            <div className="panel empty-state">
              <h2>
                {query || kind || ingredientId || createdFrom || createdTo || modifiedFrom || modifiedTo
                  ? t.noMatches
                  : trash
                    ? t.emptyTrash
                    : scope === 'archived'
                      ? t.emptyArchive
                      : t.empty}
              </h2>
              {!trash && !query && !kind && <p>{t.emptyBody}</p>}
              {suggestions.length > 0 && <div className="recipe-actions"><span>{t.didYouMean}</span>{suggestions.map(value => <Button key={value} variant="outline" onClick={() => {setQuery(value);setPage(0);}}>{value}</Button>)}</div>}
              {(query || kind || ingredientId || createdFrom || createdTo || modifiedFrom || modifiedTo) && <Button variant="outline" onClick={resetFilters}>{t.resetFilters}</Button>}
            </div>
          ) : (
            <div className={view === 'card' ? 'recipe-grid' : 'recipe-list'}>
              {items.map((recipe, index) => (
                <button
                  id={`recipe-result-${index}`}
                  className={`panel recipe-card ${view === 'list' ? 'recipe-list-row' : ''}`}
                  key={recipe.id}
                  disabled={busy}
                  onClick={() => void open(recipe.id)}
                  onKeyDown={(event) => {
                    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
                      event.preventDefault();
                      document.getElementById(`recipe-result-${index + (event.key === 'ArrowDown' ? 1 : -1)}`)?.focus();
                    }
                  }}
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
          {!loading && total > 24 && <div className="recipe-pagination" role="group" aria-label={t.pages}>
            <Button variant="outline" disabled={page === 0} onClick={() => setPage(p => p-1)}>{t.previousPage}</Button>
            <span>{t.pageNumber.replace('{page}', String(page+1))}</span>
            <Button variant="outline" disabled={!hasMore} onClick={() => setPage(p => p+1)}>{t.nextPage}</Button>
          </div>}
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
