import { z } from 'zod';
export const decimalSchema = z
  .string()
  .regex(/^(0|[1-9]\d{0,11})(\.\d{1,6})?$/)
  .refine((v) => /[1-9]/.test(v));
export const unitSchema = z
  .object({
    code: z.string(),
    dimension: z.enum(['mass', 'volume', 'count']),
    canonicalCode: z.string(),
    factor: z.number().int().positive(),
  })
  .strict();
export const ingredientSchema = z
  .object({ id: z.uuid(), name: z.string().min(1).max(200) })
  .strict();
export const lineSchema = z
  .object({
    id: z.uuid(),
    ingredientId: z.uuid(),
    quantity: decimalSchema.nullable(),
    unitCode: z.string().min(1),
    note: z.string().max(2000).nullable(),
  })
  .strict();
export const stepSchema = z
  .object({ id: z.uuid(), instructions: z.string().trim().min(1).max(10000) })
  .strict();
const fields = {
  id: z.uuid(),
  title: z.string().trim().min(1).max(200),
  description: z.string().max(10000).nullable(),
  kind: z.enum(['food', 'beverage']),
  servings: decimalSchema,
  prepMinutes: z.number().int().min(0).max(10080).nullable(),
  cookMinutes: z.number().int().min(0).max(10080).nullable(),
  notes: z.string().max(20000).nullable(),
  ingredients: z.array(lineSchema).max(500),
  steps: z.array(stepSchema).max(500),
};
export const recipeInputSchema = z
  .object({
    ...fields,
    expectedRevision: z.number().int().positive().nullable(),
  })
  .strict();
export const recipeSchema = z
  .object({
    ...fields,
    ingredientNames: z.record(z.string(), z.string()),
    revision: z.number().int().positive(),
    createdAt: z.string(),
    updatedAt: z.string(),
    deletedAt: z.string().nullable(),
  })
  .strict();
export type Recipe = z.infer<typeof recipeSchema>;
export type RecipeInput = z.infer<typeof recipeInputSchema>;
export type Ingredient = z.infer<typeof ingredientSchema>;
export type Unit = z.infer<typeof unitSchema>;
