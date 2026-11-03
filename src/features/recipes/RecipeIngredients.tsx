import { useState,type ReactNode } from 'react';
import type { Unit, RecipeInput } from '../../shared/contracts/recipe';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
import type {RecipeClient} from '../../shared/api/recipes';
import {IngredientAlternatives} from './IngredientAlternatives';
import {ConfirmDialog} from '../../shared/ui/ConfirmDialog';
import {alternativesTr as a} from '../../shared/i18n/ingredientAlternatives';
const nullable = (value: string) => (value.trim() ? value : null);
const decimal = (value: string) => value.trim().replace(',', '.');
export function RecipeIngredients({
  client,onName,lines,
  units,
  names,
  search,
  onChange,
}: {
  client:RecipeClient;onName:(id:string,name:string)=>void;
  lines: RecipeInput['ingredients'];
  units: Unit[];
  names: Record<string, string>;
  search: ReactNode;
  onChange: (lines: RecipeInput['ingredients']) => void;
}) {
  const [removing,setRemoving]=useState<string|null>(null);
  const totalAlternatives=lines.reduce((n,l)=>n+(l.alternatives?.length??0),0);
  function move(index:number,direction:number){const items=[...lines];[items[index],items[index+direction]]=[items[index+direction],items[index]];onChange(items);}
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
          <IngredientAlternatives totalAlternatives={totalAlternatives} line={line} client={client} onName={onName} units={units} names={names} onChange={next=>onChange(lines.map(l=>l.id===line.id?next:l))}/>
          <div className="recipe-actions"><Button type="button" variant="outline" aria-label={`${index+1}. ${a.lineUp}`} disabled={index===0} onClick={()=>move(index,-1)}>{a.lineUp}</Button><Button type="button" variant="outline" aria-label={`${index+1}. ${a.lineDown}`} disabled={index===lines.length-1} onClick={()=>move(index,1)}>{a.lineDown}</Button>
          <Button
            type="button"
            variant="ghost"
            onClick={() => line.alternatives?.length?setRemoving(line.id):onChange(lines.filter(l=>l.id!==line.id))}
          >
            {t.remove}
          </Button></div>
        </div>
      ))}
      {removing&&<ConfirmDialog title={a.removeLine} description={a.removeLineHelp} confirm={a.confirmRemove} cancel={a.cancel} onCancel={()=>setRemoving(null)} onConfirm={()=>{onChange(lines.filter(l=>l.id!==removing));setRemoving(null);}}><p><strong>{names[lines.find(l=>l.id===removing)?.ingredientId??'']}</strong> · {a.title}: {lines.find(l=>l.id===removing)?.alternatives?.length??0}</p></ConfirmDialog>}
    </section>
  );
}
