import { z } from 'zod';

export const recipeOrganizationSchema = z.object({
  recipeId: z.uuid(), revision: z.number().int().positive(),
  favorite: z.boolean(), toTry: z.boolean(),
  rating: z.number().int().min(1).max(5).nullable(),
  collectionIds: z.array(z.uuid()),
}).strict();
export type RecipeOrganization = z.infer<typeof recipeOrganizationSchema>;

export const organizationWriteSchema = recipeOrganizationSchema.pick({
  recipeId: true, favorite: true, toTry: true, rating: true,
}).extend({ expectedRevision: z.number().int().positive() }).strict();
export type OrganizationWrite = z.infer<typeof organizationWriteSchema>;

export const recipeCollectionSchema = z.object({
  id: z.uuid(), title: z.string().trim().min(1).max(100),
  revision: z.number().int().positive(), activeCount: z.number().int().nonnegative(),
}).strict();
export type RecipeCollection = z.infer<typeof recipeCollectionSchema>;

export const collectionWriteSchema = recipeCollectionSchema.pick({id:true,title:true})
  .extend({expectedRevision:z.number().int().positive().nullable()}).strict();
export type CollectionWrite = z.infer<typeof collectionWriteSchema>;
