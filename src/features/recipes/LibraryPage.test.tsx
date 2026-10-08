import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { LibraryPage } from './LibraryPage';
import { rendererRecipeClient } from '../../test/recipeClient';
import type { Recipe } from '../../shared/contracts/recipe';
// Renderer tests isolate interaction; real SQLite and native tests independently prove persistence.
const client = rendererRecipeClient;
test('create recipe with personal ingredient, decimal comma and reordered steps', async () => {
  const api = client();
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  expect(
    await screen.findByRole('heading', { name: 'Henüz tarif yok' }),
  ).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Yeni tarif' }));
  await user.type(screen.getByLabelText('Tarif adı'), 'İçli köfte');
  await user.clear(screen.getByLabelText('Porsiyon sayısı'));
  await user.type(screen.getByLabelText('Porsiyon sayısı'), '2,50');
  await user.type(screen.getByLabelText('Malzeme ara'), 'İçme suyu');
  await user.click(
    screen.getByRole('button', {
      name: 'Kişisel malzeme oluştur / mevcut olanı seç',
    }),
  );
  await user.type(await screen.findByLabelText('Miktar'), '0,123456');
  await user.selectOptions(screen.getByLabelText('Birim'), 'cc');
  await user.click(screen.getByRole('button', { name: 'Adım ekle' }));
  await user.type(screen.getByLabelText('1. Talimat'), 'Hazırla');
  await user.click(screen.getByRole('button', { name: 'Adım ekle' }));
  await user.type(screen.getByLabelText('2. Talimat'), 'Pişir');
  await user.click(screen.getAllByRole('button', { name: 'Yukarı taşı' })[1]);
  await user.click(screen.getByRole('button', { name: 'Tarifi kaydet' }));
  await screen.findByRole('heading', { name: 'İçli köfte' });
  expect(api.save).toHaveBeenCalledWith(
    expect.objectContaining({
      title: 'İçli köfte',
      servings: '2.50',
      ingredients: [
        expect.objectContaining({ quantity: '0.123456', unitCode: 'cc' }),
      ],
      steps: [
        expect.objectContaining({ instructions: 'Pişir' }),
        expect.objectContaining({ instructions: 'Hazırla' }),
      ],
    }),
  );
  expect(screen.getByText(/0.123456 cc/)).toBeVisible();
  expect(screen.getByText('İçme suyu')).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Düzenle' }));
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('İçli köfte');
});
test('safe deletion requires confirmation and trash offers restore', async () => {
  const api = client();
  const recipe: Recipe = {
    id: crypto.randomUUID(),
    revision: 1,
    title: 'Ayran',
    kind: 'beverage',
    servings: '1',
    description: null,
    prepMinutes: 1,
    cookMinutes: null,
    notes: null,
    createdAt: 'now',
    updatedAt: 'now',
    deletedAt: null,
    archivedAt: null,
    ingredients: [],
    steps: [],
    ingredientNames: {},
  };
  vi.mocked(api.list).mockResolvedValue([recipe]);
  vi.mocked(api.get).mockResolvedValue(recipe);
  vi.mocked(api.setDeleted).mockResolvedValue({
    ...recipe,
    deletedAt: 'now',
    revision: 2,
  });
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(await screen.findByRole('button', { name: /Ayran/ }));
  await user.click(screen.getByRole('button', { name: 'Çöp kutusuna taşı' }));
  expect(api.setDeleted).not.toHaveBeenCalled();
  const dialog = screen.getByRole('dialog');
  await user.click(within(dialog).getByRole('button', { name: 'Vazgeç' }));
  expect(api.setDeleted).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Çöp kutusuna taşı' }));
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', {
      name: 'Çöp kutusuna taşı',
    }),
  );
  await waitFor(() =>
    expect(api.setDeleted).toHaveBeenCalledWith(recipe.id, 1, true),
  );
  vi.mocked(api.get).mockResolvedValue({
    ...recipe,
    deletedAt: 'now',
    revision: 2,
  });
  await user.click(screen.getByRole('button', { name: 'Çöp kutusu' }));
  await user.click(await screen.findByRole('button', { name: /Ayran/ }));
  await user.click(screen.getByRole('button', { name: 'Geri yükle' }));
  await waitFor(() =>
    expect(api.setDeleted).toHaveBeenCalledWith(recipe.id, 2, false),
  );
});
test('save errors keep unfinished edits and cancel requires explicit discard', async () => {
  const api = client();
  vi.mocked(api.save).mockRejectedValue({
    code: 'STORAGE_BUSY',
    messageKey: 'errors.busy',
    recoverable: true,
  });
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(screen.getByRole('button', { name: 'Yeni tarif' }));
  await user.type(screen.getByLabelText('Tarif adı'), 'Çorba');
  await user.click(screen.getByRole('button', { name: 'Tarifi kaydet' }));
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'Veritabanı başka',
  );
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('Çorba');
  await user.click(screen.getByRole('button', { name: 'Vazgeç' }));
  expect(screen.getByRole('dialog')).toBeVisible();
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', {
      name: 'Düzenlemeye devam et',
    }),
  );
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('Çorba');
});
test('archive scopes, duplication and permanent deletion are explicit UI actions', async () => {
  const api = client();
  const recipe: Recipe = {
    id: crypto.randomUUID(), revision: 1, title: 'Çay', kind: 'beverage',
    servings: '1', description: null, prepMinutes: null, cookMinutes: null,
    notes: null, createdAt: 'now', updatedAt: 'now', deletedAt: null,
    archivedAt: null, ingredients: [], steps: [], ingredientNames: {},
  };
  vi.mocked(api.scope).mockResolvedValue([recipe]);
  vi.mocked(api.get).mockResolvedValue(recipe);
  vi.mocked(api.archive).mockResolvedValue({ ...recipe, revision: 2, archivedAt: 'now' });
  vi.mocked(api.duplicate).mockResolvedValue({ ...recipe, id: crypto.randomUUID() });
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(await screen.findByRole('button', { name: /Çay/ }));
  await user.click(screen.getByRole('button', { name: 'Arşivle' }));
  await screen.findByRole('button', { name: 'Arşivden çıkar' });
  expect(api.archive).toHaveBeenCalledWith(recipe.id, 1, true);
  await user.click(screen.getByRole('button', { name: 'Tarifi çoğalt' }));
  await screen.findByText('Tarif bağımsız bir kopya olarak oluşturuldu.');
  expect(api.duplicate).toHaveBeenCalledWith(recipe.id, 2);
  await user.click(screen.getByRole('button', { name: 'Kütüphaneye dön' }));
  await user.click(screen.getByRole('button', { name: 'Arşiv' }));
  await waitFor(() => expect(api.scope).toHaveBeenCalledWith('archived', null));
  vi.mocked(api.get).mockResolvedValue({ ...recipe, deletedAt: 'now', revision: 3 });
  await user.click(screen.getByRole('button', { name: 'Çöp kutusu' }));
  await user.click(await screen.findByRole('button', { name: /Çay/ }));
  await user.click(screen.getByRole('button', { name: 'Kalıcı olarak sil' }));
  expect(api.purge).not.toHaveBeenCalled();
  expect(screen.getByRole('dialog')).toHaveTextContent('geri alınamaz');
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Vazgeç' }));
  expect(api.purge).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Kalıcı olarak sil' }));
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Kalıcı olarak sil' }));
  await waitFor(() => expect(api.purge).toHaveBeenCalledWith(recipe.id, 3));
});
