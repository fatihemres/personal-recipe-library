import { useEffect, useRef, useState } from 'react';
import { Search, Plus, ChevronRight, Leaf } from 'lucide-react';
import { ingredientLibraryClient, type IngredientLibraryClient } from '../../shared/api/ingredientLibrary';
import type { RecipeClient } from '../../shared/api/recipes';
import type { LibraryDetail, LibraryPage, LibraryQuery } from '../../shared/contracts/ingredientLibrary';
import { normalizeError } from '../../shared/api/client';
import { errorMessage, messages } from '../../shared/i18n';
import { ingredientsTr as t } from '../../shared/i18n/ingredients';
import { recipesTr } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
import { IngredientDetail } from './IngredientDetail';
import { PersonalIngredientEditor } from './PersonalIngredientEditor';
export function IngredientLibrary({ client, library = ingredientLibraryClient }: {
    client: RecipeClient;
    library?: IngredientLibraryClient;
}) {
    const [query, setQuery] = useState<LibraryQuery>({ search: '', origin: 'all', categoryId: null, offset: 0, limit: 30 });
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
    const [creating, setCreating] = useState(false);
    const [newName, setNewName] = useState('');
    const [createError, setCreateError] = useState('');
    const [busy, setBusy] = useState(false);
    const mutation = useRef(false);
    const key = JSON.stringify([query, generation]);
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
    async function create() {
        if (mutation.current || !newName.trim())
            return;
        mutation.current = true;
        setBusy(true);
        setCreateError('');
        try {
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
        }
        catch (e) {
            setCreateError(errorMessage(normalizeError(e).messageKey));
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
    const categoryOptions: {
        id: string;
        label: string;
        count: number;
    }[] = [];
    function branch(parent: string | null, depth: number, visited: Set<string>) {
        for (const c of categories.filter(c => c.category.parentId === parent).sort((a, b) => a.category.tr.localeCompare(b.category.tr, 'tr'))) {
            if (visited.has(c.category.id))
                continue;
            visited.add(c.category.id);
            categoryOptions.push({ id: c.category.id, label: `${'— '.repeat(depth)}${c.category.tr}`, count: c.count });
            branch(c.category.id, depth + 1, visited);
        }
    }
    branch(null, 0, new Set());
    return <section className="ingredient-library">
    <div className="page-heading"><div><h1>{t.title}</h1><p>{t.intro}</p></div>{!target && <Button onClick={() => { setCreating(true); setCreateError(''); }}><Plus size={17} aria-hidden/>{t.create}</Button>}</div>
    {target ? <>
      {!editing && <Button variant="outline" onClick={() => setTarget(undefined)}>{t.back}</Button>}
      {detail?.key === detailKey && detail.error ? <div role="alert" className="recipe-error"><p>{detail.error}</p><Button onClick={() => setGeneration(g => g + 1)}>{messages.retry}</Button></div> : !chosen ? <p role="status">{t.loading}</p> : editing ? <PersonalIngredientEditor item={chosen.item} client={client} onDone={() => { setEditing(false); setTarget(undefined); setGeneration(g => g + 1); }}/> : <>
        {chosen.item.origin === 'personal' && <Button className="ingredient-edit-action" onClick={() => setEditing(true)}>{recipesTr.edit}</Button>}
        <IngredientDetail detail={chosen}/>
      </>}
    </> : <>
      {creating && <form className="panel recipe-form" onSubmit={e => { e.preventDefault(); void create(); }}><p>{t.createHelp}</p><label>{t.newName}<input autoFocus required maxLength={200} value={newName} disabled={busy} onChange={e => setNewName(e.target.value)}/></label>{createError && <p role="alert" className="recipe-error">{createError}</p>}<div className="recipe-actions"><Button disabled={busy || !newName.trim()}>{t.create}</Button><Button type="button" variant="outline" disabled={busy} onClick={() => setCreating(false)}>{recipesTr.cancel}</Button></div></form>}
      <div className="ingredient-views" role="group" aria-label={t.title}>{(['all', 'catalog', 'personal'] as const).map(origin => <Button key={origin} variant={query.origin === origin ? 'default' : 'outline'} aria-pressed={query.origin === origin} onClick={() => filter({ origin })}>{t[origin]} <span>{page ? origin === 'all' ? page.catalogCount + page.personalCount : origin === 'catalog' ? page.catalogCount : page.personalCount : '…'}</span></Button>)}</div>
      <div className="panel ingredient-filters"><label className="ingredient-search-label"><span><Search size={16} aria-hidden/>{t.search}</span><input type="search" maxLength={200} value={query.search} onChange={e => filter({ search: e.target.value })} aria-describedby="ingredient-search-help"/></label><label>{t.categories}<select disabled={loading} value={query.categoryId ?? ''} onChange={e => filter({ categoryId: e.target.value || null })}><option value="">{t.allCategories}</option>{categoryOptions.map(c => <option key={c.id} value={c.id}>{c.label} ({c.count})</option>)}</select></label><Button variant="ghost" onClick={() => setQuery({ search: '', origin: 'all', categoryId: null, offset: 0, limit: 30 })}>{t.reset}</Button><p id="ingredient-search-help">{t.searchHelp}</p></div>
      {loading ? <p role="status">{t.loading}</p> : result?.error ? <div role="alert" className="recipe-error"><p>{result.error}</p><Button onClick={() => setGeneration(g => g + 1)}>{messages.retry}</Button></div> : page && <>
        <p role="status" className="ingredient-result-count">{page.total} {t.results}</p>
        {!page.items.length ? <div className="panel ingredient-empty"><Leaf aria-hidden/><h2>{t.empty}</h2><p>{t.emptyHelp}</p><Button variant="outline" onClick={() => { setCreating(true); setNewName(query.search); }}>{t.create}</Button></div> : <ul className="ingredient-grid">{page.items.map(item => <li key={`${item.origin}:${item.id}`}><button type="button" className="ingredient-card" onClick={() => { setTarget({ id: item.id, origin: item.origin }); setEditing(false); }}><span className="ingredient-badge">{t[item.origin]}</span><strong>{item.name}</strong>{item.englishName && <span className="ingredient-secondary">{item.englishName}</span>}<span className="ingredient-category">{item.categories[0]?.tr ?? t.unknownCategory}</span><span className="ingredient-card-footer">{item.preferredUnit ?? t.unknown}<ChevronRight size={16} aria-hidden/></span></button></li>)}</ul>}
        <nav className="ingredient-pagination" aria-label={t.page}><Button variant="outline" disabled={page.offset === 0} onClick={() => filter({ offset: Math.max(0, page.offset - page.limit) })}>{t.previous}</Button><span>{t.page} {Math.floor(page.offset / page.limit) + 1} / {Math.max(1, Math.ceil(page.total / page.limit))}</span><Button variant="outline" disabled={page.offset + page.items.length >= page.total} onClick={() => filter({ offset: page.offset + page.limit })}>{t.next}</Button></nav>
      </>}
    </>}
  </section>;
}
