import { render, screen, waitFor } from '@testing-library/react';
import { StrictMode } from 'react';
import userEvent from '@testing-library/user-event';
import { afterEach, expect, test, vi } from 'vitest';
import { getVersion } from '@tauri-apps/api/app';
import { AppShell } from './AppShell';
import type { FoundationClient } from '../shared/api/client';
import type { Bootstrap } from '../shared/contracts/foundation';
import { rendererRecipeClient } from '../test/recipeClient';

vi.mock('@tauri-apps/api/app', () => ({ getVersion: vi.fn() }));
vi.mock('../shared/api/catalog', () => ({ catalogClient: { status: vi.fn().mockResolvedValue({ definitions: 0, validationDefinitions: 0, productionDefinitions: 0, pendingCollisions: 0 }) } }));

const data: Bootstrap = { preferences: { theme: 'light', locale: 'tr' }, storage: { schemaVersion: 5, sqliteVersion: '3.53.0', foreignKeys: true, fts5: true } };
const client: FoundationClient = { bootstrap: vi.fn().mockResolvedValue(data), savePreferences: vi.fn().mockImplementation(async value => value) };

afterEach(() => vi.clearAllMocks());

test('one Tauri metadata request supplies sidebar and Settings without a stale milestone label', async () => {
  vi.mocked(getVersion).mockResolvedValue('0.1.0');
  const user = userEvent.setup();
  render(<StrictMode><AppShell client={client} recipes={rendererRecipeClient()} /></StrictMode>);
  expect(await screen.findByText('Tarif Atlası · v0.1.0')).toBeVisible();
  expect(screen.queryByText('Aşama 3C-3')).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Ayarlar' }));
  expect(screen.getByRole('heading', { name: 'Uygulama Hakkında' })).toBeVisible();
  expect(screen.getByText('Uygulama sürümü').nextElementSibling).toHaveTextContent('v0.1.0');
  await user.click(screen.getByRole('button', { name: 'Malzeme Kütüphanesi' }));
  await user.click(screen.getByRole('button', { name: 'Ayarlar' }));
  expect(getVersion).toHaveBeenCalledTimes(1);
});

test('version failure is accessible and independent of failed SQLite bootstrap', async () => {
  vi.mocked(getVersion).mockRejectedValue(new Error('metadata unavailable'));
  const broken: FoundationClient = { ...client, bootstrap: vi.fn().mockRejectedValue({ code: 'STORAGE_UNAVAILABLE', messageKey: 'errors.storage', recoverable: true }) };
  render(<AppShell client={broken} recipes={rendererRecipeClient()} />);
  await waitFor(() => expect(screen.getByText('Tarif Atlası · Sürüm bilgisi alınamadı')).toBeVisible());
  expect(screen.getByRole('button', { name: 'Ayarlar' })).toBeEnabled();
  expect(screen.getByRole('alert')).toHaveTextContent('Depolama açılamadı');
  await userEvent.setup().click(screen.getByRole('button', { name: 'Ayarlar' }));
  expect(screen.getByRole('heading', { name: 'Uygulama Hakkında' })).toBeVisible();
  expect(getVersion).toHaveBeenCalledTimes(1);
});
