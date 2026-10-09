import { useEffect, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import type { PersonalIngredient, Unit } from '../../shared/contracts/recipe';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { IngredientSearchControl } from './IngredientSearchControl';
export function IngredientManager({ client }: { client: RecipeClient }) {
  const [selected, setSelected] = useState<PersonalIngredient>();
  const [units, setUnits] = useState<Unit[]>([]);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [busy, setBusy] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [generation, setGeneration] = useState(0);
  useEffect(() => {
    let active = true;
    void client.units().then(
      (value) => {
        if (active) setUnits(value);
      },
      (e) => {
        if (active) setError(errorMessage(normalizeError(e).messageKey));
      },
    );
    return () => {
      active = false;
    };
  }, [client]);
  async function select(ingredient: { id: string; name: string }) {
    setError('');
    try {
      const result = await client.searchPersonal(ingredient.name);
      const saved = result.items.find((i) => i.id === ingredient.id);
      if (saved) {
        setSelected(saved);
        setNotice('');
      } else {
        setSelected(undefined);
        setNotice(t.catalogIngredientReused);
      }
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    }
  }
  async function save() {
    if (!selected || busy) return;
    setBusy(true);
    setError('');
    try {
      setSelected(
        await client.editIngredient({
          id: selected.id,
          revision: selected.revision,
          name: selected.name,
          notes: selected.notes,
          preferredUnit: selected.preferredUnit,
        }),
      );
      setGeneration((n) => n + 1);
      setNotice(t.ingredientUpdated);
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      setBusy(false);
    }
  }
  async function remove() {
    if (!selected || busy) return;
    setConfirm(false);
    setBusy(true);
    try {
      await client.deleteIngredient(selected.id, selected.revision);
      setSelected(undefined);
      setGeneration((n) => n + 1);
      setNotice(t.ingredientDeleted);
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section>
      <div className="page-heading">
        <div>
          <h1>{t.personalIngredients}</h1>
          <p>{t.ingredientHelp}</p>
        </div>
      </div>
      {error && (
        <p role="alert" className="recipe-error">
          {error}
        </p>
      )}
      {notice && <p role="status">{notice}</p>}
      <div className="panel recipe-form">
        <IngredientSearchControl
          personalOnly
          key={generation}
          client={client}
          onSelect={(ingredient) => void select(ingredient)}
        />
      </div>
      {selected && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void save();
          }}
          className="panel recipe-form"
        >
          <fieldset className="recipe-fieldset" disabled={busy}>
            <p>{t.ingredientEditHelp}</p>
            <label>
              {t.ingredientName}
              <input
                required
                maxLength={200}
                value={selected.name}
                onChange={(e) =>
                  setSelected({ ...selected, name: e.target.value })
                }
              />
            </label>
            <label>
              {t.notes}
              <textarea
                maxLength={2000}
                value={selected.notes ?? ''}
                onChange={(e) =>
                  setSelected({ ...selected, notes: e.target.value || null })
                }
              />
            </label>
            <label>
              {t.preferredUnit}
              <select
                value={selected.preferredUnit ?? ''}
                onChange={(e) =>
                  setSelected({
                    ...selected,
                    preferredUnit: e.target.value || null,
                  })
                }
              >
                <option value="">{t.noPreferredUnit}</option>
                {units.map((unit) => (
                  <option key={unit.code}>{unit.code}</option>
                ))}
              </select>
            </label>
            <div className="recipe-actions">
              <Button type="submit">{t.saveIngredient}</Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => setConfirm(true)}
              >
                {t.deleteIngredient}
              </Button>
            </div>
          </fieldset>
        </form>
      )}
      {confirm && (
        <ConfirmDialog
          title={t.deleteIngredient}
          description={t.ingredientDeleteHelp}
          confirm={t.deleteIngredient}
          cancel={t.cancel}
          onConfirm={() => void remove()}
          onCancel={() => setConfirm(false)}
        />
      )}
    </section>
  );
}
