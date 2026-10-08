import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { LibraryPage } from './LibraryPage';
import type { RecipeClient } from '../../shared/api/recipes';
import type { Recipe } from '../../shared/contracts/recipe';
// Renderer tests isolate interaction; real SQLite and native tests independently prove persistence.
function client(): RecipeClient {
  const ingredient = { id: crypto.randomUUID(), name: 'İçme suyu' };
  return {
    list: vi.fn().mockResolvedValue([]),
    get: vi.fn(),
    save: vi.fn().mockImplementation(async (input) => ({
      ...input,
      expectedRevision: undefined,
      revision: 1,
      createdAt: 'now',
      updatedAt: 'now',
      deletedAt: null,
      ingredientNames: { [ingredient.id]: ingredient.name },
    })),
    setDeleted: vi.fn(),
    searchIngredients: vi.fn().mockResolvedValue([ingredient]),
    createIngredient: vi.fn().mockResolvedValue(ingredient),
    units: vi.fn().mockResolvedValue([
      { code: 'cc', dimension: 'volume', canonicalCode: 'mL', factor: 1 },
      { code: 'g', dimension: 'mass', canonicalCode: 'g', factor: 1 },
    ]),
  };
}
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
    within(screen.getByRole('dialog')).getByRole('button', { name: 'Vazgeç' }),
  );
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('Çorba');
});
