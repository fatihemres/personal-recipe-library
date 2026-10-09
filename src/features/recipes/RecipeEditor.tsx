import { useEffect, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import {
  recipeInputSchema,
  type Recipe,
  type RecipeInput,
  type Unit,
  type Draft,
} from '../../shared/contracts/recipe';
import { Button } from '../../shared/ui/button';
import { RecipeSteps } from './RecipeSteps';
import { RecipeIngredients } from './RecipeIngredients';
import { RecipeDetails } from './RecipeDetails';
import { useDurableDraft } from './useDurableDraft';
import { IngredientSearchControl } from '../ingredients';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
const nullable = (value: string) => (value.trim() ? value : null);
const decimal = (value: string) => value.trim().replace(',', '.');
export function RecipeEditor({
  recipe,
  client,
  onSaved,
  onCancel,
  recovered,
}: {
  recipe: Recipe | null;
  client: RecipeClient;
  onSaved: (recipe: Recipe) => void;
  onCancel: () => void;
  recovered?: Draft;
}) {
  const [draft, setDraft] = useState<RecipeInput>(
    () =>
      recovered?.input ??
      (recipe
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
          }),
  );
  const initial = useRef(JSON.stringify(draft));
  const dirty = !!recovered || JSON.stringify(draft) !== initial.current;
  const durable = useDurableDraft(client, draft, dirty, recovered);
  const [names, setNames] = useState<Record<string, string>>(
    recovered?.ingredientNames ?? recipe?.ingredientNames ?? {},
  );
  const [units, setUnits] = useState<Unit[]>([]);

  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);
  const [adding, setAdding] = useState(false);
  const [notice, setNotice] = useState('');
  const [discard, setDiscard] = useState(false);
  const pending = useRef(false);
  const [conflict, setConflict] = useState(false);
  const [latestSaved, setLatestSaved] = useState<Recipe>();
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
  function update<K extends keyof RecipeInput>(key: K, value: RecipeInput[K]) {
    setDraft((d) => ({ ...d, [key]: value }));
  }
  async function save(asCopy = false) {
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
      const result = await durable.session.commit(parsed.data, asCopy);
      initial.current = JSON.stringify(draft);
      onSaved(result);
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
      setConflict(
        ['CONFLICT', 'DRAFT_CONFLICT', 'NOT_FOUND'].includes(
          normalizeError(e).code,
        ),
      );
    } finally {
      pending.current = false;
      setSaving(false);
    }
  }
  async function leave(erase: boolean) {
    if (pending.current) return;
    pending.current = true;
    setSaving(true);
    try {
      if (erase) await durable.session.discard();
      else await durable.flush();
      onCancel();
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      pending.current = false;
      setSaving(false);
    }
  }
  async function viewLatest() {
    try {
      setLatestSaved(await client.get(draft.id));
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    }
  }
  function selectIngredient(ingredient: {
    id: string;
    name: string;
    preferredUnit?: string | null;
  }) {
    setNames((previous) => ({ ...previous, [ingredient.id]: ingredient.name }));
    setDraft((current) => ({
      ...current,
      ingredients: [
        ...current.ingredients,
        {
          id: crypto.randomUUID(),
          ingredientId: ingredient.id,
          quantity: null,
          unitCode: ingredient.preferredUnit ?? 'g',
          note: null,
        },
      ],
    }));
    setNotice(t.ingredientSaved);
  }
  return (
    <section className="recipe-editor">
      <div className="page-heading">
        <div>
          <p className="eyebrow">{t.heading}</p>
          <h1>{draft.expectedRevision !== null ? t.edit : t.newRecipe}</h1>
        </div>
      </div>
      {error && (
        <p role="alert" className="recipe-error">
          {error}
        </p>
      )}
      {notice && <p role="status">{notice}</p>}
      <p role="status">
        {dirty ? t.unsaved : t.editorClean} ·{' '}
        {durable.status === 'pending'
          ? t.draftPending
          : durable.status === 'saving'
            ? t.draftSaving
            : durable.status === 'saved'
              ? t.draftSaved
              : durable.status === 'failed'
                ? t.draftFailed
                : ''}
      </p>
      {durable.failure ? (
        <div className="recipe-error" role="alert">
          <p>{errorMessage(normalizeError(durable.failure).messageKey)}</p>
          <Button
            variant="outline"
            onClick={() => void durable.flush().catch(() => {})}
          >
            {t.retry}
          </Button>
        </div>
      ) : null}
      {(conflict ||
        (recovered?.input.expectedRevision && !recovered.recipeId)) && (
        <p role="alert">{t.conflictHelp}</p>
      )}

      {conflict && (
        <Button variant="outline" onClick={() => void viewLatest()}>
          {t.viewLatest}
        </Button>
      )}
      {latestSaved && (
        <details open>
          <summary>{t.viewLatest}</summary>
          <RecipeDetails recipe={latestSaved} />
        </details>
      )}
      <form
        noValidate
        onSubmit={(e) => {
          e.preventDefault();
          void save();
        }}
      >
        <fieldset
          disabled={saving || durable.closing}
          className="recipe-fieldset"
        >
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
            names={names}
            search={
              <IngredientSearchControl
                personalOnly={false}
                client={client}
                onSelect={selectIngredient}
                onBusy={setAdding}
              />
            }
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
              onClick={() => (dirty ? setDiscard(true) : void leave(true))}
            >
              {t.cancel}
            </Button>
            {dirty && (
              <Button
                type="button"
                variant="outline"
                onClick={() => void leave(false)}
              >
                {t.keepDraft}
              </Button>
            )}
            {(conflict || recovered?.input.expectedRevision) && (
              <Button
                type="button"
                variant="outline"
                onClick={() => void save(true)}
              >
                {t.saveAsNew}
              </Button>
            )}
          </div>
        </fieldset>
      </form>
      {discard && (
        <ConfirmDialog
          title={t.confirmDiscard}
          description={t.discardHelp}
          confirm={t.discard}
          cancel={t.continueEditing}
          extra={{
            label: t.save,
            onClick: () => {
              setDiscard(false);
              void save();
            },
          }}
          onCancel={() => setDiscard(false)}
          onConfirm={() => void leave(true)}
        />
      )}
    </section>
  );
}
