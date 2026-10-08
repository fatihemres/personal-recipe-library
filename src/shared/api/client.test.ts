import { beforeEach, expect, test, vi } from 'vitest';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { foundationClient, normalizeError } from './client';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }));
beforeEach(() => vi.mocked(isTauri).mockReturnValue(true));
test('uses registered command and validates bootstrap response', async () => {
  vi.mocked(invoke).mockResolvedValue({ preferences: { theme: 'dark', locale: 'tr' }, storage: { schemaVersion: 1, sqliteVersion: '3.53', foreignKeys: true, fts5: true } });
  expect((await foundationClient.bootstrap()).preferences.theme).toBe('dark');
  expect(invoke).toHaveBeenCalledWith('bootstrap', undefined);
});
test('invalid native response is rejected', async () => {
  vi.mocked(invoke).mockResolvedValue({ preferences: { theme: 'purple' } });
  await expect(foundationClient.bootstrap()).rejects.toMatchObject({ code: 'INVALID_RESPONSE' });
});
test('validates preference input before IPC and sends exact arguments', async () => {
  vi.mocked(invoke).mockClear();
  await expect(foundationClient.savePreferences({ theme: 'dark', locale: 'en' } as never)).rejects.toMatchObject({ code: 'INVALID_INPUT' });
  expect(invoke).not.toHaveBeenCalled();
  vi.mocked(invoke).mockResolvedValue({ theme: 'light', locale: 'tr' });
  await expect(foundationClient.savePreferences({ theme: 'light', locale: 'tr' })).resolves.toEqual({ theme: 'light', locale: 'tr' });
  expect(invoke).toHaveBeenCalledWith('save_preferences', { preferences: { theme: 'light', locale: 'tr' } });
});
test('browser preview cannot silently pretend to persist', async () => {
  vi.mocked(isTauri).mockReturnValue(false);
  await expect(foundationClient.bootstrap()).rejects.toMatchObject({ code: 'DESKTOP_REQUIRED' });
});
test('error normalization preserves safe structured errors', () => {
  expect(normalizeError({ code: 'SCHEMA_TOO_NEW', messageKey: 'errors.newerSchema', recoverable: false }).code).toBe('SCHEMA_TOO_NEW');
  expect(normalizeError(new Error('/private/user/path'))).toEqual({ code: 'UNKNOWN', messageKey: 'errors.unknown', recoverable: true });
});
