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
export const stepDetailsSchema = z.object({
  formatVersion: z.literal(1),
  title: z.string().max(200).nullable(),
  durationMinutes: z.string().max(32).nullable(),
  temperatureValue: z.string().max(32).nullable(),
  temperatureUnit: z.enum(['C','F']).nullable(),
  heatLevel: z.string().max(200).nullable(),
  equipment: z.string().max(1000).nullable(),
  technique: z.string().max(1000).nullable(),
  tips: z.string().max(5000).nullable(),
}).strict();
const stepFields = {
  id: z.uuid(), instructions: z.string().max(10000),
  details: stepDetailsSchema.optional(),
  ingredientLineIds: z.array(z.uuid()).max(500).optional(),
};
export const stepSchema = z
  .object({ ...stepFields, instructions: z.string().trim().min(1).max(10000) })
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
export type StepDetails = z.infer<typeof stepDetailsSchema>;
export function stepDetailsIssues(d?: StepDetails) {
  return {
    duration: !!d && d.durationMinutes!==null && (!/^\d+$/.test(d.durationMinutes) || Number(d.durationMinutes)>10080),
    temperature: !!d && ((d.temperatureValue===null)!==(d.temperatureUnit===null) ||
      (d.temperatureValue!==null && (d.temperatureValue.length>5 || !/^-?\d+$/.test(d.temperatureValue) || Number(d.temperatureValue)<-999 || Number(d.temperatureValue)>9999))),
  };
}
function validStepDetails(input: {steps: z.infer<typeof stepSchema>[]; ingredients: z.infer<typeof lineSchema>[]}) {
  const lines=new Set(input.ingredients.map(line=>line.id));
  return input.steps.every(step=>{
    const ids=step.ingredientLineIds??[];
    const issues=stepDetailsIssues(step.details);
    return new Set(ids).size===ids.length && ids.every(id=>lines.has(id)) && !issues.duration && !issues.temperature;
  });
}
export const recipeInputSchema = z
  .object({
    ...fields,
    expectedRevision: z.number().int().positive().nullable(),
  })
  .strict().refine(validStepDetails);
export const recipeSchema = z
  .object({
    ...fields,
    ingredientNames: z.record(z.string(), z.string()),
    revision: z.number().int().positive(),
    createdAt: z.string(),
    updatedAt: z.string(),
    deletedAt: z.string().nullable(),
    archivedAt: z.string().nullable(),
  })
  .strict();
export type Recipe = z.infer<typeof recipeSchema>;
export type RecipeInput = z.infer<typeof recipeInputSchema>;
export type Ingredient = z.infer<typeof ingredientSchema>;
export type Unit = z.infer<typeof unitSchema>;

// Drafts intentionally accept unfinished text that committed recipes reject.
export const draftInputSchema = z.object({ ...fields, expectedRevision: z.number().int().positive().nullable(),
  title: z.string().max(200),
  servings: z.string().max(100),
  ingredients: z
    .array(lineSchema.extend({ quantity: z.string().max(100).nullable() }))
    .max(500),
  steps: z
    .array(
      z.object(stepFields).strict(),
    )
    .max(500),
  prepMinutes: z.number().int().nullable(),
  cookMinutes: z.number().int().nullable(),
}).strict();
export const draftSchema = z
  .object({
    id: z.uuid(),
    revision: z.number().int().positive(),
    recipeId: z.uuid().nullable(),
    updatedAt: z.string(),
    input: draftInputSchema,
    ingredientNames: z.record(z.string(), z.string()),
  })
  .strict();
export const personalIngredientSchema = ingredientSchema.extend({
  notes: z.string().max(2000).nullable(),
  preferredUnit: z.string().nullable(),
  revision: z.number().int().positive(),
  origin: z.enum(['personal', 'catalog']),
});
export const ingredientSearchSchema = z
  .object({
    items: z.array(personalIngredientSchema),
    total: z.number().int().nonnegative(),
    hasMore: z.boolean(),
  })
  .strict();
export type Draft = z.infer<typeof draftSchema>;
export type PersonalIngredient = z.infer<typeof personalIngredientSchema>;
export type IngredientSearch = z.infer<typeof ingredientSearchSchema>;
