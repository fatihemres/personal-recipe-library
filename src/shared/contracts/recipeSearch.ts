import { z } from 'zod';

export const recipeSearchQuerySchema = z.object({
  text: z.string().max(200),
  scope: z.enum(['active', 'archived', 'trash']),
  kind: z.enum(['food', 'beverage']).nullable(),
  ingredientId: z.uuid().nullable(),
  favoriteOnly: z.boolean().default(false),
  toTryOnly: z.boolean().default(false),
  collectionId: z.uuid().nullable().default(null),
  createdFrom: z.iso.date().nullable(),
  createdTo: z.iso.date().nullable(),
  modifiedFrom: z.iso.date().nullable(),
  modifiedTo: z.iso.date().nullable(),
  sort: z.enum(['modified', 'created', 'title_asc', 'title_desc', 'relevance']),
  offset: z.number().int().min(0).max(1_000_000),
  limit: z.number().int().min(1).max(50),
}).strict();

export const recipeSearchItemSchema = z.object({
  id: z.uuid(), title: z.string(), description: z.string().nullable(),
  kind: z.enum(['food', 'beverage']), servings: z.string(),
  createdAt: z.string(), updatedAt: z.string(),
  organizationRevision: z.number().int().positive(), favorite: z.boolean(),
  toTry: z.boolean(), rating: z.number().int().min(1).max(5).nullable(),
}).strict();

export const recipeSearchPageSchema = z.object({
  items: z.array(recipeSearchItemSchema), total: z.number().int().nonnegative(),
  hasMore: z.boolean(), suggestions: z.array(z.string()),
}).strict();

export type RecipeSearchQuery = z.infer<typeof recipeSearchQuerySchema>;
export type RecipeSearchItem = z.infer<typeof recipeSearchItemSchema>;
export type RecipeSearchPage = z.infer<typeof recipeSearchPageSchema>;
