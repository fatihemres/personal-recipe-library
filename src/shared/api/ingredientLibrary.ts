import { z } from 'zod';
import { request } from './client';
import { libraryQuerySchema, libraryPageSchema, libraryDetailSchema, originSchema, type LibraryQuery, type LibraryPage, type LibraryDetail, catalogPresentationSchema, ingredientCandidateSchema, type CatalogPresentation, type IngredientCandidate } from '../contracts/ingredientLibrary';
export interface IngredientLibraryClient {
    browse(query: LibraryQuery): Promise<LibraryPage>;
    favorite(id:string, origin:'catalog'|'personal', favorite:boolean):Promise<boolean>;
    presentation(input:CatalogPresentation):Promise<LibraryDetail>;
    candidates(name:string):Promise<IngredientCandidate[]>;
    detail(id: string, origin: 'catalog' | 'personal'): Promise<LibraryDetail>;
}
export const ingredientLibraryClient: IngredientLibraryClient = {
    browse: query => request('ingredient_library', libraryPageSchema, { query: libraryQuerySchema.parse(query) }),
    favorite: (id, origin, favorite) => request('set_ingredient_favorite', z.boolean(), { id:z.uuid().parse(id), origin:originSchema.parse(origin), favorite }),
    presentation: input => request('set_catalog_presentation', libraryDetailSchema, { input:catalogPresentationSchema.parse(input) }),
    candidates: name => request('ingredient_candidates', z.array(ingredientCandidateSchema), { name:z.string().max(200).parse(name) }),
    detail: (id, origin) => request('ingredient_detail', libraryDetailSchema, { id: z.uuid().parse(id), origin: originSchema.parse(origin) }),
};
