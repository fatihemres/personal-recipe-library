import { beforeEach, expect, test, vi } from 'vitest';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { ingredientLibraryClient } from './ingredientLibrary';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }));
beforeEach(() => { vi.mocked(isTauri).mockReturnValue(true); vi.mocked(invoke).mockReset(); });
test('paged library IPC validates requests and real DTO shape boundaries', async () => {
  const query = { search: 'ŞEK', origin: 'all' as const, categoryId: null, offset: 30, limit: 30 };
  vi.mocked(invoke).mockResolvedValue({ items: [], total: 0, catalogCount: 508, personalCount: 0, categories: [], offset: 30, limit: 30 });
  expect((await ingredientLibraryClient.browse(query)).catalogCount).toBe(508);
  expect(invoke).toHaveBeenCalledWith('ingredient_library', { query });
  expect(() => ingredientLibraryClient.browse({ ...query, limit: 101 })).toThrow();
  vi.mocked(invoke).mockResolvedValue({ items: [], total: -1 });
  await expect(ingredientLibraryClient.browse(query)).rejects.toMatchObject({ code: 'INVALID_RESPONSE' });
});
test('detail IPC rejects malformed identifiers and incomplete responses', async () => {
  expect(() => ingredientLibraryClient.detail('bad', 'catalog')).toThrow();
  expect(invoke).not.toHaveBeenCalled();
  vi.mocked(invoke).mockResolvedValue({ item: { origin: 'catalog' } });
  await expect(ingredientLibraryClient.detail(crypto.randomUUID(), 'catalog')).rejects.toMatchObject({ code: 'INVALID_RESPONSE' });
});
