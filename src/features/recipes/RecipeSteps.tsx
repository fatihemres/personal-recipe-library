import type { RecipeInput } from '../../shared/contracts/recipe';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
export function RecipeSteps({
  steps,
  onChange,
}: {
  steps: RecipeInput['steps'];
  onChange: (steps: RecipeInput['steps']) => void;
}) {
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
      {steps.map((step, index) => (
        <div className="step-row" key={step.id}>
          <label>
            {index + 1}. {t.instructions}
            <textarea
              required
              maxLength={10000}
              value={step.instructions}
              onChange={(e) =>
                onChange(
                  steps.map((s, n) =>
                    n === index ? { ...s, instructions: e.target.value } : s,
                  ),
                )
              }
            />
          </label>
          <div className="recipe-actions">
            <Button
              type="button"
              variant="outline"
              disabled={index === 0}
              onClick={() => move(index, -1)}
            >
              {t.up}
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={index === steps.length - 1}
              onClick={() => move(index, 1)}
            >
              {t.down}
            </Button>
            <Button
              type="button"
              variant="ghost"
              onClick={() => onChange(steps.filter((_, n) => n !== index))}
            >
              {t.remove}
            </Button>
          </div>
        </div>
      ))}
      <Button
        type="button"
        variant="outline"
        onClick={() =>
          onChange([...steps, { id: crypto.randomUUID(), instructions: '' }])
        }
      >
        {t.addStep}
      </Button>
    </section>
  );
}
