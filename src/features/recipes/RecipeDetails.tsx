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
            <li key={s.id}>{s.instructions}</li>
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
