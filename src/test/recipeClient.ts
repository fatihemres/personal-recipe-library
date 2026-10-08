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
    list,
    scope: list,
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
