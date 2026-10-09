import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { IngredientLibrary } from './IngredientLibrary';
import type { IngredientLibraryClient } from '../../shared/api/ingredientLibrary';
import type { LibraryItem, LibraryPage, LibraryDetail } from '../../shared/contracts/ingredientLibrary';
import { rendererRecipeClient } from '../../test/recipeClient';
// Renderer-only fixtures. Real data/counts/persistence are verified by Rust and native checks.
const item: LibraryItem = { id: crypto.randomUUID(), origin: 'catalog', name: 'Toz şeker', englishName: 'Granulated sugar', categories: [], recipeId: null, notes: null, preferredUnit: 'g', revision: null, ingredientType: 'food' };
const category = { id: crypto.randomUUID(), parentId: null, key: 'sweeteners', tr: 'Tatlandırıcılar', en: 'Sweeteners' };
const page: LibraryPage = { items: [item], total: 508, catalogCount: 508, personalCount: 0, categories: [{ category, count: 7 }], offset: 0, limit: 30 };
const detail: LibraryDetail = { item, aliases: [{ locale: 'tr', name: 'Şeker' }], canonicalTr: 'Toz şeker', dimensions: ['mass'], sources: [], metadata: [], catalogVersion: 4 };
function api(): IngredientLibraryClient { return { browse: vi.fn().mockResolvedValue(page), detail: vi.fn().mockResolvedValue(detail) }; }
test('view, category and search filters compose; paging is delegated to IPC', async () => {
  const library = api(); const user = userEvent.setup(); render(<IngredientLibrary client={rendererRecipeClient()} library={library} />);
  await screen.findByText('508 eşleşme');
  await user.click(screen.getByRole('button', { name: 'Hazır Katalog 508' }));
  await waitFor(() => expect(screen.getByRole('combobox')).toBeEnabled());
  await user.selectOptions(screen.getByRole('combobox'), category.id);
  await user.type(screen.getByRole('searchbox'), 'ŞEK');
  await waitFor(() => expect(library.browse).toHaveBeenLastCalledWith({ search: 'ŞEK', origin: 'catalog', categoryId: category.id, offset: 0, limit: 30 }));
  await user.click(screen.getByRole('button', { name: 'Sonraki sayfa' }));
  await waitFor(() => expect(library.browse).toHaveBeenLastCalledWith(expect.objectContaining({ offset: 30 })));
  await user.click(screen.getByRole('button', { name: 'Filtreleri temizle' }));
  expect(screen.getByRole('searchbox')).toHaveValue('');
});
test('catalog details are readonly and explicitly distinguish unknown metadata', async () => {
  const library = api(); const user = userEvent.setup(); render(<IngredientLibrary client={rendererRecipeClient()} library={library} />);
  await user.click(await screen.findByRole('button', { name: /Toz şeker/ }));
  expect(await screen.findByRole('heading', { name: 'Toz şeker' })).toBeVisible();
  expect(screen.getByText(/Bilgi eksikliği sıfır değer/)).toBeVisible();
  expect(screen.queryByRole('button', { name: 'Düzenle' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Kütüphaneye dön' }));
  expect(screen.getByRole('searchbox')).toBeVisible();
});
test('database failures remain separate from empty search results and allow retry', async () => {
  const library = api(); vi.mocked(library.browse).mockRejectedValueOnce({ code: 'STORAGE_UNAVAILABLE', messageKey: 'errors.storage', recoverable: true });
  const user = userEvent.setup(); render(<IngredientLibrary client={rendererRecipeClient()} library={library} />);
  expect(await screen.findByRole('alert')).toHaveTextContent('Depolama açılamadı');
  expect(screen.queryByText('Eşleşen malzeme yok')).not.toBeInTheDocument();
  vi.mocked(library.browse).mockResolvedValue({ ...page, items: [], total: 0 });
  await user.click(screen.getByRole('button', { name: 'Yeniden dene' }));
  expect(await screen.findByRole('heading', { name: 'Eşleşen malzeme yok' })).toBeVisible();
  expect(screen.getAllByRole('button', { name: 'Kişisel malzeme ekle' })).toHaveLength(2);
});
test('personal details expose revision-safe editing and conflict errors retain entered text', async () => {
  const personal = { ...item, origin: 'personal' as const, revision: 2, recipeId: item.id };
  const library = api(); vi.mocked(library.detail).mockResolvedValue({ ...detail, item: personal });
  const client = rendererRecipeClient(); vi.mocked(client.editIngredient).mockRejectedValue({ code: 'CONFLICT', messageKey: 'errors.conflict', recoverable: true });
  const user = userEvent.setup(); render(<IngredientLibrary client={client} library={library} />);
  await user.click(await screen.findByRole('button', { name: /Toz şeker/ }));
  await user.click(await screen.findByRole('button', { name: 'Düzenle' }));
  const name = screen.getByRole('textbox', { name: 'Malzeme adı' }); await user.clear(name); await user.type(name, 'Aile şekerim');
  await user.click(screen.getByRole('button', { name: 'Malzemeyi kaydet' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('başka bir işlemde değiştirildi');
  expect(name).toHaveValue('Aile şekerim');
  expect(client.editIngredient).toHaveBeenCalledWith(expect.objectContaining({ id: item.id, revision: 2, name: 'Aile şekerim' }));
});
