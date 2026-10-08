import { useEffect, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import {
  recipeInputSchema,
  type Recipe,
  type RecipeInput,
  type Ingredient,
  type Unit,
} from '../../shared/contracts/recipe';
import { Button } from '../../shared/ui/button';
import { RecipeSteps } from './RecipeSteps';
import { RecipeIngredients } from './RecipeIngredients';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
const nullable = (value: string) => (value.trim() ? value : null);
const decimal = (value: string) => value.trim().replace(',', '.');
export function RecipeEditor({
  recipe,
  client,
  onSaved,
  onCancel,
}: {
  recipe: Recipe | null;
  client: RecipeClient;
  onSaved: (recipe: Recipe) => void;
  onCancel: () => void;
}) {
  const [draft, setDraft] = useState<RecipeInput>(() =>
    recipe
      ? {
          id: recipe.id,
          expectedRevision: recipe.revision,
          title: recipe.title,
          description: recipe.description,
          kind: recipe.kind,
          servings: recipe.servings,
          prepMinutes: recipe.prepMinutes,
          cookMinutes: recipe.cookMinutes,
          notes: recipe.notes,
          ingredients: recipe.ingredients,
          steps: recipe.steps,
        }
      : {
          id: crypto.randomUUID(),
          expectedRevision: null,
          title: '',
          description: null,
          kind: 'food',
          servings: '1',
          prepMinutes: null,
          cookMinutes: null,
          notes: null,
          ingredients: [],
          steps: [],
        },
  );
  const initial = useRef(JSON.stringify(draft));
  const dirty = JSON.stringify(draft) !== initial.current;
  const [names, setNames] = useState<Record<string, string>>(
    recipe?.ingredientNames ?? {},
  );
  const [units, setUnits] = useState<Unit[]>([]);
  const [ingredients, setIngredients] = useState<Ingredient[]>([]);
  const [query, setQuery] = useState('');
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);
  const [adding, setAdding] = useState(false);
  const [notice, setNotice] = useState('');
  const [discard, setDiscard] = useState(false);
  const pending = useRef(false);
  const searchGeneration = useRef(0);
  useEffect(() => {
    let active = true;
    void client
      .units()
      .then((v) => {
        if (active) setUnits(v);
      })
      .catch((e) => {
        if (active) setError(errorMessage(normalizeError(e).messageKey));
      });
    return () => {
      active = false;
    };
  }, [client]);
  useEffect(() => {
    let active = true;
    const generation = ++searchGeneration.current;
    const timer = setTimeout(() => {
      void client
        .searchIngredients(query)
        .then((v) => {
          if (active && generation === searchGeneration.current) {
            setIngredients(v);
            setNames((previous) => ({
              ...previous,
              ...Object.fromEntries(v.map((i) => [i.id, i.name])),
            }));
          }
        })
        .catch((e) => {
          if (active) setError(errorMessage(normalizeError(e).messageKey));
        });
    }, 150);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [client, query]);
  useEffect(() => {
    const guard = (e: BeforeUnloadEvent) => {
      if (dirty) {
        e.preventDefault();
        e.returnValue = '';
      }
    };
    window.addEventListener('beforeunload', guard);
    return () => window.removeEventListener('beforeunload', guard);
  }, [dirty]);
  function update<K extends keyof RecipeInput>(key: K, value: RecipeInput[K]) {
    setDraft((d) => ({ ...d, [key]: value }));
  }
  async function save() {
    if (pending.current) return;
    const parsed = recipeInputSchema.safeParse(draft);
    if (!parsed.success) {
      setError(t.invalid);
      return;
    }
    pending.current = true;
    setSaving(true);
    setError('');
    try {
      const result = await client.save(parsed.data);
      initial.current = JSON.stringify(draft);
      onSaved(result);
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      pending.current = false;
      setSaving(false);
    }
  }
  async function createIngredient() {
    if (adding || !query.trim()) return;
    setAdding(true);
    try {
      const result = await client.createIngredient(query);
      setNames((previous) => ({ ...previous, [result.id]: result.name }));
      setIngredients((items) => [
        result,
        ...items.filter((i) => i.id !== result.id),
      ]);
      setDraft((current) => ({
        ...current,
        ingredients: [
          ...current.ingredients,
          {
            id: crypto.randomUUID(),
            ingredientId: result.id,
            quantity: null,
            unitCode: 'g',
            note: null,
          },
        ],
      }));
      setNotice(t.ingredientSaved);
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      setAdding(false);
    }
  }
  return (
    <section className="recipe-editor">
      <div className="page-heading">
        <div>
          <p className="eyebrow">{t.heading}</p>
          <h1>{recipe ? t.edit : t.newRecipe}</h1>
        </div>
      </div>
      {error && (
        <p role="alert" className="recipe-error">
          {error}
        </p>
      )}
      {notice && <p role="status">{notice}</p>}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void save();
        }}
      >
        <fieldset disabled={saving} className="recipe-fieldset">
          <div className="panel recipe-form">
            <label>
              {t.title}
              <input
                autoFocus
                required
                maxLength={200}
                value={draft.title}
                onChange={(e) => update('title', e.target.value)}
              />
            </label>
            <label>
              {t.description}
              <textarea
                maxLength={10000}
                value={draft.description ?? ''}
                onChange={(e) =>
                  update('description', nullable(e.target.value))
                }
              />
            </label>
            <div className="form-grid">
              <label>
                {t.kind}
                <select
                  value={draft.kind}
                  onChange={(e) =>
                    update('kind', e.target.value as 'food' | 'beverage')
                  }
                >
                  <option value="food">{t.food}</option>
                  <option value="beverage">{t.beverage}</option>
                </select>
              </label>
              <label>
                {t.servings}
                <input
                  required
                  inputMode="decimal"
                  value={draft.servings}
                  onChange={(e) => update('servings', decimal(e.target.value))}
                />
              </label>
              {(['prepMinutes', 'cookMinutes'] as const).map((key) => (
                <label key={key}>
                  {key === 'prepMinutes' ? t.prep : t.cook}
                  <input
                    type="number"
                    min={0}
                    max={10080}
                    step={1}
                    value={draft[key] ?? ''}
                    onChange={(e) =>
                      update(
                        key,
                        e.target.value === '' ? null : Number(e.target.value),
                      )
                    }
                  />
                </label>
              ))}
            </div>
          </div>
          <RecipeIngredients
            lines={draft.ingredients}
            units={units}
            ingredients={ingredients}
            names={names}
            query={query}
            onQuery={setQuery}
            adding={adding}
            onCreate={() => void createIngredient()}
            onChange={(lines) => update('ingredients', lines)}
          />
          <RecipeSteps
            steps={draft.steps}
            onChange={(steps) => update('steps', steps)}
          />
          <section className="panel recipe-form">
            <label>
              {t.notes}
              <textarea
                maxLength={20000}
                value={draft.notes ?? ''}
                onChange={(e) => update('notes', nullable(e.target.value))}
              />
            </label>
          </section>
          <div className="recipe-actions">
            <Button type="submit" disabled={!units.length || adding}>
              {saving ? t.saving : t.save}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => (dirty ? setDiscard(true) : onCancel())}
            >
              {t.cancel}
            </Button>
          </div>
        </fieldset>
      </form>
      {discard && (
        <ConfirmDialog
          title={t.confirmDiscard}
          description={t.discardHelp}
          confirm={t.discard}
          cancel={t.cancel}
          onCancel={() => setDiscard(false)}
          onConfirm={onCancel}
        />
      )}
    </section>
  );
}
