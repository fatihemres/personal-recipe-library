import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { IngredientSearchControl } from './IngredientSearchControl';
import { rendererRecipeClient } from '../../test/recipeClient';
test('Turkish search uses actual requested text and supports arrow/Enter selection', async () => {
  const api = rendererRecipeClient();
  const sugar = {
    id: crypto.randomUUID(),
    name: 'Şeker',
    revision: 1,
    notes: null,
    preferredUnit: 'g',
    origin: 'personal' as const,
  };
  vi.mocked(api.searchPersonal).mockResolvedValue({
    items: [sugar],
    total: 1,
    hasMore: false,
  });
  const select = vi.fn();
  const user = userEvent.setup();
  render(<IngredientSearchControl client={api} onSelect={select} />);
  await user.type(screen.getByRole('combobox'), 'ŞEK');
  expect(await screen.findByRole('option', { name: 'Şeker' })).toBeVisible();
  expect(api.searchPersonal).toHaveBeenLastCalledWith('ŞEK');
  await user.keyboard('{ArrowDown}{Enter}');
  expect(select).toHaveBeenCalledWith(sugar);
});
test('empty catalog, unmatched search and failure are distinct and missing ingredient can be created', async () => {
  const api = rendererRecipeClient();
  vi.mocked(api.searchPersonal).mockResolvedValue({
    items: [],
    total: 0,
    hasMore: false,
  });
  const user = userEvent.setup();
  const select = vi.fn();
  render(<IngredientSearchControl client={api} onSelect={select} />);
  expect(
    await screen.findByText(/Henüz kişisel malzeme kaydetmediniz/),
  ).toBeVisible();
  await user.type(screen.getByRole('combobox'), 'Şeker');
  await waitFor(() =>
    expect(api.searchPersonal).toHaveBeenLastCalledWith('Şeker'),
  );
  await user.click(
    screen.getByRole('button', {
      name: 'Kişisel malzeme oluştur / mevcut olanı seç',
    }),
  );
  expect(api.createIngredient).toHaveBeenCalledWith('Şeker');
  expect(select).toHaveBeenCalled();
  vi.mocked(api.searchPersonal).mockResolvedValue({
    items: [],
    total: 3,
    hasMore: false,
  });
  await user.type(screen.getByRole('combobox'), 'x');
  expect(await screen.findByText(/Bu ada uyan kayıt yok/)).toBeVisible();
  vi.mocked(api.searchPersonal).mockRejectedValue({
    code: 'STORAGE_BUSY',
    messageKey: 'errors.busy',
    recoverable: true,
  });
  await user.type(screen.getByRole('combobox'), 'y');
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'katalog boş demek değildir',
  );
  expect(screen.queryByText(/Bu ada uyan kayıt yok/)).not.toBeInTheDocument();
});
test('mouse selection returns keyboard focus to search and closes the suggestions', async () => {
  const api = rendererRecipeClient();
  const select = vi.fn();
  const user = userEvent.setup();
  render(<IngredientSearchControl client={api} onSelect={select} />);
  await user.click(await screen.findByRole('option', { name: 'İçme suyu' }));
  expect(select).toHaveBeenCalledOnce();
  expect(screen.getByRole('combobox')).toHaveFocus();
  expect(screen.getByRole('combobox')).toHaveAttribute('aria-expanded', 'false');
});
