import { z } from 'zod';
// M2 contracts only: not repository tables or enabled recipe functionality.
const decimal = z.string().regex(/^(0|[1-9]\d*)(\.\d+)?$/);
export const unitSchema = z.object({ code: z.string().min(1), dimension: z.enum(['mass', 'volume', 'count', 'temperature']) }).strict();
export const ingredientSchema = z.object({ id: z.uuid(), canonicalName: z.string().trim().min(1), displayName: z.string().trim().min(1) }).strict();
export const stepSchema = z.object({ id: z.uuid(), position: z.number().int().nonnegative(), description: z.string().trim().min(1) }).strict();
export const recipeDraftSchema = z.object({
  kind: z.enum(['food', 'beverage']), title: z.string().trim().min(1),
  yield: decimal.refine((value) => /[1-9]/.test(value)),
  ingredients: z.array(z.object({ ingredientId: z.uuid(), quantity: decimal.nullable(), unitCode: z.string().min(1) }).strict()),
  steps: z.array(stepSchema),
}).strict();
export type RecipeDraft = z.infer<typeof recipeDraftSchema>;
