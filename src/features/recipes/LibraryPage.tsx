import { useCallback, useEffect, useRef, useState } from 'react';
import { recipeClient, type RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import type { Draft, Recipe } from '../../shared/contracts/recipe';
import type { RecipeSearchItem, RecipeSearchQuery } from '../../shared/contracts/recipeSearch';
import type { SavedFilter } from '../../shared/contracts/recipeWorkspace';
import type { RecipeOrganization, RecipeCollection } from '../../shared/contracts/recipeOrganization';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { RecipeDetails } from './RecipeDetails';
import { DraftRecovery } from './DraftRecovery';
import { RecipeEditor } from './RecipeEditor';
import { PrivateNotePanel } from './PrivateNotePanel';
export function LibraryPage({
  client = recipeClient,
  section = 'library',
  navigationRequest,
  registerLeaveGuard,
}: {
  client?: RecipeClient;
  section?: 'library' | 'collections';
  navigationRequest?: {sequence:number; kind?:'food'|'beverage'; favoriteOnly?:boolean; collectionId?:string; recipeId?:string};
  registerLeaveGuard?: (guard:(()=>Promise<boolean>)|null)=>void;
}) {
  const [items, setItems] = useState<RecipeSearchItem[]>([]);
  const [total, setTotal] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [selected, setSelected] = useState<Recipe | null>(null);
  const [organization, setOrganization] = useState<RecipeOrganization | null>(null);
  const [collections, setCollections] = useState<RecipeCollection[]>([]);
  const [savedFilters,setSavedFilters] = useState<SavedFilter[]>([]);
  const [filterName,setFilterName] = useState('');
  const [renamingFilter,setRenamingFilter] = useState<SavedFilter|null>(null);
  const [collectionId, setCollectionId] = useState<string | null>(null);
  const [collectionTitle, setCollectionTitle] = useState('');
  const [collectionEditing, setCollectionEditing] = useState<RecipeCollection | null>(null);
  const [collectionDelete, setCollectionDelete] = useState<RecipeCollection | null>(null);
  const [favoriteOnly, setFavoriteOnly] = useState(false);
  const [toTryOnly, setToTryOnly] = useState(false);
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
  const noteLeaveGuard=useRef<(()=>Promise<boolean>)|null>(null);
  useEffect(()=>{
    registerLeaveGuard?.(async()=>noteLeaveGuard.current ? noteLeaveGuard.current():true);
    return()=>registerLeaveGuard?.(null);
  },[registerLeaveGuard]);
  useEffect(()=>{
    if(!navigationRequest) return;
    setScope('active');setPage(0);setKind(navigationRequest.kind??null);
    setFavoriteOnly(!!navigationRequest.favoriteOnly);setToTryOnly(false);
    setCollectionId(navigationRequest.collectionId??null);
    setQuery('');setSearchText('');setIngredientId(null);
    setCreatedFrom('');setCreatedTo('');setModifiedFrom('');setModifiedTo('');setSort('modified');
    if(navigationRequest.recipeId) void (async()=>{if(!noteLeaveGuard.current || await noteLeaveGuard.current()) await open(navigationRequest.recipeId!);})();
    else {setSelected(null);setOrganization(null);}
  // The sequence marks a deliberate external navigation, not ordinary rerenders.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  },[navigationRequest?.sequence]);
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
  const filters = {text:searchText,scope,kind,ingredientId,favoriteOnly,toTryOnly,collectionId,
    createdFrom:createdFrom || null,createdTo:createdTo || null,
    modifiedFrom:modifiedFrom || null,modifiedTo:modifiedTo || null,
    sort,offset:page*24,limit:24} as const;
  const load = useCallback(async () => {
    const current = ++generation.current;
    setLoading(true);
    setError('');
    try {
      const [recipes, recovery, groups, saved] = await Promise.all([
        client.searchRecipes(filters),
        client.drafts(),
        client.collections(),
        client.savedFilters(),
      ]);
      if (current === generation.current) {
        setItems(recipes.items);
        setTotal(recipes.total);
        setHasMore(recipes.hasMore);
        setSuggestions(recipes.suggestions);
        setDrafts(recovery);
        setCollections(groups);
        setSavedFilters(saved);
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
  }, [client, searchText, scope, kind, ingredientId, favoriteOnly, toTryOnly, collectionId, createdFrom, createdTo, modifiedFrom, modifiedTo, sort, page]);
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
      const [recipe, state] = await Promise.all([client.get(id),client.getOrganization(id)]);
      setSelected(recipe);
      setOrganization(state);
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      setBusy(false);
      operation.current = false;
    }
  }
  async function updateOrganization(next: Pick<RecipeOrganization,'favorite'|'toTry'|'rating'>, id: string, revision: number) {
    if (operation.current) return;
    operation.current = true; setBusy(true); setError('');
    try {
      const state = await client.setOrganization({recipeId:id,expectedRevision:revision,...next});
      setItems(previous => previous.map(item => item.id===id ? {...item,organizationRevision:state.revision,favorite:state.favorite,toTry:state.toTry,rating:state.rating} : item));
      if (selected?.id===id) setOrganization(state);
      await load();
    } catch (e) { setError(errorMessage(normalizeError(e).messageKey)); }
    finally { operation.current=false; setBusy(false); }
  }
  async function toggleMembership(group: RecipeCollection, member: boolean) {
    if (!selected || !organization || operation.current) return;
    operation.current=true; setBusy(true); setError('');
    try {
      setOrganization(await client.setMembership(selected.id,group.id,organization.revision,member));
      await load();
    } catch(e) { setError(errorMessage(normalizeError(e).messageKey)); }
    finally { operation.current=false; setBusy(false); }
  }
  async function saveCollection() {
    if (operation.current) return;
    operation.current=true; setBusy(true); setError('');
    try {
      await client.saveCollection({id:collectionEditing?.id ?? crypto.randomUUID(),expectedRevision:collectionEditing?.revision ?? null,title:collectionTitle});
      setCollectionTitle(''); setCollectionEditing(null); await load();
    } catch(e) { setError(errorMessage(normalizeError(e).messageKey)); }
    finally { operation.current=false; setBusy(false); }
  }
  async function saveCurrentFilter() {
    if(operation.current || !filterName.trim()) return;
    operation.current=true;setBusy(true);setError('');
    try {
      await client.saveFilter({id:renamingFilter?.id??crypto.randomUUID(),expectedRevision:renamingFilter?.revision??null,
        title:filterName,query:(renamingFilter?.query ?? {...filters,offset:0,limit:24}) as RecipeSearchQuery & {offset:0;limit:24}});
      setFilterName('');setRenamingFilter(null);setNotice(t.filterSaved);await load();
    } catch(e){setError(errorMessage(normalizeError(e).messageKey));}
    finally{operation.current=false;setBusy(false);}
  }
  async function removeFilter(filter:SavedFilter) {
    if(operation.current) return;
    operation.current=true;setBusy(true);setError('');
    try {await client.deleteFilter(filter.id,filter.revision);setNotice(t.filterDeleted);await load();}
    catch(e){setError(errorMessage(normalizeError(e).messageKey));}
    finally{operation.current=false;setBusy(false);}
  }
  async function applySavedFilter(filter:SavedFilter) {
    let q:RecipeSearchQuery;
    try{q=await client.resolveFilter(filter.id);}catch(e){setError(errorMessage(normalizeError(e).messageKey));return;}
    setQuery(q.text);setSearchText(q.text);setScope(q.scope);setKind(q.kind);setIngredientId(q.ingredientId);
    setIngredientName('');setIngredientQuery('');setFavoriteOnly(q.favoriteOnly);setToTryOnly(q.toTryOnly);
    setCollectionId(q.collectionId);setCreatedFrom(q.createdFrom??'');setCreatedTo(q.createdTo??'');
    setModifiedFrom(q.modifiedFrom??'');setModifiedTo(q.modifiedTo??'');setSort(q.sort);setPage(0);
    searchRef.current?.focus();
  }
  async function removeCollection() {
    if (!collectionDelete || operation.current) return;
    operation.current=true; setBusy(true); setError('');
    try {
      await client.deleteCollection(collectionDelete.id,collectionDelete.revision);
      if (collectionId===collectionDelete.id) setCollectionId(null);
      setCollectionDelete(null); await load();
    } catch(e) { setError(errorMessage(normalizeError(e).messageKey)); }
    finally { operation.current=false; setBusy(false); }
  }
  async function deletion(deleted: boolean) {
    if (!selected || operation.current) return;
    if(noteLeaveGuard.current && !await noteLeaveGuard.current()) return;
    operation.current = true;
    setBusy(true);
    setConfirm(false);
    try {
      await client.setDeleted(selected.id, selected.revision, deleted);
      setSelected(null);
      setOrganization(null);
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
    if(noteLeaveGuard.current && !await noteLeaveGuard.current()) return;
    operation.current = true;
    setBusy(true);
    setError('');
    setPurgeConfirm(false);
    try {
      if (action === 'purge') {
        await client.purge(selected.id, selected.revision);
        setSelected(null);
        setOrganization(null);
        setNotice(t.purged);
      } else if (action === 'duplicate') {
        const copy = await client.duplicate(selected.id, selected.revision);
        setSelected(copy);
        setOrganization(await client.getOrganization(copy.id));
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
      setOrganization(original ? await client.getOrganization(original.id) : null);
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
    void client.getOrganization(recipe.id).then(setOrganization, e => setError(errorMessage(normalizeError(e).messageKey)));
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
    setFavoriteOnly(false); setToTryOnly(false); setCollectionId(null);
    setIngredientName(''); setIngredientQuery(''); setCreatedFrom('');
    setCreatedTo(''); setModifiedFrom(''); setModifiedTo('');
    setSort('modified'); setPage(0);
  }
  return (
    <section>
      <div className="page-heading">
        <div>
          <p className="eyebrow">{t.brand}</p>
          <h1>{selected ? selected.title : section === 'collections' ? t.collections : t.heading}</h1>
          <p>{selected ? selected.description : t.intro}</p>
        </div>
        <Button
          disabled={busy}
          onClick={() => {
            void (async()=>{
              if(noteLeaveGuard.current && !await noteLeaveGuard.current())return;
              setSelected(null);setRecovered(undefined);setEditing(true);
            })();
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
            <Button variant="outline" onClick={() => { void (async()=>{if(noteLeaveGuard.current && !await noteLeaveGuard.current())return;setSelected(null);window.setTimeout(() => searchRef.current?.focus(), 0);})(); }}>
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
                    onClick={() => {void (async()=>{if(noteLeaveGuard.current && !await noteLeaveGuard.current())return;setRecovered(undefined);setEditing(true);})();}}
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
          <PrivateNotePanel key={selected.id} recipeId={selected.id} client={client} readOnly={!!selected.deletedAt} registerLeaveGuard={guard=>{noteLeaveGuard.current=guard;}} />
          {organization && !selected.deletedAt && <div className="panel recipe-organization">
            <div className="recipe-actions" role="group" aria-label={t.favorites}>
              <Button variant={organization.favorite ? 'default' : 'outline'} aria-pressed={organization.favorite} disabled={busy}
                onClick={() => void updateOrganization({favorite:!organization.favorite,toTry:organization.toTry,rating:organization.rating},selected.id,organization.revision)}>
                {organization.favorite ? t.favoriteRemove : t.favoriteAdd}</Button>
              <Button variant={organization.toTry ? 'default' : 'outline'} aria-pressed={organization.toTry} disabled={busy}
                onClick={() => void updateOrganization({favorite:organization.favorite,toTry:!organization.toTry,rating:organization.rating},selected.id,organization.revision)}>
                {organization.toTry ? t.toTryRemove : t.toTryAdd}</Button>
            </div>
            <fieldset><legend>{t.rating}</legend><p>{organization.rating === null ? t.unrated : `${organization.rating}/5`}</p><div className="recipe-actions">
              {[1,2,3,4,5].map(value => <Button key={value} variant={organization.rating===value ? 'default' : 'outline'}
                aria-pressed={organization.rating===value} disabled={busy} aria-label={t.ratingStar.replace('{count}',String(value))}
                onClick={() => void updateOrganization({favorite:organization.favorite,toTry:organization.toTry,rating:value},selected.id,organization.revision)}>{value} ★</Button>)}
              <Button variant="outline" disabled={busy || organization.rating===null} onClick={() => void updateOrganization({favorite:organization.favorite,toTry:organization.toTry,rating:null},selected.id,organization.revision)}>{t.ratingClear}</Button>
            </div></fieldset>
            <fieldset><legend>{t.collectionMembership}</legend>
              {collections.length ? collections.map(group => <label key={group.id} className="recipe-collection-choice">
                <input type="checkbox" checked={organization.collectionIds.includes(group.id)} disabled={busy}
                  onChange={event => void toggleMembership(group,event.target.checked)} /> {group.title}
              </label>) : <p>{t.collectionEmpty}</p>}
            </fieldset>
          </div>}
        </>
      ) : (
        <>
          {section === 'collections' && <div className="panel recipe-collections">
            <h2>{t.collections}</h2>
            <form onSubmit={event => {event.preventDefault();void saveCollection();}} className="recipe-actions">
              <label>{t.collectionName}<input value={collectionTitle} maxLength={100} required onChange={event=>setCollectionTitle(event.target.value)} /></label>
              <Button type="submit" disabled={busy || !collectionTitle.trim()}>{collectionEditing ? t.collectionRename : t.collectionCreate}</Button>
              {collectionEditing && <Button type="button" variant="outline" onClick={()=>{setCollectionEditing(null);setCollectionTitle('');}}>{t.cancel}</Button>}
            </form>
            {collections.length ? <ul className="recipe-collection-list">{collections.map(group => <li key={group.id}>
              <Button variant={collectionId===group.id ? 'default':'outline'} aria-pressed={collectionId===group.id}
                onClick={()=>{setCollectionId(group.id);setScope('active');setPage(0);}}>{group.title} · {t.collectionCount.replace('{count}',String(group.activeCount))}</Button>
              <Button variant="outline" aria-label={`${group.title}: ${t.collectionRename}`} onClick={()=>{setCollectionEditing(group);setCollectionTitle(group.title);}}>{t.collectionRename}</Button>
              <Button variant="outline" aria-label={`${group.title}: ${t.collectionDelete}`} onClick={()=>setCollectionDelete(group)}>{t.collectionDelete}</Button>
            </li>)}</ul> : <p>{t.collectionEmpty}</p>}
            {collectionId && <Button variant="outline" onClick={()=>{setCollectionId(null);setPage(0);}}>{t.collectionFilterClear}</Button>}
          </div>}
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
                onClick={() => { setScope('archived'); setFavoriteOnly(false); setToTryOnly(false); setCollectionId(null); setPage(0); }}
              >
                {t.archived}
              </Button>
              <Button
                variant={trash ? 'default' : 'outline'}
                onClick={() => { setScope('trash'); setFavoriteOnly(false); setToTryOnly(false); setCollectionId(null); setPage(0); }}
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
            {scope === 'active' && <div className="recipe-actions" role="group" aria-label={t.favorites}>
              <Button variant={favoriteOnly ? 'default':'outline'} aria-pressed={favoriteOnly} onClick={()=>{setFavoriteOnly(!favoriteOnly);setPage(0);}}>{t.favorites}</Button>
              <Button variant={toTryOnly ? 'default':'outline'} aria-pressed={toTryOnly} onClick={()=>{setToTryOnly(!toTryOnly);setPage(0);}}>{t.toTry}</Button>
            </div>}
            <label>
              {t.search}
              <input ref={searchRef} type="search" maxLength={200} value={query}
                onChange={(e) => { setQuery(e.target.value); setSort(e.target.value.trim() ? 'relevance' : 'modified'); setPage(0); }}
                onKeyDown={(event) => {
                  if (event.key === 'Escape') { setQuery(''); setPage(0); }
                  if (event.key === 'ArrowDown' && items.length) {
                    event.preventDefault(); document.querySelector<HTMLElement>('#recipe-result-0 .recipe-card-open')?.focus();
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
          <section className="panel saved-filter-panel" aria-label={t.savedFilters}>
            <h2>{t.savedFilters}</h2>
            <form className="recipe-actions" onSubmit={e=>{e.preventDefault();void saveCurrentFilter();}}>
              <label>{t.filterName}<input value={filterName} maxLength={100} required onChange={e=>setFilterName(e.target.value)} /></label>
              <Button type="submit" disabled={busy||!filterName.trim()}>{renamingFilter?t.filterRename:t.saveFilter}</Button>
              {renamingFilter && <Button type="button" variant="outline" onClick={()=>{setRenamingFilter(null);setFilterName('');}}>{t.cancel}</Button>}
            </form>
            {savedFilters.length ? <ul className="saved-filter-list">{savedFilters.map(filter=><li key={filter.id}>
              <Button variant="outline" onClick={()=>void applySavedFilter(filter)}>{filter.title} · {t.filterApply}</Button>
              <Button variant="outline" disabled={!filter.available} onClick={()=>{setRenamingFilter(filter);setFilterName(filter.title);}} aria-label={`${filter.title}: ${t.filterRename}`}>{t.filterRename}</Button>
              <Button variant="outline" onClick={()=>void removeFilter(filter)} aria-label={`${filter.title}: ${t.filterDelete}`}>{t.filterDelete}</Button>
              {!filter.available && <span role="status">{t.filterUnavailable}</span>}
            </li>)}</ul> : <p>{t.filterEmpty}</p>}
          </section>
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
                {query || kind || ingredientId || favoriteOnly || toTryOnly || collectionId || createdFrom || createdTo || modifiedFrom || modifiedTo
                  ? t.noMatches
                  : trash
                    ? t.emptyTrash
                    : scope === 'archived'
                      ? t.emptyArchive
                      : t.empty}
              </h2>
              {!trash && !query && !kind && <p>{t.emptyBody}</p>}
              {suggestions.length > 0 && <div className="recipe-actions"><span>{t.didYouMean}</span>{suggestions.map(value => <Button key={value} variant="outline" onClick={() => {setQuery(value);setPage(0);}}>{value}</Button>)}</div>}
              {(query || kind || ingredientId || favoriteOnly || toTryOnly || collectionId || createdFrom || createdTo || modifiedFrom || modifiedTo) && <Button variant="outline" onClick={resetFilters}>{t.resetFilters}</Button>}
            </div>
          ) : (
            <div className={view === 'card' ? 'recipe-grid' : 'recipe-list'}>
              {items.map((recipe, index) => (
                <div
                  id={`recipe-result-${index}`}
                  className={`panel recipe-card ${view === 'list' ? 'recipe-list-row' : ''}`}
                  key={recipe.id}
                >
                  <button type="button" className="recipe-card-open" disabled={busy} onClick={() => void open(recipe.id)} onKeyDown={(event) => {
                    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
                      event.preventDefault();
                      document.querySelector<HTMLElement>(`#recipe-result-${index + (event.key === 'ArrowDown' ? 1 : -1)} .recipe-card-open`)?.focus();
                    }
                  }}
                  >
                  <span className="badge">
                    {recipe.kind === 'food' ? t.food : t.beverage}
                  </span>
                  <h2 id={`recipe-title-${recipe.id}`}>{recipe.title}</h2>
                  <p>{recipe.description}</p>
                  <span>
                    {recipe.servings} {t.portions}
                  </span>
                  </button>
                  {!trash && <div className="recipe-actions">
                    <Button variant={recipe.favorite?'default':'outline'} aria-pressed={recipe.favorite} disabled={busy}
                      aria-label={recipe.favorite?t.favoriteRemove:t.favoriteAdd} aria-describedby={`recipe-title-${recipe.id}`}
                      onClick={()=>void updateOrganization({favorite:!recipe.favorite,toTry:recipe.toTry,rating:recipe.rating},recipe.id,recipe.organizationRevision)}>{recipe.favorite?'★':'☆'}</Button>
                    <Button variant={recipe.toTry?'default':'outline'} aria-pressed={recipe.toTry} disabled={busy}
                      aria-label={recipe.toTry?t.toTryRemove:t.toTryAdd} aria-describedby={`recipe-title-${recipe.id}`}
                      onClick={()=>void updateOrganization({favorite:recipe.favorite,toTry:!recipe.toTry,rating:recipe.rating},recipe.id,recipe.organizationRevision)}>{t.toTry}</Button>
                  </div>}
                </div>
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
      {collectionDelete && <ConfirmDialog title={t.collectionDeleteConfirm} description={t.collectionDeleteHelp}
        confirm={t.collectionDelete} cancel={t.cancel} busy={busy} error={error}
        onCancel={()=>setCollectionDelete(null)} onConfirm={()=>void removeCollection()} />}
    </section>
  );
}
