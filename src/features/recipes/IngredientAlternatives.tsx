import { useState } from 'react';
import { createPortal } from 'react-dom';
import { alternativeSchema, decimalSchema, type RecipeInput, type Unit, type IngredientAlternative } from '../../shared/contracts/recipe';
import type { RecipeClient } from '../../shared/api/recipes';
import { IngredientSearchControl } from '../ingredients';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { alternativesTr as t } from '../../shared/i18n/ingredientAlternatives';

type Line = RecipeInput['ingredients'][number];
export function IngredientAlternatives({ line, units, names, client, onChange, onName, totalAlternatives }: {
  line: Line;
  totalAlternatives: number;
  units: Unit[];
  names: Record<string, string>;
  client: RecipeClient;
  onChange: (line: Line) => void;
  onName: (id: string, name: string) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [adding, setAdding] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');
  const [using, setUsing] = useState<IngredientAlternative | null>(null);
  const [quantity, setQuantity] = useState('');
  const [unit, setUnit] = useState('');
  const [reviewed, setReviewed] = useState(false);
  const alternatives = line.alternatives ?? [];

  function change(items: IngredientAlternative[]) { onChange({ ...line, alternatives: items }); }
  function move(index: number, direction: number) {
    const items = [...alternatives];
    [items[index], items[index + direction]] = [items[index + direction], items[index]];
    change(items);
  }
  function selectIngredient(ingredient: {id: string; name: string}) {
    if (ingredient.id === line.ingredientId ||
      alternatives.some(a => a.id !== editing && a.ingredientId === ingredient.id) ||
      (!editing && (alternatives.length >= 10 || totalAlternatives >= 1000))) {
      setError(t.invalid);
      return;
    }
    setError('');
    onName(ingredient.id, ingredient.name);
    if (editing) {
      const changed = alternatives.some(a => a.id === editing && a.ingredientId !== ingredient.id);
      change(alternatives.map(a => a.id !== editing || a.ingredientId === ingredient.id
        ? a : { ...a, ingredientId: ingredient.id, quantity: null, unitCode: null }));
      setNotice(changed ? t.changed : '');
    } else {
      change([...alternatives, { id: crypto.randomUUID(), ingredientId: ingredient.id, quantity: null, unitCode: null }]);
    }
    setAdding(false);
    setEditing(null);
  }
  function startSearch(id: string | null) {
    setEditing(id);
    setAdding(true);
    setError('');
    setNotice('');
  }
  function startUse(alt: IngredientAlternative) {
    setAdding(false);
    setEditing(null);
    setUsing(alt);
    setQuantity(alt.quantity ?? '');
    setUnit(alt.unitCode ?? '');
    setReviewed(false);
  }
  function applyToDraft() {
    if (!using || !alternatives.some(a => a.id === using.id)) return;
    const selected = alternativeSchema.safeParse({ ...using, quantity: quantity || null, unitCode: unit });
    if (!selected.success || !unit || (!quantity && !reviewed)) return;
    // No IPC here: the authoritative write remains the editor's deliberate Save.
    // The selected row becomes primary; the former primary is not inferred as an alternative.
    onChange({ ...line, ingredientId: using.ingredientId, quantity: quantity || null, unitCode: unit,
      alternatives: alternatives.filter(a => a.id !== using.id) });
    setUsing(null);
  }
  const useDisabled = !unit || (!quantity && !reviewed) || (!!quantity && !decimalSchema.safeParse(quantity).success);
  return <details className="ingredient-alternatives" onToggle={e => setExpanded(e.currentTarget.open)}>
    <summary>{t.title} · {alternatives.length}</summary>
    {expanded && <section aria-label={t.title}>
      <p>{t.help}</p>
      {error && <p role="alert">{error}</p>}
      {notice && <p role="status">{notice}</p>}
      {!alternatives.length && <p>{t.empty}</p>}
      <ol className="alternative-list">{alternatives.map((alt, index) => <li key={alt.id}>
        <strong>{names[alt.ingredientId] ?? alt.ingredientId}</strong>
        <div className="form-grid">
          <label>{t.quantity}<input aria-label={`${index + 1}. ${t.quantity}`} inputMode="decimal" maxLength={100}
            value={alt.quantity ?? ''} aria-invalid={alt.quantity !== null && !decimalSchema.safeParse(alt.quantity).success}
            onChange={e => change(alternatives.map(a => a.id === alt.id
              ? { ...a, quantity: e.target.value.trim() ? e.target.value.replace(',', '.') : null } : a))} /></label>
          <label>{t.unit}<select aria-label={`${index + 1}. ${t.unit}`} value={alt.unitCode ?? ''}
            onChange={e => change(alternatives.map(a => a.id === alt.id ? { ...a, unitCode: e.target.value || null } : a))}>
            <option value="">{t.unknownUnit}</option>{units.map(u => <option key={u.code}>{u.code}</option>)}
          </select></label>
        </div>
        <div className="recipe-actions">
          <Button type="button" variant="outline" aria-label={`${index + 1}. ${t.edit}`} onClick={() => startSearch(alt.id)}>{t.edit}</Button>
          <Button type="button" variant="outline" onClick={() => startUse(alt)}>{t.use}</Button>
          <Button type="button" variant="outline" aria-label={`${index + 1}. ${t.up}`} disabled={index === 0} onClick={() => move(index, -1)}>{t.up}</Button>
          <Button type="button" variant="outline" aria-label={`${index + 1}. ${t.down}`} disabled={index === alternatives.length - 1} onClick={() => move(index, 1)}>{t.down}</Button>
          <Button type="button" variant="ghost" aria-label={`${names[alt.ingredientId]} · ${t.remove}`} onClick={() => {
            change(alternatives.filter(a => a.id !== alt.id));
            if (editing === alt.id) { setAdding(false); setEditing(null); }
          }}>{t.remove}</Button>
        </div>
      </li>)}</ol>
      <Button type="button" variant="outline" aria-expanded={adding} disabled={alternatives.length >= 10 || totalAlternatives >= 1000}
        onClick={() => adding ? setAdding(false) : startSearch(null)}>{t.add}</Button>
      {adding && <>
        {editing && <p>{t.edit}: {names[alternatives.find(a => a.id === editing)?.ingredientId ?? '']}</p>}
        <IngredientSearchControl allowCreate={false} personalOnly={false} client={client} onSelect={selectIngredient} />
      </>}
      {/* Portal keeps review inputs outside the recipe form: Enter cannot submit Save. */}
      {using && createPortal(<ConfirmDialog title={t.review} description={t.reviewHelp} confirm={t.confirmUse} cancel={t.cancel}
        confirmDisabled={useDisabled} onCancel={() => setUsing(null)} onConfirm={applyToDraft}>
        <div className="recipe-form alternative-review">
          <p><strong>{names[using.ingredientId]}</strong></p>
          <div className="form-grid">
            <label>{t.quantity}<input inputMode="decimal" maxLength={19} value={quantity}
              onChange={e => { setQuantity(e.target.value.replace(',', '.')); setReviewed(false); }} /></label>
            <label>{t.unit}<select value={unit} onChange={e => setUnit(e.target.value)}>
              <option value="">{t.chooseUnit}</option>{units.map(u => <option key={u.code}>{u.code}</option>)}
            </select></label>
          </div>
          {!quantity && <label className="step-ingredient-choice"><input type="checkbox" checked={reviewed}
            onChange={e => setReviewed(e.target.checked)} /><span>{t.unknownReview}</span></label>}
        </div>
      </ConfirmDialog>, document.body)}
    </section>}
  </details>;
}
