import { z } from 'zod';
export const originSchema = z.enum(['catalog', 'personal']);
export const libraryQuerySchema = z.object({ search: z.string().max(200), origin: z.enum(['all', 'catalog', 'personal']), categoryId: z.uuid().nullable(), offset: z.number().int().min(0).max(1000000), limit: z.number().int().min(1).max(100), view: z.enum(['favorites','recent']).nullable().optional() }).strict();
export const categorySchema = z.object({ id: z.uuid(), parentId: z.uuid().nullable(), key: z.string(), tr: z.string(), en: z.string() }).strict();
export const libraryItemSchema = z.object({ id: z.uuid(), origin: originSchema, recipeId: z.uuid().nullable(), name: z.string(), englishName: z.string().nullable(), notes: z.string().nullable(), preferredUnit: z.string().nullable(), revision: z.number().int().positive().nullable(), ingredientType: z.string().nullable(), categories: z.array(categorySchema), favorite: z.boolean(), lastUsedAt: z.string().nullable() }).strict();
export const libraryPageSchema = z.object({ items: z.array(libraryItemSchema), total: z.number().int().nonnegative(), catalogCount: z.number().int().nonnegative(), personalCount: z.number().int().nonnegative(), categories: z.array(z.object({ category: categorySchema, count: z.number().int().nonnegative() }).strict()), offset: z.number().int().nonnegative(), limit: z.number().int().positive() }).strict();
export const libraryDetailSchema = z.object({ item: libraryItemSchema, canonicalTr: z.string().nullable(), aliases: z.array(z.object({ locale: z.string(), name: z.string() }).strict()), dimensions: z.array(z.string()), sources: z.array(z.object({ name: z.string(), sourceId: z.string(), version: z.string(), externalId: z.string(), url: z.string(), license: z.string(), attribution: z.string(), description: z.string() }).strict()), metadata: z.array(z.object({ kind: z.string(), code: z.string(), value: z.string(), unit: z.string().nullable(), basis: z.string() }).strict()), catalogVersion: z.number().int().positive().nullable(), personalAlias: z.string().nullable() }).strict();
export type LibraryQuery = z.infer<typeof libraryQuerySchema>;
export type LibraryPage = z.infer<typeof libraryPageSchema>;
export type LibraryItem = z.infer<typeof libraryItemSchema>;
export type LibraryDetail = z.infer<typeof libraryDetailSchema>;

export const catalogPresentationSchema = z.object({ id: z.uuid(), revision: z.number().int().positive(), name: z.string().min(1).max(200).nullable(), alias: z.string().min(1).max(200).nullable(), preferredUnit: z.string().nullable(), notes: z.string().max(2000).nullable() }).strict();
export type CatalogPresentation = z.infer<typeof catalogPresentationSchema>;
export const ingredientCandidateSchema = z.object({ id:z.uuid(), origin:originSchema, name:z.string(), reason:z.string() }).strict();
export type IngredientCandidate = z.infer<typeof ingredientCandidateSchema>;
