import { z } from 'zod';
import { recipeSchema } from './recipe';
const positive=z.number().int().positive().max(Number.MAX_SAFE_INTEGER);
export const versionSummarySchema=z.object({
 recipeId:z.uuid(),versionNumber:positive,formatVersion:positive,sourceRevision:positive,
 provenance:z.enum(['created','edited','upgrade_baseline','restored']),restoredFromVersion:positive.nullable(),
 capturedAt:z.string(),contentFingerprint:z.string().regex(/^[a-f0-9]{64}$/),snapshotChecksum:z.string().regex(/^[a-f0-9]{64}$/),
}).strict();
export const snapshotSchema=recipeSchema.omit({id:true,revision:true,createdAt:true,updatedAt:true,deletedAt:true,archivedAt:true}).extend({formatVersion:z.union([z.literal(1),z.literal(2),z.literal(3),z.literal(4)]),recipeId:z.uuid(),recipeCreatedAt:z.string()}).strict().refine(s=>(s.formatVersion>=4 || s.steps.every(step=>step.photoAssetId==null)) && (s.formatVersion>=3 || s.coverAssetId==null) && (s.formatVersion!==1 || s.ingredients.every(l=>!l.alternatives?.length)));
export const recipeVersionSchema=z.object({summary:versionSummarySchema,snapshot:snapshotSchema}).strict();
export const versionComparisonSchema=z.object({currentRevision:positive,changes:z.array(z.object({path:z.array(z.string()),before:z.json(),after:z.json()}).strict())}).strict();
export const restoreVersionSchema=z.object({recipeId:z.uuid(),versionNumber:positive,expectedRevision:positive}).strict();
export const restoreResultSchema=z.object({recipe:recipeSchema,changed:z.boolean(),versionNumber:positive}).strict();
export type RecipeVersionSummary=z.infer<typeof versionSummarySchema>;
export type RecipeVersion=z.infer<typeof recipeVersionSchema>;
export type VersionComparison=z.infer<typeof versionComparisonSchema>;
export type RestoreVersion=z.infer<typeof restoreVersionSchema>;
export type RestoreResult=z.infer<typeof restoreResultSchema>;
