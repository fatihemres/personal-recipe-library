import type { Recipe } from '../../shared/contracts/recipe';
import { recipesTr as t } from '../../shared/i18n/recipes';
export function RecipeDetails({ recipe }: { recipe: Recipe }) {
  return (
    <div className="panel recipe-form">
      <p>
        {recipe.kind === 'food' ? t.food : t.beverage} · {recipe.servings}{' '}
        {t.portions}
      </p>
      {recipe.prepMinutes !== null && (
        <p>
          {t.prep}: {recipe.prepMinutes} {t.minute}
        </p>
      )}
      {recipe.cookMinutes !== null && (
        <p>
          {t.cook}: {recipe.cookMinutes} {t.minute}
        </p>
      )}
      <h2>{t.ingredients}</h2>
      {recipe.ingredients.length ? (
        <ul className="detail-list">
          {recipe.ingredients.map((i) => (
            <li key={i.id}>
              <strong>{recipe.ingredientNames[i.ingredientId]}</strong> —{' '}
              {i.quantity === null ? t.unknown : `${i.quantity} ${i.unitCode}`}
              {i.note && <p>{i.note}</p>}
            </li>
          ))}
        </ul>
      ) : (
        <p>{t.noIngredients}</p>
      )}
      <h2>{t.steps}</h2>
      {recipe.steps.length ? (
        <ol className="detail-list">
          {recipe.steps.map((s) => (
            <li key={s.id}>
              {s.details?.title && <h3>{s.details.title}</h3>}
              <p className="preserve-text">{s.instructions}</p>
              {s.details && <dl className="step-metadata">
                {s.details.durationMinutes!==null && <><dt>{t.stepDuration}</dt><dd>{s.details.durationMinutes}</dd></>}
                {s.details.temperatureValue!==null && <><dt>{t.stepTemperature}</dt><dd>{s.details.temperatureValue} °{s.details.temperatureUnit}</dd></>}
                {(['heatLevel','equipment','technique','tips'] as const).map(key=>s.details?.[key] && <div key={key}><dt>{key==='heatLevel'?t.stepHeat:key==='equipment'?t.stepEquipment:key==='technique'?t.stepTechnique:t.stepTips}</dt><dd className="preserve-text">{s.details[key]}</dd></div>)}
              </dl>}
              {!!s.ingredientLineIds?.length && <><strong>{t.stepIngredients}</strong><ul>{s.ingredientLineIds.map(id=>{
                const line=recipe.ingredients.find(line=>line.id===id);
                return line?<li key={id}>{recipe.ingredients.indexOf(line)+1}. {recipe.ingredientNames[line.ingredientId]} · {line.quantity??t.unknown} {line.unitCode}{line.note?` · ${line.note}`:''}</li>:null;
              })}</ul></>}
            </li>
          ))}
        </ol>
      ) : (
        <p>{t.noSteps}</p>
      )}
      {recipe.notes && (
        <>
          <h2>{t.notes}</h2>
          <p className="preserve-text">{recipe.notes}</p>
        </>
      )}
    </div>
  );
}
