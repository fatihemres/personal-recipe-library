import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { AppShell } from './AppShell';
import type { FoundationClient } from '../shared/api/client';
import { rendererRecipeClient } from '../test/recipeClient';
import type { Bootstrap } from '../shared/contracts/foundation';
const recipes=rendererRecipeClient();
const data: Bootstrap = { preferences: { theme: 'light', locale: 'tr' }, storage: { schemaVersion: 1, sqliteVersion: '3.53.0', foreignKeys: true, fts5: true } };
function client(): FoundationClient { return { bootstrap: vi.fn().mockResolvedValue(data), savePreferences: vi.fn().mockImplementation(async (p) => p) }; }
// These injected clients isolate renderer behavior; Rust/native tests prove real persistence.
test('Turkish navigation, honest upcoming modules and library state', async () => {
  const user = userEvent.setup(); render(<AppShell recipes={recipes} client={client()} />);
  expect(await screen.findByRole('heading', { name: 'Tarif kütüphanesi' })).toBeVisible();
  expect(screen.getByText('Henüz kullanıma açık değil')).toBeVisible();
  expect(screen.queryByRole('button', { name: 'Tarif ekle' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Ayarlar' }));
  expect(screen.getByRole('heading', { name: 'Yerel depolama' })).toBeVisible();
  expect(screen.getByRole('main')).toHaveFocus();
});
test('theme selection persists through client and applies after successful save', async () => {
  const api = client(); const user = userEvent.setup(); render(<AppShell recipes={recipes} client={api} />);
  await screen.findByRole('heading', { name: 'Tarif kütüphanesi' });
  await user.click(screen.getByRole('button', { name: 'Ayarlar' }));
  await user.click(screen.getByRole('radio', { name: /Koyu/ }));
  await waitFor(() => expect(document.documentElement).toHaveClass('dark'));
  expect(api.savePreferences).toHaveBeenCalledWith({ theme: 'dark', locale: 'tr' });
  expect(screen.getByRole('radio', { name: /Koyu/ })).toBeChecked();
});
test('failed save retains previous preference and reports recoverable error', async () => {
  const api = client(); vi.mocked(api.savePreferences).mockRejectedValue({ code: 'STORAGE_UNAVAILABLE', messageKey: 'errors.storage', recoverable: true });
  const user = userEvent.setup(); render(<AppShell recipes={recipes} client={api} />);
  await screen.findByRole('heading', { name: 'Tarif kütüphanesi' }); await user.click(screen.getByRole('button', { name: 'Ayarlar' })); await user.click(screen.getByRole('radio', { name: /Koyu/ }));
  expect(await screen.findByRole('alert')).toHaveTextContent('Depolama açılamadı');
  expect(screen.getByRole('radio', { name: /Açık/ })).toBeChecked(); expect(document.documentElement).not.toHaveClass('dark');
});
test('loading is explicit, storage error retry recovers without fake library', async () => {
  let reject!: (reason: unknown) => void;
  const api = client(); vi.mocked(api.bootstrap).mockImplementationOnce(() => new Promise((_, fail) => { reject = fail; }));
  const user = userEvent.setup(); render(<AppShell recipes={recipes} client={api} />);
  expect(screen.getByRole('status')).toHaveTextContent('hazırlanıyor');
  await act(async () => reject({ code: 'STORAGE_UNAVAILABLE', messageKey: 'errors.storage', recoverable: true }));
  expect(screen.getByRole('alert')).toHaveTextContent('silinmedi');
  expect(screen.queryByRole('heading', { name: 'Tarif kütüphanesi' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Yeniden dene' }));
  expect(await screen.findByRole('heading', { name: 'Tarif kütüphanesi' })).toBeVisible();
});
test('persisted dark theme loads and keyboard reaches active navigation', async () => {
  const api = client(); vi.mocked(api.bootstrap).mockResolvedValue({ ...data, preferences: { theme: 'dark', locale: 'tr' } });
  const user = userEvent.setup(); render(<AppShell recipes={recipes} client={api} />);
  await screen.findByRole('heading', { name: 'Tarif kütüphanesi' }); expect(document.documentElement).toHaveClass('dark');
  await user.tab(); expect(screen.getByRole('link', { name: 'İçeriğe geç' })).toHaveFocus();
  await user.tab(); expect(screen.getByRole('button', { name: 'Tarif kütüphanesi' })).toHaveFocus();
});

test('system theme follows device appearance changes', async () => {
  let change!: () => void;
  const media = { matches: false, addEventListener: vi.fn((_event, listener) => { change = listener; }), removeEventListener: vi.fn() };
  vi.spyOn(window, 'matchMedia').mockReturnValue(media as unknown as MediaQueryList);
  const api = client(); vi.mocked(api.bootstrap).mockResolvedValue({ ...data, preferences: { theme: 'system', locale: 'tr' } });
  render(<AppShell recipes={recipes} client={api} />); await screen.findByRole('heading', { name: 'Tarif kütüphanesi' });
  expect(document.documentElement).not.toHaveClass('dark');
  media.matches = true; act(() => change()); expect(document.documentElement).toHaveClass('dark');
});

test('unfinished recipe survives settings navigation and storage retry', async () => {
  const api = client(); vi.mocked(api.savePreferences).mockRejectedValue({code:'STORAGE_BUSY',messageKey:'errors.busy',recoverable:true});
  const recipeApi={...recipes,units:vi.fn().mockResolvedValue([{code:'g',dimension:'mass',canonicalCode:'g',factor:1}])};
  const user=userEvent.setup();render(<AppShell client={api} recipes={recipeApi}/>);
  await screen.findByRole('heading',{name:'Tarif kütüphanesi'});await user.click(screen.getByRole('button',{name:'Yeni tarif'}));await user.type(screen.getByLabelText('Tarif adı'),'Kaydedilmemiş çorba');
  await user.click(screen.getByRole('button',{name:'Ayarlar'}));await user.click(screen.getByRole('radio',{name:/Koyu/}));
  await screen.findByRole('alert');await user.click(screen.getByRole('button',{name:'Yeniden dene'}));
  await waitFor(()=>expect(api.bootstrap).toHaveBeenCalledTimes(2));await user.click(screen.getByRole('button',{name:'Tarif kütüphanesi'}));
  expect(await screen.findByLabelText('Tarif adı')).toHaveValue('Kaydedilmemiş çorba');
});

vi.mock('../shared/api/catalog', () => ({ catalogClient: { status: vi.fn().mockResolvedValue({ definitions: 0, validationDefinitions: 0, productionDefinitions: 0, pendingCollisions: 0 }) } }));
