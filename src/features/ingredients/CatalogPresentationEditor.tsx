import { useEffect, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import type { IngredientLibraryClient } from '../../shared/api/ingredientLibrary';
import type { LibraryDetail } from '../../shared/contracts/ingredientLibrary';
import type { Unit } from '../../shared/contracts/recipe';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { ingredientsTr as t } from '../../shared/i18n/ingredients';
import { recipesTr as r } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';

export function CatalogPresentationEditor({ detail, client, library, onDone, registerLeaveGuard }: {
  detail: LibraryDetail;
  client: RecipeClient;
  library: IngredientLibraryClient;
  onDone: () => void;
  registerLeaveGuard?: (guard:(()=>Promise<boolean>)|null)=>void;
}) {
  const [name, setName] = useState(detail.item.name);
  const [alias, setAlias] = useState(detail.personalAlias ?? '');
  const [notes, setNotes] = useState(detail.item.notes ?? '');
  const [unit, setUnit] = useState(detail.item.preferredUnit ?? '');
  const [units, setUnits] = useState<Unit[]>([]);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [leaving,setLeaving]=useState(false);
  const resolver=useRef<((allowed:boolean)=>void)|null>(null);
  const dirty=name!==detail.item.name || alias!==(detail.personalAlias??'') || notes!==(detail.item.notes??'') || unit!==(detail.item.preferredUnit??'');
  const askLeave=()=>dirty ? new Promise<boolean>(resolve=>{resolver.current=resolve;setLeaving(true);}) : Promise.resolve(true);
  useEffect(()=>{registerLeaveGuard?.(askLeave);return()=>registerLeaveGuard?.(null);});
  useEffect(() => {
    let active = true;
    void client.units().then(value => { if (active) setUnits(value); }, cause => { if (active) setError(errorMessage(normalizeError(cause).messageKey)); });
    return () => { active = false; };
  }, [client]);
  async function apply(reset: boolean, finish = true): Promise<boolean> {
    if (busy || detail.item.revision === null || !detail.item.recipeId) return false;
    setBusy(true); setError('');
    try {
      await library.presentation({
        id: detail.item.id, revision: detail.item.revision,
        name: reset || name.trim() === detail.canonicalTr ? null : name.trim(),
        alias: reset || !alias.trim() ? null : alias.trim(),
        preferredUnit: reset ? null : unit || null,
        notes: reset || !notes.trim() ? null : notes.trim(),
      });
      if (finish) onDone();
      return true;
    } catch (cause) { setError(errorMessage(normalizeError(cause).messageKey)); return false; }
    finally { setBusy(false); }
  }
  return <form className="panel recipe-form" onSubmit={event => { event.preventDefault(); void apply(false); }}>
    <h2>{t.editCatalog}</h2><p>{t.catalogEditHelp}</p>
    {error && <p role="alert" className="recipe-error">{error}</p>}
    {dirty && <p role="status">{r.unsaved}</p>}
    <fieldset className="recipe-fieldset" disabled={busy}>
      <label>{t.catalogName}<input required maxLength={200} value={name} onChange={event => setName(event.target.value)} /></label>
      <label>{t.personalAlias}<input maxLength={200} value={alias} onChange={event => setAlias(event.target.value)} /></label>
      <label>{t.unit}<select value={unit} onChange={event => setUnit(event.target.value)}><option value="">{t.unknown}</option>{units.filter(value => detail.dimensions.includes(value.dimension)).map(value => <option key={value.code} value={value.code}>{value.code}</option>)}</select></label>
      <label>{r.notes}<textarea maxLength={2000} value={notes} onChange={event => setNotes(event.target.value)} /></label>
      <div className="recipe-actions"><Button type="submit">{t.savePresentation}</Button><Button type="button" variant="outline" onClick={() => void apply(true)}>{t.resetPresentation}</Button><Button type="button" variant="ghost" onClick={() => { void askLeave().then(allowed=>{if(allowed)onDone();}); }}>{r.cancel}</Button></div>
    </fieldset>
    {leaving && <ConfirmDialog title={t.leaveTitle} description={t.leaveHelp} confirm={t.leaveSave} cancel={t.continue} busy={busy} error={error} extra={{label:t.leaveDiscard,onClick:()=>{setLeaving(false);resolver.current?.(true);resolver.current=null;}}} onConfirm={()=>{void apply(false,false).then(ok=>{if(ok){setLeaving(false);resolver.current?.(true);resolver.current=null;}});}} onCancel={()=>{setLeaving(false);resolver.current?.(false);resolver.current=null;}} />}
  </form>;
}
