import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { LibraryPage } from './LibraryPage';
import { rendererRecipeClient } from '../../test/recipeClient';
import type { Draft } from '../../shared/contracts/recipe';
function draft(): Draft {
  return {
    id: crypto.randomUUID(),
    revision: 4,
    recipeId: null,
    updatedAt: '2026-10-08T12:00:00Z',
    ingredientNames: {},
    input: {
      id: crypto.randomUUID(),
      expectedRevision: null,
      title: 'Kurtarılan çorba',
      description: null,
      kind: 'food',
      servings: '1.',
      prepMinutes: null,
      cookMinutes: null,
      notes: null,
      ingredients: [],
      steps: [{ id: crypto.randomUUID(), instructions: '' }],
    },
  };
}
test('recovery restores unfinished fields and keep-draft navigation flushes SQLite client', async () => {
  const api = rendererRecipeClient();
  const recovered = draft();
  vi.mocked(api.drafts).mockResolvedValue([recovered]);
  vi.mocked(api.getDraft).mockResolvedValue(recovered);
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(
    await screen.findByRole('button', { name: 'Düzenlemeye devam et' }),
  );
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('Kurtarılan çorba');
  expect(screen.getByLabelText('Porsiyon sayısı')).toHaveValue('1.');
  expect(screen.getByLabelText('1. Talimat')).toHaveValue('');
  await user.type(screen.getByLabelText('Tarif adı'), ' düzenleme');
  await user.click(
    screen.getByRole('button', { name: 'Taslağı sakla ve çık' }),
  );
  await screen.findByRole('heading', { name: 'Tarif kütüphanesi' });
  expect(api.saveDraft).toHaveBeenCalledWith(
    recovered.id,
    4,
    expect.objectContaining({ title: 'Kurtarılan çorba düzenleme' }),
  );
  expect(api.commit).not.toHaveBeenCalled();
});
test('discard recovery requires confirmation and uses the displayed draft revision', async () => {
  const api = rendererRecipeClient();
  const recovered = draft();
  vi.mocked(api.drafts).mockResolvedValue([recovered]);
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(await screen.findByRole('button', { name: 'Taslağı sil' }));
  expect(api.discardDraft).not.toHaveBeenCalled();
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', {
      name: 'Taslağı sil',
    }),
  );
  await waitFor(() =>
    expect(api.discardDraft).toHaveBeenCalledWith(recovered.id, 4),
  );
});
