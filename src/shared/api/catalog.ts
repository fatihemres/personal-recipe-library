import { z } from 'zod';
import { request } from './client';
import { personalIngredientSchema } from '../contracts/recipe';
export const catalogStatusSchema = z.object({ definitions: z.number().int().nonnegative(), validationDefinitions: z.number().int().nonnegative(), productionDefinitions: z.number().int().nonnegative(), pendingCollisions: z.number().int().nonnegative() }).strict();
const ingredientEditSchema = personalIngredientSchema.omit({ origin: true }).strict();
export const catalogClient = {
  status: () => request('catalog_status', catalogStatusSchema),
  linkPersonal: (id: string, revision: number, catalogId: string) => request('link_personal_catalog', z.null(), { id: z.uuid().parse(id), revision: z.number().int().positive().parse(revision), catalogId: z.uuid().parse(catalogId) }),
  customize: (input: z.infer<typeof ingredientEditSchema>) => request('customize_catalog_ingredient', personalIngredientSchema, { input: ingredientEditSchema.parse(input) }),
  createCategory: (tr: string, en: string, parent: string | null) => request('create_personal_category', z.uuid(), { tr: z.string().trim().min(1).max(200).parse(tr), en: z.string().trim().min(1).max(200).parse(en), parent: z.uuid().nullable().parse(parent) }),
};
