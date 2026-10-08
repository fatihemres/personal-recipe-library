import type { ReactNode } from 'react';
import type { Unit, RecipeInput } from '../../shared/contracts/recipe';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
const nullable = (value: string) => (value.trim() ? value : null);
const decimal = (value: string) => value.trim().replace(',', '.');
export function RecipeIngredients({
  lines,
  units,
  names,
  search,
  onChange,
}: {
  lines: RecipeInput['ingredients'];
  units: Unit[];
  names: Record<string, string>;
  search: ReactNode;
  onChange: (lines: RecipeInput['ingredients']) => void;
}) {
  return (
    <section className="panel recipe-form">
      <h2>{t.ingredients}</h2>
      <p className="muted">{t.quantityHelp}</p>
      {search}
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
