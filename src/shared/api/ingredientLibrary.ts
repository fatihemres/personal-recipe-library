import { z } from 'zod';
import { request } from './client';
import { libraryQuerySchema, libraryPageSchema, libraryDetailSchema, originSchema, type LibraryQuery, type LibraryPage, type LibraryDetail } from '../contracts/ingredientLibrary';
export interface IngredientLibraryClient {
    browse(query: LibraryQuery): Promise<LibraryPage>;
    detail(id: string, origin: 'catalog' | 'personal'): Promise<LibraryDetail>;
}
export const ingredientLibraryClient: IngredientLibraryClient = {
    browse: query => request('ingredient_library', libraryPageSchema, { query: libraryQuerySchema.parse(query) }),
    detail: (id, origin) => request('ingredient_detail', libraryDetailSchema, { id: z.uuid().parse(id), origin: originSchema.parse(origin) }),
};
