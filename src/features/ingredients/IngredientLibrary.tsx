import { useEffect, useRef, useState } from 'react';
import { Search, Plus, ChevronRight, Leaf, Star } from 'lucide-react';
import { ingredientLibraryClient, type IngredientLibraryClient } from '../../shared/api/ingredientLibrary';
import type { RecipeClient } from '../../shared/api/recipes';
import type { LibraryDetail, LibraryPage, LibraryQuery } from '../../shared/contracts/ingredientLibrary';
import { normalizeError } from '../../shared/api/client';
import { errorMessage, messages } from '../../shared/i18n';
import { ingredientsTr as t } from '../../shared/i18n/ingredients';
import { recipesTr } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { IngredientDetail } from './IngredientDetail';
import { PersonalIngredientEditor } from './PersonalIngredientEditor';
import { CatalogPresentationEditor } from './CatalogPresentationEditor';
import { orderedCategoryOptions } from './categoryTree';
export function IngredientLibrary({ client, library = ingredientLibraryClient, registerLeaveGuard }: {
    client: RecipeClient;
    library?: IngredientLibraryClient;
    registerLeaveGuard?: (guard:(()=>Promise<boolean>)|null)=>void;
}) {
    const [query, setQuery] = useState<LibraryQuery>({ search: '', origin: 'all', categoryId: null, offset: 0, limit: 30, view:null, uncategorized:false });
    const [generation, setGeneration] = useState(0);
    const [result, setResult] = useState<{
        key: string;
        page?: LibraryPage;
        error?: string;
    }>();
    const [target, setTarget] = useState<{
        id: string;
        origin: 'catalog' | 'personal';
    }>();
    const [detail, setDetail] = useState<{
        key: string;
        value?: LibraryDetail;
        error?: string;
    }>();
    const [editing, setEditing] = useState(false);
    const [editingCatalog, setEditingCatalog] = useState(false);
    const [candidates, setCandidates] = useState<Awaited<ReturnType<IngredientLibraryClient['candidates']>>>([]);
    const [duplicateAcknowledged, setDuplicateAcknowledged] = useState(false);
    const [candidateError, setCandidateError] = useState('');
    const [creating, setCreating] = useState(false);
    const [newName, setNewName] = useState('');
    const [createError, setCreateError] = useState('');
    const [actionError, setActionError] = useState('');
    const [busy, setBusy] = useState(false);
    const [createLeaving, setCreateLeaving] = useState(false);
    const createLeaveResolver = useRef<((allowed:boolean)=>void)|null>(null);
    const mutation = useRef(false);
    const key = JSON.stringify([query, generation]);
    useEffect(() => {
      if (!creating || newName.trim().length < 2) return;
      let alive = true;
      const timer = setTimeout(() => { void library.candidates(newName).then(value => { if (alive) { setCandidates(value); setCandidateError(''); } }, e => { if (alive) setCandidateError(errorMessage(normalizeError(e).messageKey)); }); }, 150);
      return () => { alive = false; clearTimeout(timer); };
    }, [creating, newName, library]);
    async function toggleFavorite(id:string, origin:'catalog'|'personal', favorite:boolean) {
      if (mutation.current) return;
      mutation.current = true;
      try { await library.favorite(id, origin, !favorite); setActionError(''); setGeneration(g => g + 1); }
      catch(e) { setActionError(errorMessage(normalizeError(e).messageKey)); }
      finally { mutation.current = false; }
    }
    const detailKey = JSON.stringify([target, generation]);
    useEffect(() => {
        let alive = true;
        const timer = setTimeout(() => { void library.browse(query).then(page => { if (alive)
            setResult({ key, page }); }, e => { if (alive)
            setResult({ key, error: errorMessage(normalizeError(e).messageKey) }); }); }, 150);
        return () => { alive = false; clearTimeout(timer); };
    }, [query, key, library]);
    useEffect(() => {
        if (!target)
            return;
        let alive = true;
        void library.detail(target.id, target.origin).then(value => { if (alive)
            setDetail({ key: detailKey, value }); }, e => { if (alive)
            setDetail({ key: detailKey, error: errorMessage(normalizeError(e).messageKey) }); });
        return () => { alive = false; };
    }, [target, detailKey, library]);
    function filter(patch: Partial<LibraryQuery>) { setQuery(q => ({ ...q, ...patch, offset: patch.offset ?? 0 })); }
    const askCreateLeave = () => creating && newName.trim() ? new Promise<boolean>(resolve => { createLeaveResolver.current = resolve; setCreateLeaving(true); }) : Promise.resolve(true);
    useEffect(() => {
      if (target || !creating) return;
      registerLeaveGuard?.(askCreateLeave);
      return () => registerLeaveGuard?.(null);
    });
    async function create(): Promise<boolean> {
        if (mutation.current || !newName.trim())
            return false;
        mutation.current = true;
        setBusy(true);
        setCreateError('');
        try {
            const currentCandidates=await library.candidates(newName);
            setCandidates(currentCandidates);
            if (currentCandidates.some(c => c.reason === 'exact_name' || c.reason === 'possible_synonym')) {
              if (createLeaving) setCreateError(t.strongMatchHelp);
              return false;
            }
            if (currentCandidates.length && !duplicateAcknowledged) {
              if (createLeaving) setCreateError(t.duplicateWarning);
              return false;
            }
            const value = await client.createIngredient(newName);
            // Existing create semantics reuse exact canonical/personal names. Resolve
            // actual origin rather than labeling every successful response personal.
            let saved: LibraryDetail;
            try {
                saved = await library.detail(value.id, 'personal');
            }
            catch (e) {
                if (normalizeError(e).code !== 'NOT_FOUND')
                    throw e;
                saved = await library.detail(value.id, 'catalog');
            }
            setGeneration(g => g + 1);
            setTarget({ id: saved.item.id, origin: saved.item.origin });
            setCreating(false);
            setNewName('');
            setCandidates([]);
            setDuplicateAcknowledged(false);
            return true;
        }
        catch (e) {
            setCreateError(errorMessage(normalizeError(e).messageKey));
            return false;
        }
        finally {
            mutation.current = false;
            setBusy(false);
        }
    }
    const page = result?.page;
    const loading = result?.key !== key;
    const chosen = detail?.key === detailKey ? detail.value : undefined;
    const categories = page?.categories ?? [];
    const categoryOptions = orderedCategoryOptions(categories);
    return <section className="ingredient-library">
    <div className="page-heading"><div><h1>{t.title}</h1><p>{t.intro}</p></div>{!target && <Button onClick={() => { setCreating(true); setCreateError(''); }}><Plus size={17} aria-hidden/>{t.create}</Button>}</div>
    {actionError && <p role="alert" className="recipe-error">{actionError}</p>}
    {target ? <>
      {!editing && !editingCatalog && <Button variant="outline" onClick={() => setTarget(undefined)}>{t.back}</Button>}
        {detail?.key === detailKey && detail.error ? <div role="alert" className="recipe-error"><p>{detail.error}</p><Button onClick={() => setGeneration(g => g + 1)}>{messages.retry}</Button></div> : !chosen ? <p role="status">{t.loading}</p> : editing ? <PersonalIngredientEditor item={chosen.item} categories={categories} client={client} library={library} onSelectCandidate={candidate => { setEditing(false); setTarget(candidate); }} registerLeaveGuard={registerLeaveGuard} onDone={() => { setEditing(false); setTarget(undefined); setGeneration(g => g + 1); }}/> : editingCatalog ? <CatalogPresentationEditor detail={chosen} client={client} library={library} registerLeaveGuard={registerLeaveGuard} onDone={() => { setEditingCatalog(false); setGeneration(g => g + 1); }}/> : <>
        {chosen.item.origin === 'personal' && <Button className="ingredient-edit-action" onClick={() => setEditing(true)}>{recipesTr.edit}</Button>}
        {chosen.item.origin === 'catalog' && chosen.item.recipeId && <Button variant="outline" onClick={() => setEditingCatalog(true)}>{t.editCatalog}</Button>}
        <Button variant="outline" aria-label={chosen.item.favorite ? t.favoriteRemove : t.favoriteAdd} onClick={() => void toggleFavorite(chosen.item.id, chosen.item.origin as 'catalog'|'personal', chosen.item.favorite)}><Star size={16} fill={chosen.item.favorite ? 'currentColor' : 'none'} aria-hidden/>{chosen.item.favorite ? t.favoriteRemove : t.favoriteAdd}</Button>
        <IngredientDetail detail={chosen}/>
      </>}
    </> : <>
      {creating && <form className="panel recipe-form" onSubmit={e => { e.preventDefault(); void create(); }}><p>{t.createHelp}</p><label>{t.newName}<input autoFocus required maxLength={200} value={newName} disabled={busy} onChange={e => { setNewName(e.target.value); setCandidates([]); setDuplicateAcknowledged(false); }}/></label>{candidateError && <p role="alert" className="recipe-error">{candidateError}</p>}{candidates.length>0 && <div role="status"><p>{t.duplicateWarning}</p><ul>{candidates.map(c => <li key={c.origin+c.id}>{c.name} · {t[c.origin]} · {t[c.reason as keyof typeof t] ?? t.ambiguous} <Button type="button" variant="outline" onClick={() => { setTarget({ id:c.id, origin:c.origin }); setCreating(false); }}>{t.useExisting}</Button></li>)}</ul>{candidates.some(c => c.reason === 'exact_name' || c.reason === 'possible_synonym') ? <p>{t.strongMatchHelp}</p> : <Button type="button" variant="outline" onClick={() => setDuplicateAcknowledged(true)}>{t.createAnyway}</Button>}</div>}{createError && <p role="alert" className="recipe-error">{createError}</p>}<div className="recipe-actions"><Button disabled={busy || !newName.trim()}>{t.create}</Button><Button type="button" variant="outline" disabled={busy} onClick={() => {void askCreateLeave().then(allowed=>{if(allowed)setCreating(false);});}}>{recipesTr.cancel}</Button></div>{createLeaving && <ConfirmDialog title={t.leaveTitle} description={t.leaveHelp} confirm={t.leaveSave} cancel={t.continue} busy={busy} error={createError || candidateError} extra={{label:t.leaveDiscard,onClick:()=>{setCreateLeaving(false);createLeaveResolver.current?.(true);createLeaveResolver.current=null;}}} onConfirm={()=>{void create().then(ok=>{if(ok){setCreateLeaving(false);createLeaveResolver.current?.(true);createLeaveResolver.current=null;}});}} onCancel={()=>{setCreateLeaving(false);createLeaveResolver.current?.(false);createLeaveResolver.current=null;}} />}</form>}
      <div className="ingredient-views" role="group" aria-label={t.title}>{(['all', 'catalog', 'personal'] as const).map(origin => <Button key={origin} variant={query.origin === origin ? 'default' : 'outline'} aria-pressed={query.origin === origin} onClick={() => filter({ origin })}>{t[origin]} <span>{page ? origin === 'all' ? page.catalogCount + page.personalCount : origin === 'catalog' ? page.catalogCount : page.personalCount : '…'}</span></Button>)}</div>
      <div className="ingredient-views" role="group" aria-label={t.title}>{(['favorites','recent'] as const).map(view => <Button key={view} variant={query.view === view ? 'default' : 'outline'} aria-pressed={query.view === view} onClick={() => filter({ view: query.view === view ? null : view })}>{view === 'favorites' && <Star size={16} aria-hidden/>}{t[view]}</Button>)}</div>
      {query.view === 'recent' && <p className="muted">{t.recentHelp}</p>}
      <div className="panel ingredient-filters"><label className="ingredient-search-label"><span><Search size={16} aria-hidden/>{t.search}</span><input type="search" maxLength={200} value={query.search} onChange={e => filter({ search: e.target.value })} aria-describedby="ingredient-search-help"/></label><label>{t.categories}<select disabled={loading} value={query.uncategorized ? 'uncategorized' : query.categoryId ?? ''} onChange={e => { const value=e.currentTarget.value; filter({ categoryId: value && value !== 'uncategorized' ? value : null, uncategorized:value === 'uncategorized' }); }}><option value="">{t.allCategories}</option><option value="uncategorized">{t.uncategorized}</option>{categoryOptions.map(c => <option key={c.id} value={c.id}>{c.label} ({c.count})</option>)}</select></label><Button variant="ghost" onClick={() => setQuery({ search: '', origin: 'all', categoryId: null, offset: 0, limit: 30, view:null, uncategorized:false })}>{t.reset}</Button><p id="ingredient-search-help">{t.searchHelp} {t.categoryHelp}</p></div>
      {loading ? <p role="status">{t.loading}</p> : result?.error ? <div role="alert" className="recipe-error"><p>{result.error}</p><Button onClick={() => setGeneration(g => g + 1)}>{messages.retry}</Button></div> : page && <>
        <p role="status" className="ingredient-result-count">{page.total} {t.results}</p>
        {!page.items.length ? <div className="panel ingredient-empty"><Leaf aria-hidden/><h2>{query.categoryId ? t.emptyCategory : t.empty}</h2><p>{t.emptyHelp}</p><Button variant="outline" onClick={() => { setCreating(true); setNewName(query.search); }}>{t.create}</Button></div> : <ul className="ingredient-grid">{page.items.map(item => <li key={`${item.origin}:${item.id}`} className="ingredient-card-container"><Button type="button" variant="outline" className="ingredient-card-favorite" aria-label={`${item.name}: ${item.favorite ? t.favoriteRemove : t.favoriteAdd}`} onClick={() => void toggleFavorite(item.id,item.origin as 'catalog'|'personal',item.favorite)}><Star size={16} fill={item.favorite ? 'currentColor' : 'none'} aria-hidden/></Button><button type="button" className="ingredient-card" onClick={() => { setTarget({ id: item.id, origin: item.origin }); setEditing(false); }}><span className="ingredient-badge">{t[item.origin]}</span><strong>{item.name}</strong>{item.englishName && <span className="ingredient-secondary">{item.englishName}</span>}<span className="ingredient-category">{item.categories[0]?.tr ?? t.unknownCategory}</span>{query.view === 'recent' && item.lastUsedAt && <span className="ingredient-secondary">{new Date(item.lastUsedAt).toLocaleString('tr-TR')}</span>}<span className="ingredient-card-footer">{item.preferredUnit ?? t.unknown}<ChevronRight size={16} aria-hidden/></span></button></li>)}</ul>}
        <nav className="ingredient-pagination" aria-label={t.page}><Button variant="outline" disabled={page.offset === 0} onClick={() => filter({ offset: Math.max(0, page.offset - page.limit) })}>{t.previous}</Button><span>{t.page} {Math.floor(page.offset / page.limit) + 1} / {Math.max(1, Math.ceil(page.total / page.limit))}</span><Button variant="outline" disabled={page.offset + page.items.length >= page.total} onClick={() => filter({ offset: page.offset + page.limit })}>{t.next}</Button></nav>
      </>}
    </>}
  </section>;
}
