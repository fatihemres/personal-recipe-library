import { z } from 'zod';
import { recipeSchema } from './recipe';
const positive=z.number().int().positive().max(Number.MAX_SAFE_INTEGER);
export const versionSummarySchema=z.object({
 recipeId:z.uuid(),versionNumber:positive,formatVersion:positive,sourceRevision:positive,
 provenance:z.enum(['created','edited','upgrade_baseline','restored']),restoredFromVersion:positive.nullable(),
 capturedAt:z.string(),contentFingerprint:z.string().regex(/^[a-f0-9]{64}$/),snapshotChecksum:z.string().regex(/^[a-f0-9]{64}$/),
}).strict();
export const snapshotSchema=recipeSchema.omit({id:true,revision:true,createdAt:true,updatedAt:true,deletedAt:true,archivedAt:true}).extend({formatVersion:z.literal(1),recipeId:z.uuid(),recipeCreatedAt:z.string()}).strict();
export const recipeVersionSchema=z.object({summary:versionSummarySchema,snapshot:snapshotSchema}).strict();
export const versionComparisonSchema=z.object({currentRevision:positive,changes:z.array(z.object({path:z.array(z.string()),before:z.json(),after:z.json()}).strict())}).strict();
export const restoreVersionSchema=z.object({recipeId:z.uuid(),versionNumber:positive,expectedRevision:positive}).strict();
export const restoreResultSchema=z.object({recipe:recipeSchema,changed:z.boolean(),versionNumber:positive}).strict();
export type RecipeVersionSummary=z.infer<typeof versionSummarySchema>;
export type RecipeVersion=z.infer<typeof recipeVersionSchema>;
export type VersionComparison=z.infer<typeof versionComparisonSchema>;
export type RestoreVersion=z.infer<typeof restoreVersionSchema>;
export type RestoreResult=z.infer<typeof restoreResultSchema>;
