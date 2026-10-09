import { useEffect, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import type { LibraryItem } from '../../shared/contracts/ingredientLibrary';
import type { Unit } from '../../shared/contracts/recipe';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as r } from '../../shared/i18n/recipes';
import { ingredientsTr as t } from '../../shared/i18n/ingredients';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
export function PersonalIngredientEditor({ item, client, onDone }: {
    item: LibraryItem;
    client: RecipeClient;
    onDone: () => void;
}) {
    const [name, setName] = useState(item.name);
    const [notes, setNotes] = useState(item.notes ?? '');
    const [unit, setUnit] = useState(item.preferredUnit ?? '');
    const [units, setUnits] = useState<Unit[]>([]);
    const [error, setError] = useState('');
    const [busy, setBusy] = useState(false);
    const [confirm, setConfirm] = useState<'discard' | 'delete'>();
    useEffect(() => {
        let alive = true;
        void client.units().then(v => { if (alive)
            setUnits(v); }, e => { if (alive)
            setError(errorMessage(normalizeError(e).messageKey)); });
        return () => { alive = false; };
    }, [client]);
    const dirty = name !== item.name || notes !== (item.notes ?? '') || unit !== (item.preferredUnit ?? '');
    async function save() {
        if (busy || item.origin !== 'personal' || item.revision === null)
            return;
        setBusy(true);
        setError('');
        try {
            await client.editIngredient({ id: item.id, revision: item.revision, name, notes: notes || null, preferredUnit: unit || null });
            onDone();
        }
        catch (e) {
            setError(errorMessage(normalizeError(e).messageKey));
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
    <fieldset className="recipe-fieldset" disabled={busy}>
      <label>{r.ingredientName}<input required maxLength={200} value={name} onChange={e => setName(e.target.value)}/></label>
      <label>{r.notes}<textarea maxLength={2000} value={notes} onChange={e => setNotes(e.target.value)}/></label>
      <label>{r.preferredUnit}<select value={unit} onChange={e => setUnit(e.target.value)}><option value="">{r.noPreferredUnit}</option>{units.map(u => <option key={u.code}>{u.code}</option>)}</select></label>
      <div className="recipe-actions"><Button type="submit">{r.saveIngredient}</Button><Button type="button" variant="outline" onClick={() => dirty ? setConfirm('discard') : onDone()}>{r.cancel}</Button><Button type="button" variant="ghost" onClick={() => setConfirm('delete')}>{r.deleteIngredient}</Button></div>
    </fieldset>
    {confirm && <ConfirmDialog title={confirm === 'discard' ? t.discardTitle : r.deleteIngredient} description={confirm === 'discard' ? t.discardHelp : r.ingredientDeleteHelp} confirm={confirm === 'discard' ? t.discard : r.deleteIngredient} cancel={t.continue} onConfirm={() => confirm === 'discard' ? onDone() : void remove()} onCancel={() => setConfirm(undefined)}/>}
  </form>;
}
