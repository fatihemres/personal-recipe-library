import { stepDetailsIssues, type RecipeInput, type StepDetails } from '../../shared/contracts/recipe';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';

const emptyDetails: StepDetails = {
  formatVersion: 1, title: null, durationMinutes: null, temperatureValue: null,
  temperatureUnit: null, heatLevel: null, equipment: null, technique: null, tips: null,
};
export function RecipeSteps({ steps, onChange, ingredients = [], names = {} }: {
  steps: RecipeInput['steps'];
  onChange: (steps: RecipeInput['steps']) => void;
  ingredients?: RecipeInput['ingredients'];
  names?: Record<string, string>;
}) {
  function updateDetails<K extends Exclude<keyof StepDetails, 'formatVersion'>>(index: number, key: K, value: StepDetails[K]) {
    onChange(steps.map((step, n) => n === index
      ? { ...step, details: { ...emptyDetails, ...step.details, [key]: value } }
      : step));
  }
  function link(index: number, id: string, checked: boolean) {
    onChange(steps.map((step, n) => n === index ? {
      ...step, ingredientLineIds: checked
        ? [...(step.ingredientLineIds ?? []), id]
        : (step.ingredientLineIds ?? []).filter(value => value !== id),
    } : step));
  }
  function move(index: number, delta: number) {
    const result = [...steps];
    const target = index + delta;
    if (target < 0 || target >= result.length) return;
    [result[index], result[target]] = [result[target], result[index]];
    onChange(result);
  }
  return (
    <section className="panel recipe-form">
      <h2>{t.steps}</h2>
      {steps.map((step, index) => {
        const issues = stepDetailsIssues(step.details);
        return <div className="step-row" data-step-id={step.id} key={step.id}>
          <label>{index + 1}. {t.instructions}
            <textarea required maxLength={10000} value={step.instructions}
              onChange={e => onChange(steps.map((s, n) => n === index ? { ...s, instructions: e.target.value } : s))} />
          </label>
          <details className="step-advanced">
            <summary>{t.stepAdvanced}{(issues.duration || issues.temperature) && <span className="step-validation"> · {t.stepInvalidDetails}</span>}</summary>
            <div className="step-advanced-content">
              <div className="form-grid step-core-fields">
                <label>{t.stepTitle}<input maxLength={200} value={step.details?.title ?? ''}
                  onChange={e => updateDetails(index, 'title', e.target.value || null)} /></label>
                <div className="step-field">
                  <label>{t.stepDuration}<input inputMode="numeric" maxLength={32} value={step.details?.durationMinutes ?? ''}
                    aria-invalid={issues.duration} aria-describedby={`duration-help-${step.id}`}
                    onChange={e => updateDetails(index, 'durationMinutes', e.target.value || null)} /></label>
                  <small id={`duration-help-${step.id}`}>{t.stepDurationInvalid}</small>
                </div>
                <label>{t.stepTemperature}<input inputMode="numeric" maxLength={32} value={step.details?.temperatureValue ?? ''}
                  aria-invalid={issues.temperature} aria-describedby={`temperature-help-${step.id}`}
                  onChange={e => updateDetails(index, 'temperatureValue', e.target.value || null)} /></label>
                <label>{t.stepTemperatureUnit}<select value={step.details?.temperatureUnit ?? ''}
                  aria-invalid={issues.temperature} aria-describedby={`temperature-help-${step.id}`}
                  onChange={e => updateDetails(index, 'temperatureUnit', (e.target.value || null) as 'C' | 'F' | null)}>
                  <option value="">{t.stepUnitUnknown}</option><option value="C">°C</option><option value="F">°F</option>
                </select></label>
              </div>
              <p className="muted" id={`temperature-help-${step.id}`}>{t.stepTemperatureInvalid}</p>
              <div className="form-grid">
                <label>{t.stepHeat}<input maxLength={200} value={step.details?.heatLevel ?? ''}
                  onChange={e => updateDetails(index, 'heatLevel', e.target.value || null)} /></label>
                <label>{t.stepEquipment}<input maxLength={1000} value={step.details?.equipment ?? ''}
                  onChange={e => updateDetails(index, 'equipment', e.target.value || null)} /></label>
                <label>{t.stepTechnique}<input maxLength={1000} value={step.details?.technique ?? ''}
                  onChange={e => updateDetails(index, 'technique', e.target.value || null)} /></label>
              </div>
              <label>{t.stepTips}<textarea maxLength={5000} value={step.details?.tips ?? ''}
                onChange={e => updateDetails(index, 'tips', e.target.value || null)} /></label>
              <fieldset className="step-ingredient-links"><legend>{t.stepIngredients}</legend>
                <p className="muted">{t.stepIngredientsHelp}</p>
                {!ingredients.length && <p>{t.stepNoIngredients}</p>}
                {ingredients.map((line, lineIndex) => <label key={line.id} className="step-ingredient-choice">
                  <input type="checkbox" checked={(step.ingredientLineIds ?? []).includes(line.id)}
                    onChange={e => link(index, line.id, e.target.checked)} />
                  <span>{lineIndex + 1}. {names[line.ingredientId] ?? t.chooseIngredient} · {line.quantity ?? t.unknown} {line.unitCode}{line.note ? ` · ${line.note}` : ''}</span>
                </label>)}
                {(step.ingredientLineIds ?? []).filter(id => !ingredients.some(line => line.id === id)).map(id =>
                  <p role="alert" key={id}>{t.stepMissingLine} <Button type="button" variant="outline" onClick={() => link(index, id, false)}>{t.remove}</Button></p>)}
              </fieldset>
            </div>
          </details>
          <div className="recipe-actions">
            <Button type="button" variant="outline" aria-label={`${index + 1}. ${t.stepNumber}: ${t.up}`}
              disabled={index === 0} onClick={() => move(index, -1)}>{t.up}</Button>
            <Button type="button" variant="outline" aria-label={`${index + 1}. ${t.stepNumber}: ${t.down}`}
              disabled={index === steps.length - 1} onClick={() => move(index, 1)}>{t.down}</Button>
            <Button type="button" variant="ghost" onClick={() => onChange(steps.filter((_, n) => n !== index))}>{t.remove}</Button>
          </div>
        </div>;
      })}
      <Button type="button" variant="outline" onClick={() => onChange([...steps, { id: crypto.randomUUID(), instructions: '' }])}>{t.addStep}</Button>
    </section>
  );
}
