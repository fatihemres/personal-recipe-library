import { beforeEach, expect, test, vi } from 'vitest';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { catalogClient } from './catalog';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }));
beforeEach(() => { vi.mocked(isTauri).mockReturnValue(true); vi.mocked(invoke).mockReset(); });
test('catalog status separates validation from production and validates IPC', async () => {
  vi.mocked(invoke).mockResolvedValue({ definitions: 8, validationDefinitions: 8, productionDefinitions: 0, pendingCollisions: 1 });
  expect((await catalogClient.status()).productionDefinitions).toBe(0);
  expect(invoke).toHaveBeenCalledWith('catalog_status', undefined);
  vi.mocked(invoke).mockResolvedValue({ definitions: -1 });
  await expect(catalogClient.status()).rejects.toMatchObject({ code: 'INVALID_RESPONSE' });
});
test('category creation rejects invalid input before invoking and validates identifiers', async () => {
  expect(() => catalogClient.createCategory('', 'Custom', null)).toThrow();
  expect(invoke).not.toHaveBeenCalled();
  vi.mocked(invoke).mockResolvedValue('ec613355-c84c-4d83-ab1a-e34bc3cae3dc');
  await expect(catalogClient.createCategory('Özel', 'Custom', null)).resolves.toBe('ec613355-c84c-4d83-ab1a-e34bc3cae3dc');
});
