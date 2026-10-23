import { z } from 'zod';
import { recipeSearchItemSchema, recipeSearchQuerySchema } from './recipeSearch';
import { recipeCollectionSchema } from './recipeOrganization';

export const privateNoteSchema = z.object({recipeId:z.uuid(),formatVersion:z.literal(1),markdown:z.string(),revision:z.number().int().nonnegative(),updatedAt:z.string().nullable()}).strict();
export const privateNoteWriteSchema = z.object({recipeId:z.uuid(),expectedRevision:z.number().int().nonnegative(),markdown:z.string().max(50_000)}).strict();
export type PrivateNote = z.infer<typeof privateNoteSchema>;
export type PrivateNoteWrite = z.infer<typeof privateNoteWriteSchema>;

export const savedFilterQuerySchema = recipeSearchQuerySchema.extend({offset:z.literal(0),limit:z.literal(24)});
export const savedFilterSchema = z.object({id:z.uuid(),title:z.string(),revision:z.number().int().positive(),formatVersion:z.number().int().positive(),query:recipeSearchQuerySchema.nullable(),available:z.boolean()}).strict();
export const savedFilterWriteSchema = z.object({id:z.uuid(),expectedRevision:z.number().int().positive().nullable(),title:z.string().trim().min(1).max(100),query:savedFilterQuerySchema}).strict();
export type SavedFilter = z.infer<typeof savedFilterSchema>;
export type SavedFilterWrite = z.infer<typeof savedFilterWriteSchema>;

export const dashboardSchema = z.object({
  activeTotal:z.number().int().nonnegative(),foodCount:z.number().int().nonnegative(),
  beverageCount:z.number().int().nonnegative(),favoriteCount:z.number().int().nonnegative(),
  recentCreated:z.array(recipeSearchItemSchema),recentModified:z.array(recipeSearchItemSchema),
  collections:z.array(recipeCollectionSchema),
}).strict();
export type Dashboard = z.infer<typeof dashboardSchema>;
