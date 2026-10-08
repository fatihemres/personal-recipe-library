import { z } from 'zod';
import { request } from './client';
import {
  recipeSchema,
  recipeInputSchema,
  ingredientSchema,
  unitSchema,
  type Recipe,
  type RecipeInput,
  type Ingredient,
  type Unit,
} from '../contracts/recipe';
export interface RecipeClient {
  list(trash: boolean, kind: 'food' | 'beverage' | null): Promise<Recipe[]>;
  get(id: string): Promise<Recipe>;
  save(input: RecipeInput): Promise<Recipe>;
  setDeleted(id: string, revision: number, deleted: boolean): Promise<Recipe>;
  searchIngredients(query: string): Promise<Ingredient[]>;
  createIngredient(name: string): Promise<Ingredient>;
  units(): Promise<Unit[]>;
}
export const recipeClient: RecipeClient = {
  list: (trash, kind) =>
    request('list_recipes', z.array(recipeSchema), { trash, kind }),
  get: (id) => request('get_recipe', recipeSchema, { id }),
  save: (input) => {
    const parsed = recipeInputSchema.safeParse(input);
    if (!parsed.success)
      return Promise.reject({
        code: 'INVALID_INPUT',
        messageKey: 'errors.validation',
        recoverable: true,
      });
    return request('save_recipe', recipeSchema, { input: parsed.data });
  },
  setDeleted: (id, revision, deleted) =>
    request('set_recipe_deleted', recipeSchema, { id, revision, deleted }),
  searchIngredients: (query) =>
    request('search_ingredients', z.array(ingredientSchema), { query }),
  createIngredient: (name) =>
    request('create_ingredient', ingredientSchema, { name }),
  units: () => request('list_units', z.array(unitSchema)),
};
