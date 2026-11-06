import { convertFileSrc, isTauri } from '@tauri-apps/api/core';
import { recipeLinksSchema,parentWriteSchema,relatedWriteSchema,purgePreviewSchema,reviewedPurgeSchema,type RecipeLinks,type ParentWrite,type RelatedWrite,type PurgePreview,type ReviewedPurge } from '../contracts/recipeRelations';
import { versionSummarySchema, recipeVersionSchema, versionComparisonSchema, restoreVersionSchema, restoreResultSchema, type RecipeVersionSummary, type RecipeVersion, type VersionComparison, type RestoreVersion, type RestoreResult } from '../contracts/recipeHistory';
import { z } from 'zod';
import { recipeOrganizationSchema, organizationWriteSchema, recipeCollectionSchema, collectionWriteSchema,
  type RecipeOrganization, type OrganizationWrite, type RecipeCollection, type CollectionWrite } from '../contracts/recipeOrganization';
import { request } from './client';
import { dashboardSchema, privateNoteSchema, privateNoteWriteSchema, savedFilterSchema, savedFilterWriteSchema,
  type Dashboard, type PrivateNote, type PrivateNoteWrite, type SavedFilter, type SavedFilterWrite } from '../contracts/recipeWorkspace';
import { recipeSearchPageSchema, recipeSearchQuerySchema, type RecipeSearchPage, type RecipeSearchQuery } from '../contracts/recipeSearch';
import {
  draftSchema,
  draftInputSchema,
  personalIngredientSchema,
  ingredientSearchSchema,
  type Draft,
  type PersonalIngredient,
  type IngredientSearch,
  recipeSchema,
  recipeInputSchema,
  ingredientSchema,
  unitSchema,
  type Recipe,
  type RecipeInput,
  type Ingredient,
  type Unit,
} from '../contracts/recipe';
export interface RecipeClient {
  pickStepPhoto(): Promise<{id:string;width:number;height:number}|null>;
  pickCover(): Promise<{id:string;width:number;height:number}|null>;
  coverUrl(id:string,variant:'thumb'|'preview'):string;
  links(id:string,offset:number,limit:number):Promise<RecipeLinks>;
  createVariation(id:string,revision:number):Promise<Recipe>;
  setParent(write:ParentWrite):Promise<null>;
  setRelated(write:RelatedWrite):Promise<null>;
  previewPurge(id:string,revision:number,offset:number,limit:number):Promise<PurgePreview>;
  purgeReviewed(write:ReviewedPurge):Promise<null>;
  versions(id:string,before:number|null,limit:number):Promise<RecipeVersionSummary[]>;
  version(id:string,version:number):Promise<RecipeVersion>;
  compareVersions(id:string,left:number,right:number|null):Promise<VersionComparison>;
  restoreVersion(write:RestoreVersion):Promise<RestoreResult>;
  getPrivateNote(recipeId:string):Promise<PrivateNote>;
  savePrivateNote(write:PrivateNoteWrite):Promise<PrivateNote>;
  savedFilters():Promise<SavedFilter[]>;
  resolveFilter(id:string):Promise<RecipeSearchQuery>;
  saveFilter(write:SavedFilterWrite):Promise<SavedFilter>;
  deleteFilter(id:string,revision:number):Promise<null>;
  dashboard():Promise<Dashboard>;
  getOrganization(id:string):Promise<RecipeOrganization>;
  setOrganization(write:OrganizationWrite):Promise<RecipeOrganization>;
  collections():Promise<RecipeCollection[]>;
  saveCollection(write:CollectionWrite):Promise<RecipeCollection>;
  deleteCollection(id:string,revision:number):Promise<null>;
  setMembership(recipeId:string,collectionId:string,expectedRevision:number,member:boolean):Promise<RecipeOrganization>;
  searchRecipes(query: RecipeSearchQuery): Promise<RecipeSearchPage>;
  list(trash: boolean, kind: 'food' | 'beverage' | null): Promise<Recipe[]>;
  get(id: string): Promise<Recipe>;
  save(input: RecipeInput): Promise<Recipe>;
  setDeleted(id: string, revision: number, deleted: boolean): Promise<Recipe>;
  searchIngredients(query: string): Promise<Ingredient[]>;
  createIngredient(name: string): Promise<Ingredient>;
  units(): Promise<Unit[]>;
  drafts(): Promise<Draft[]>;
  getDraft(id: string): Promise<Draft>;
  saveDraft(
    id: string,
    expectedRevision: number | null,
    input: RecipeInput,
  ): Promise<Draft>;
  discardDraft(id: string, revision: number): Promise<null>;
  commit(
    input: RecipeInput,
    draftId: string,
    draftRevision: number,
    asCopy: boolean,
  ): Promise<Recipe>;
  duplicate(id: string, revision: number): Promise<Recipe>;
  scope(
    scope: 'active' | 'archived' | 'trash',
    kind: 'food' | 'beverage' | null,
  ): Promise<Recipe[]>;
  archive(id: string, revision: number, archived: boolean): Promise<Recipe>;
  purge(id: string, revision: number): Promise<null>;
  searchAvailable?(query: string): Promise<IngredientSearch>;
  suggestAvailable?(query:string, mode:'all'|'recent'|'favorites'):Promise<IngredientSearch>;
  searchPersonal(query: string): Promise<IngredientSearch>;
  editIngredient(input: {
    id: string;
    revision: number;
    name: string;
    notes: string | null;
    preferredUnit: string | null;
    categoryIds?: string[];
  }): Promise<PersonalIngredient>;
  deleteIngredient(id: string, revision: number): Promise<null>;
}
const importedPhotoSchema=z.object({id:z.uuid(),width:z.number().int().positive().max(30000000),height:z.number().int().positive().max(30000000)}).strict().refine(v=>v.width*v.height<=30000000).nullable();
export const recipeClient: RecipeClient = {
  pickCover:()=>request('pick_recipe_cover',importedPhotoSchema),
  pickStepPhoto:()=>request('pick_recipe_step_photo',importedPhotoSchema),
  coverUrl:(id,variant)=>{z.uuid().parse(id);z.enum(['thumb','preview']).parse(variant);const key=`${id}/${variant}`;return isTauri()?`${convertFileSrc(id,'recipe-media')}/${variant}`:`recipe-media://localhost/${key}`;},
  links:(id,offset,limit)=>request('recipe_links',recipeLinksSchema,{id:z.uuid().parse(id),offset:z.number().int().min(0).max(1000000).parse(offset),limit:z.number().int().min(1).max(100).parse(limit)}),
  createVariation:(id,revision)=>request('create_recipe_variation',recipeSchema,{id:z.uuid().parse(id),revision:z.number().int().positive().parse(revision)}),
  setParent:write=>request('set_variation_parent',z.null(),{write:parentWriteSchema.parse(write)}),
  setRelated:write=>request('set_related_recipe',z.null(),{write:relatedWriteSchema.parse(write)}),
  previewPurge:(id,revision,offset,limit)=>request('preview_recipe_purge',purgePreviewSchema,{id:z.uuid().parse(id),revision:z.number().int().positive().parse(revision),offset:z.number().int().min(0).max(1000000).parse(offset),limit:z.number().int().min(1).max(100).parse(limit)}),
  purgeReviewed:write=>request('purge_reviewed_recipe',z.null(),{write:reviewedPurgeSchema.parse(write)}),
  versions:(id,before,limit)=>request('list_recipe_versions',z.array(versionSummarySchema),{id:z.uuid().parse(id),before:z.number().int().positive().nullable().parse(before),limit:z.number().int().min(1).max(100).parse(limit)}),
  version:(id,version)=>request('get_recipe_version',recipeVersionSchema,{id:z.uuid().parse(id),version:z.number().int().positive().parse(version)}),
  compareVersions:(id,left,right)=>request('compare_recipe_versions',versionComparisonSchema,{id:z.uuid().parse(id),left:z.number().int().positive().parse(left),right:z.number().int().positive().nullable().parse(right)}),
  restoreVersion:write=>request('restore_recipe_version',restoreResultSchema,{write:restoreVersionSchema.parse(write)}),
  getPrivateNote: recipeId => request('get_private_note',privateNoteSchema,{recipeId:z.uuid().parse(recipeId)}),
  savePrivateNote: write => request('save_private_note',privateNoteSchema,{write:privateNoteWriteSchema.parse(write)}),
  savedFilters: () => request('list_saved_filters',z.array(savedFilterSchema)),
  resolveFilter: id => request('resolve_saved_filter',recipeSearchQuerySchema,{id:z.uuid().parse(id)}),
  saveFilter: write => request('save_saved_filter',savedFilterSchema,{write:savedFilterWriteSchema.parse(write)}),
  deleteFilter: (id,revision) => request('delete_saved_filter',z.null(),{id:z.uuid().parse(id),revision:z.number().int().positive().parse(revision)}),
  dashboard: () => request('recipe_dashboard',dashboardSchema),
  getOrganization: (id) => request('get_recipe_organization',recipeOrganizationSchema,{id:z.uuid().parse(id)}),
  setOrganization: (write) => request('set_recipe_organization',recipeOrganizationSchema,{write:organizationWriteSchema.parse(write)}),
  collections: () => request('list_recipe_collections',z.array(recipeCollectionSchema)),
  saveCollection: (write) => request('save_recipe_collection',recipeCollectionSchema,{write:collectionWriteSchema.parse(write)}),
  deleteCollection: (id,revision) => request('delete_recipe_collection',z.null(),{id:z.uuid().parse(id),revision:z.number().int().positive().parse(revision)}),
  setMembership: (recipeId,collectionId,expectedRevision,member) => request('set_recipe_collection_membership',recipeOrganizationSchema,{recipeId:z.uuid().parse(recipeId),collectionId:z.uuid().parse(collectionId),expectedRevision:z.number().int().positive().parse(expectedRevision),member}),
  searchRecipes: (query) => request('search_recipes', recipeSearchPageSchema, { query: recipeSearchQuerySchema.parse(query) }),
  list: (trash, kind) =>
    request('list_recipes', z.array(recipeSchema), { trash, kind }),
  get: (id) => request('get_recipe', recipeSchema, { id }),
  save: (input) => {
    const parsed = recipeInputSchema.safeParse(input);
    if (!parsed.success)
      return Promise.reject({
        code: 'INVALID_INPUT',
        messageKey: 'errors.validation',
        recoverable: true,
      });
    return request('save_recipe', recipeSchema, { input: parsed.data });
  },
  setDeleted: (id, revision, deleted) =>
    request('set_recipe_deleted', recipeSchema, { id, revision, deleted }),
  searchIngredients: (query) =>
    request('search_ingredients', z.array(ingredientSchema), { query }),
  createIngredient: (name) =>
    request('create_ingredient', ingredientSchema, { name }),
  drafts: () => request('list_drafts', z.array(draftSchema)),
  getDraft: (id) => request('get_draft', draftSchema, { id }),
  saveDraft: (id, expectedRevision, input) =>
    request('save_draft', draftSchema, {
      write: { id, expectedRevision, input: draftInputSchema.parse(input) },
    }),
  discardDraft: (id, revision) =>
    request('discard_draft', z.null(), { id, revision }),
  commit: (input, draftId, draftRevision, asCopy) =>
    request('commit_recipe', recipeSchema, {
      input: recipeInputSchema.parse(input),
      draftId,
      draftRevision,
      asCopy,
    }),
  duplicate: (id, revision) =>
    request('duplicate_recipe', recipeSchema, { id, revision }),
  scope: (scope, kind) =>
    request('scope_recipes', z.array(recipeSchema), { scope, kind }),
  archive: (id, revision, archived) =>
    request('archive_recipe', recipeSchema, { id, revision, archived }),
  purge: (id, revision) => request('purge_recipe', z.null(), { id, revision }),
  searchAvailable: (query) => request('search_available_ingredients', ingredientSearchSchema, { query }),
  suggestAvailable: (query,mode) => request('ingredient_suggestions', ingredientSearchSchema, { query, mode }),
  searchPersonal: (query) =>
    request('search_personal_ingredients', ingredientSearchSchema, { query }),
  editIngredient: (input) =>
    request('edit_ingredient', personalIngredientSchema, { input }),
  deleteIngredient: (id, revision) =>
    request('delete_ingredient', z.null(), { id, revision }),
  units: () => request('list_units', z.array(unitSchema)),
};
