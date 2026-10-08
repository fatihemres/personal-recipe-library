import type {
  Ingredient,
  Unit,
  RecipeInput,
} from '../../shared/contracts/recipe';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
const nullable = (value: string) => (value.trim() ? value : null);
const decimal = (value: string) => value.trim().replace(',', '.');
export function RecipeIngredients({
  lines,
  units,
  ingredients,
  names,
  query,
  onQuery,
  adding,
  onCreate,
  onChange,
}: {
  lines: RecipeInput['ingredients'];
  units: Unit[];
  ingredients: Ingredient[];
  names: Record<string, string>;
  query: string;
  onQuery: (query: string) => void;
  adding: boolean;
  onCreate: () => void;
  onChange: (lines: RecipeInput['ingredients']) => void;
}) {
  return (
    <section className="panel recipe-form">
      <h2>{t.ingredients}</h2>
      <p className="muted">{t.quantityHelp}</p>
      <label>
        {t.searchIngredient}
        <input
          maxLength={200}
          value={query}
          onChange={(e) => onQuery(e.target.value)}
        />
      </label>
      <div className="ingredient-results">
        {ingredients.map((i) => (
          <Button
            key={i.id}
            type="button"
            variant="outline"
            onClick={() =>
              onChange([
                ...lines,
                {
                  id: crypto.randomUUID(),
                  ingredientId: i.id,
                  quantity: null,
                  unitCode: 'g',
                  note: null,
                },
              ])
            }
          >
            {i.name}
          </Button>
        ))}
      </div>
      <Button
        type="button"
        variant="outline"
        disabled={adding || !query.trim()}
        onClick={() => onCreate()}
      >
        {t.createIngredient}
      </Button>
      {lines.map((line, index) => (
        <div className="ingredient-row" key={line.id}>
          <p>
            <strong>{names[line.ingredientId] ?? t.chooseIngredient}</strong>
          </p>
          <div className="form-grid">
            <label>
              {t.quantity}
              <input
                inputMode="decimal"
                value={line.quantity ?? ''}
                onChange={(e) =>
                  onChange(
                    lines.map((l, n) =>
                      n === index
                        ? {
                            ...l,
                            quantity: nullable(decimal(e.target.value)),
                          }
                        : l,
                    ),
                  )
                }
              />
            </label>
            <label>
              {t.unit}
              <select
                value={line.unitCode}
                onChange={(e) =>
                  onChange(
                    lines.map((l, n) =>
                      n === index ? { ...l, unitCode: e.target.value } : l,
                    ),
                  )
                }
              >
                {units.map((u) => (
                  <option key={u.code}>{u.code}</option>
                ))}
              </select>
            </label>
            <label>
              {t.lineNote}
              <input
                maxLength={2000}
                value={line.note ?? ''}
                onChange={(e) =>
                  onChange(
                    lines.map((l, n) =>
                      n === index
                        ? { ...l, note: nullable(e.target.value) }
                        : l,
                    ),
                  )
                }
              />
            </label>
          </div>
          <Button
            type="button"
            variant="ghost"
            onClick={() => onChange(lines.filter((_, n) => n !== index))}
          >
            {t.remove}
          </Button>
        </div>
      ))}
    </section>
  );
}
