import { z } from 'zod';
import { request } from './client';
import {
  draftSchema,
  draftInputSchema,
  personalIngredientSchema,
  ingredientSearchSchema,
  type Draft,
  type PersonalIngredient,
  type IngredientSearch,
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
  drafts(): Promise<Draft[]>;
  getDraft(id: string): Promise<Draft>;
  saveDraft(
    id: string,
    expectedRevision: number | null,
    input: RecipeInput,
  ): Promise<Draft>;
  discardDraft(id: string, revision: number): Promise<null>;
  commit(
    input: RecipeInput,
    draftId: string,
    draftRevision: number,
    asCopy: boolean,
  ): Promise<Recipe>;
  duplicate(id: string, revision: number): Promise<Recipe>;
  scope(
    scope: 'active' | 'archived' | 'trash',
    kind: 'food' | 'beverage' | null,
  ): Promise<Recipe[]>;
  archive(id: string, revision: number, archived: boolean): Promise<Recipe>;
  purge(id: string, revision: number): Promise<null>;
  searchAvailable?(query: string): Promise<IngredientSearch>;
  searchPersonal(query: string): Promise<IngredientSearch>;
  editIngredient(input: {
    id: string;
    revision: number;
    name: string;
    notes: string | null;
    preferredUnit: string | null;
  }): Promise<PersonalIngredient>;
  deleteIngredient(id: string, revision: number): Promise<null>;
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
  drafts: () => request('list_drafts', z.array(draftSchema)),
  getDraft: (id) => request('get_draft', draftSchema, { id }),
  saveDraft: (id, expectedRevision, input) =>
    request('save_draft', draftSchema, {
      write: { id, expectedRevision, input: draftInputSchema.parse(input) },
    }),
  discardDraft: (id, revision) =>
    request('discard_draft', z.null(), { id, revision }),
  commit: (input, draftId, draftRevision, asCopy) =>
    request('commit_recipe', recipeSchema, {
      input: recipeInputSchema.parse(input),
      draftId,
      draftRevision,
      asCopy,
    }),
  duplicate: (id, revision) =>
    request('duplicate_recipe', recipeSchema, { id, revision }),
  scope: (scope, kind) =>
    request('scope_recipes', z.array(recipeSchema), { scope, kind }),
  archive: (id, revision, archived) =>
    request('archive_recipe', recipeSchema, { id, revision, archived }),
  purge: (id, revision) => request('purge_recipe', z.null(), { id, revision }),
  searchAvailable: (query) => request('search_available_ingredients', ingredientSearchSchema, { query }),
  searchPersonal: (query) =>
    request('search_personal_ingredients', ingredientSearchSchema, { query }),
  editIngredient: (input) =>
    request('edit_ingredient', personalIngredientSchema, { input }),
  deleteIngredient: (id, revision) =>
    request('delete_ingredient', z.null(), { id, revision }),
  units: () => request('list_units', z.array(unitSchema)),
};
