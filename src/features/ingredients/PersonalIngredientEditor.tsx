import { useEffect, useRef, useState } from 'react';
import type { IngredientLibraryClient } from '../../shared/api/ingredientLibrary';
import type { RecipeClient } from '../../shared/api/recipes';
import type { LibraryItem, LibraryPage } from '../../shared/contracts/ingredientLibrary';
import type { Unit } from '../../shared/contracts/recipe';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as r } from '../../shared/i18n/recipes';
import { ingredientsTr as t } from '../../shared/i18n/ingredients';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { orderedCategoryOptions } from './categoryTree';
export function PersonalIngredientEditor({ item, categories, client, library, onDone, onSelectCandidate, registerLeaveGuard }: {
    item: LibraryItem;
    categories?: LibraryPage['categories'];
    client: RecipeClient;
    library: IngredientLibraryClient;
    onSelectCandidate: (target:{id:string;origin:'catalog'|'personal'})=>void;
    onDone: () => void;
    registerLeaveGuard?: (guard:(()=>Promise<boolean>)|null)=>void;
}) {
    const [name, setName] = useState(item.name);
    const [notes, setNotes] = useState(item.notes ?? '');
    const [unit, setUnit] = useState(item.preferredUnit ?? '');
    const [categoryIds, setCategoryIds] = useState<string[]>(item.ingredientType === null ? item.categories.map(c => c.id) : []);
    const [units, setUnits] = useState<Unit[]>([]);
    const [error, setError] = useState('');
    const [candidates, setCandidates] = useState<Awaited<ReturnType<IngredientLibraryClient['candidates']>>>([]);
    const [acknowledged, setAcknowledged] = useState(false);
    const [busy, setBusy] = useState(false);
    const [confirm, setConfirm] = useState<'discard' | 'delete'>();
    const [leaving, setLeaving] = useState(false);
    const resolver = useRef<((allowed:boolean)=>void)|null>(null);
    const pendingLeave = useRef<Promise<boolean>|null>(null);
    useEffect(() => () => {
      resolver.current?.(false);
      resolver.current = null;
    }, []);
    useEffect(() => {
        let alive = true;
        void client.units().then(v => { if (alive)
            setUnits(v); }, e => { if (alive)
            setError(errorMessage(normalizeError(e).messageKey)); });
        return () => { alive = false; };
    }, [client]);
    useEffect(() => {
      if (name.trim() === item.name || name.trim().length < 2) { setCandidates([]); return; }
      let active=true;
      const timer=setTimeout(() => { void library.candidates(name).then(value => { if (active) setCandidates(value.filter(v => v.id !== item.id)); },cause => { if (active) setError(errorMessage(normalizeError(cause).messageKey)); }); },150);
      return () => { active=false; clearTimeout(timer); };
    },[library,name,item.id,item.name]);
    const dirty = name !== item.name || notes !== (item.notes ?? '') || unit !== (item.preferredUnit ?? '') || (item.ingredientType === null && (categoryIds.length !== item.categories.length || categoryIds.some(id => !item.categories.some(c => c.id === id))));
    const askLeave = () => {
      if (pendingLeave.current) return pendingLeave.current;
      if (!dirty) return Promise.resolve(true);
      pendingLeave.current = new Promise<boolean>(resolve => {
        resolver.current = allowed => { pendingLeave.current = null; resolve(allowed); };
        setLeaving(true);
      });
      return pendingLeave.current;
    };
    useEffect(() => {
      registerLeaveGuard?.(askLeave);
      return () => registerLeaveGuard?.(null);
    });
    async function save(finish = true): Promise<boolean> {
        if (busy || item.origin !== 'personal' || item.revision === null)
            return false;
        setBusy(true);
        setError('');
        try {
            const matches=(await library.candidates(name)).filter(value => value.id!==item.id);
            setCandidates(matches);
            if (matches.length && !acknowledged) { setError(t.duplicateWarning); return false; }
            await client.editIngredient({ id: item.id, revision: item.revision, name, notes: notes || null, preferredUnit: unit || null, ...(item.ingredientType === null ? {categoryIds} : {}) });
            if (finish) onDone();
            return true;
        }
        catch (e) {
            setError(errorMessage(normalizeError(e).messageKey));
            return false;
        }
        finally {
            setBusy(false);
        }
    }
    async function remove() {
        if (busy || item.origin !== 'personal' || item.revision === null)
            return;
        setConfirm(undefined);
        setBusy(true);
        setError('');
        try {
            await client.deleteIngredient(item.id, item.revision);
            onDone();
        }
        catch (e) {
            setError(errorMessage(normalizeError(e).messageKey));
        }
        finally {
            setBusy(false);
        }
    }
    return <form className="panel recipe-form" onSubmit={e => { e.preventDefault(); void save(); }}>
    <h2>{r.personalIngredients}</h2><p>{r.ingredientEditHelp}</p>
    {error && <p role="alert" className="recipe-error">{error}</p>}
    {dirty && <p role="status">{r.unsaved}</p>}
    {candidates.length>0 && <div role="status"><p>{t.duplicateWarning}</p><ul>{candidates.map(value => <li key={value.origin+value.id}>{value.name} · {t[value.origin]} · {t[value.reason as keyof typeof t] ?? t.ambiguous} <Button type="button" variant="outline" onClick={() => { void askLeave().then(allowed => { if (allowed) onSelectCandidate({id:value.id,origin:value.origin}); }); }}>{t.useExisting}</Button></li>)}</ul><Button type="button" variant="outline" onClick={() => { setAcknowledged(true); setError(''); }}>{t.createAnyway}</Button></div>}
    <fieldset className="recipe-fieldset" disabled={busy}>
      <label>{r.ingredientName}<input required maxLength={200} value={name} onChange={e => { setName(e.target.value); setAcknowledged(false); }}/></label>
      <label>{r.notes}<textarea maxLength={2000} value={notes} onChange={e => setNotes(e.target.value)}/></label>
      <label>{r.preferredUnit}<select value={unit} onChange={e => setUnit(e.target.value)}><option value="">{r.noPreferredUnit}</option>{units.map(u => <option key={u.code}>{u.code}</option>)}</select></label>
      {item.ingredientType === null && <fieldset className="ingredient-category-assign"><legend>{t.personalCategories}</legend><p>{t.categoryAssignHelp}</p><div className="ingredient-category-options">{orderedCategoryOptions(categories ?? []).map(category => <label key={category.id}><input type="checkbox" checked={categoryIds.includes(category.id)} onChange={e => setCategoryIds(ids => e.target.checked ? [...ids,category.id] : ids.filter(id => id !== category.id))}/>{category.label}</label>)}</div></fieldset>}
      <div className="recipe-actions"><Button type="submit">{r.saveIngredient}</Button><Button type="button" variant="outline" onClick={() => dirty ? setConfirm('discard') : onDone()}>{r.cancel}</Button><Button type="button" variant="ghost" onClick={() => setConfirm('delete')}>{r.deleteIngredient}</Button></div>
    </fieldset>
    {leaving && <ConfirmDialog title={t.leaveTitle} description={t.leaveHelp} confirm={t.leaveSave} cancel={t.continue} busy={busy} error={error} extra={{label:t.leaveDiscard,onClick:()=>{setLeaving(false);resolver.current?.(true);resolver.current=null;}}} onConfirm={() => { void save(false).then(ok => { if (ok) { setLeaving(false); resolver.current?.(true); resolver.current=null; } }); }} onCancel={() => { setLeaving(false); resolver.current?.(false); resolver.current=null; }} />}
    {confirm && <ConfirmDialog title={confirm === 'discard' ? t.discardTitle : r.deleteIngredient} description={confirm === 'discard' ? t.discardHelp : r.ingredientDeleteHelp} confirm={confirm === 'discard' ? t.discard : r.deleteIngredient} cancel={t.continue} onConfirm={() => confirm === 'discard' ? onDone() : void remove()} onCancel={() => setConfirm(undefined)}/>}
  </form>;
}
