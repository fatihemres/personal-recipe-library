import { vi } from 'vitest';
import type { RecipeClient } from '../shared/api/recipes';
// Renderer-only interaction fixture. Real Rust/SQLite tests prove persistence.
export function rendererRecipeClient(): RecipeClient {
  const ingredient = {
    id: crypto.randomUUID(),
    name: 'İçme suyu',
    notes: null,
    preferredUnit: null,
    revision: 1,
    origin: 'personal' as const,
  };
  const list = vi.fn().mockResolvedValue([]);
  const save = vi
    .fn()
    .mockImplementation(async (input) => ({
      ...input,
      expectedRevision: undefined,
      revision: 1,
      createdAt: '2026-10-08T12:00:00Z',
      updatedAt: '2026-10-08T12:00:00Z',
      deletedAt: null,
      archivedAt: null,
      ingredientNames: { [ingredient.id]: ingredient.name },
    }));
  return {
    getPrivateNote: vi.fn().mockImplementation(async recipeId => ({recipeId,formatVersion:1,markdown:'',revision:0,updatedAt:null})),
    savePrivateNote: vi.fn().mockImplementation(async write => ({recipeId:write.recipeId,formatVersion:1,markdown:write.markdown,revision:write.expectedRevision+1,updatedAt:'2026-10-08T12:00:00Z'})),
    savedFilters: vi.fn().mockResolvedValue([]),
    resolveFilter: vi.fn(),
    saveFilter: vi.fn().mockImplementation(async write => ({id:write.id,title:write.title,revision:(write.expectedRevision??0)+1,formatVersion:1,query:write.query,available:true})),
    deleteFilter: vi.fn().mockResolvedValue(null),
    dashboard: vi.fn().mockResolvedValue({activeTotal:0,foodCount:0,beverageCount:0,favoriteCount:0,recentCreated:[],recentModified:[],collections:[]}),
    getOrganization: vi.fn().mockImplementation(async (id) => ({recipeId:id,revision:1,favorite:false,toTry:false,rating:null,collectionIds:[]})),
    setOrganization: vi.fn().mockImplementation(async (write) => ({recipeId:write.recipeId,revision:write.expectedRevision+1,favorite:write.favorite,toTry:write.toTry,rating:write.rating,collectionIds:[]})),
    collections: vi.fn().mockResolvedValue([]),
    saveCollection: vi.fn().mockImplementation(async (write) => ({id:write.id,title:write.title,revision:1,activeCount:0})),
    deleteCollection: vi.fn().mockResolvedValue(null),
    setMembership: vi.fn().mockImplementation(async (recipeId,collectionId,expectedRevision,member) => ({recipeId,revision:expectedRevision+1,favorite:false,toTry:false,rating:null,collectionIds:member?[collectionId]:[]})),
    list,
    scope: list,
    searchRecipes: vi.fn().mockImplementation(async (query) => {
      const recipes = await list(query.scope, query.kind);
      const matches = recipes.filter((recipe: {title: string; kind: string; id: string; favorite?:boolean; toTry?:boolean}) =>
        (!query.kind || recipe.kind === query.kind)
        && (!query.favoriteOnly || recipe.favorite)
        && (!query.toTryOnly || recipe.toTry)
        && recipe.title.toLocaleLowerCase('tr').includes(query.text.toLocaleLowerCase('tr')),
      );
      return { items: matches.slice(query.offset, query.offset + query.limit).map((recipe:{favorite?:boolean;toTry?:boolean;rating?:number|null;organizationRevision?:number})=>({organizationRevision:recipe.organizationRevision??1,favorite:recipe.favorite??false,toTry:recipe.toTry??false,rating:recipe.rating??null,...recipe})), total: matches.length,
        hasMore: query.offset + query.limit < matches.length, suggestions: [] };
    }),
    get: vi.fn(),
    save,
    commit: vi.fn().mockImplementation((input) => save(input)),
    setDeleted: vi.fn(),
    searchIngredients: vi.fn().mockResolvedValue([ingredient]),
    createIngredient: vi.fn().mockResolvedValue(ingredient),
    units: vi.fn().mockResolvedValue([
      { code: 'cc', dimension: 'volume', canonicalCode: 'mL', factor: 1 },
      { code: 'g', dimension: 'mass', canonicalCode: 'g', factor: 1 },
    ]),
    drafts: vi.fn().mockResolvedValue([]),
    getDraft: vi.fn(),
    saveDraft: vi
      .fn()
      .mockImplementation(async (id, revision, input) => ({
        id,
        revision: (revision ?? 0) + 1,
        recipeId: input.expectedRevision === null ? null : input.id,
        updatedAt: '2026-10-08T12:00:00Z',
        input,
        ingredientNames: { [ingredient.id]: ingredient.name },
      })),
    discardDraft: vi.fn().mockResolvedValue(null),
    duplicate: vi.fn(),
    archive: vi.fn(),
    purge: vi.fn().mockResolvedValue(null),
    searchPersonal: vi
      .fn()
      .mockResolvedValue({ items: [ingredient], total: 1, hasMore: false }),
    editIngredient: vi.fn(),
    deleteIngredient: vi.fn().mockResolvedValue(null),
  };
}
