import { z } from 'zod';
export const themeSchema = z.enum(['light', 'dark', 'system']);
export const preferencesSchema = z.object({ theme: themeSchema, locale: z.literal('tr') }).strict();
export const storageSchema = z.object({
  schemaVersion: z.number().int().positive(), sqliteVersion: z.string().min(1),
  foreignKeys: z.literal(true), fts5: z.literal(true),
}).strict();
export const bootstrapSchema = z.object({ preferences: preferencesSchema, storage: storageSchema }).strict();
export const appErrorSchema = z.object({ code: z.string(), messageKey: z.string(), recoverable: z.boolean() }).strict();
export type Theme = z.infer<typeof themeSchema>;
export type Preferences = z.infer<typeof preferencesSchema>;
export type Bootstrap = z.infer<typeof bootstrapSchema>;
export type AppError = z.infer<typeof appErrorSchema>;
