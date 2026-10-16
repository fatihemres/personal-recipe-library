import { act, renderHook } from '@testing-library/react';
import { afterEach, expect, test, vi } from 'vitest';
import type { Draft, RecipeInput } from '../../shared/contracts/recipe';
import { rendererRecipeClient } from '../../test/recipeClient';
import { DraftSession } from './DraftSession';
import { useDurableDraft } from './useDurableDraft';

afterEach(() => { vi.useRealTimers(); vi.restoreAllMocks(); });

test('a stalled SQLite save does not accumulate periodic draft requests', async () => {
  vi.useFakeTimers();
  const client = rendererRecipeClient();
  const input: RecipeInput = { id: crypto.randomUUID(), expectedRevision: null, title: 'Taslak', kind: 'food', servings: '1', description: null, prepMinutes: null, cookMinutes: null, notes: null, ingredients: [], steps: [] };
  let finish!: (draft: Draft) => void;
  vi.mocked(client.saveDraft).mockImplementation(() => new Promise<Draft>(resolve => { finish = resolve; }));
  const flush = vi.spyOn(DraftSession.prototype, 'flush');
  const view = renderHook(() => useDurableDraft(client, input, true));
  await act(async () => { vi.advanceTimersByTime(9000); await Promise.resolve(); });
  expect(flush).toHaveBeenCalledTimes(2); // one debounce and one in-flight periodic request
  expect(client.saveDraft).toHaveBeenCalledTimes(1);
  await act(async () => {
    finish({ id: 'draft', revision: 1, recipeId: null, updatedAt: 'now', input, ingredientNames: {} });
    await Promise.resolve();
  });
  await act(async () => { vi.advanceTimersByTime(2000); await Promise.resolve(); });
  expect(flush).toHaveBeenCalledTimes(3); // periodic checks resume after settlement
  view.unmount();
});
