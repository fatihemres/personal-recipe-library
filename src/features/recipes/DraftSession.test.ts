import { expect, test, vi } from 'vitest';
import { DraftSession } from './DraftSession';
import { rendererRecipeClient } from '../../test/recipeClient';
import type { RecipeInput } from '../../shared/contracts/recipe';
function input(): RecipeInput {
  return {
    id: crypto.randomUUID(),
    expectedRevision: null,
    title: 'Taslak',
    kind: 'food',
    servings: '1',
    description: null,
    prepMinutes: null,
    cookMinutes: null,
    notes: null,
    ingredients: [],
    steps: [],
  };
}
test('autosave, explicit commit and late autosave are serialized without resurrection', async () => {
  const api = rendererRecipeClient();
  const value = input();
  let resolve!: () => void;
  const events: string[] = [];
  vi.mocked(api.saveDraft).mockImplementation(async (id, revision, payload) => {
    events.push(`write:${payload.title}`);
    if (payload.title === 'Bir')
      await new Promise<void>((done) => {
        resolve = done;
      });
    return {
      id,
      revision: (revision ?? 0) + 1,
      recipeId: null,
      updatedAt: 'now',
      input: payload,
      ingredientNames: {},
    };
  });
  vi.mocked(api.commit).mockImplementation(async (payload) => {
    events.push('commit');
    return {
      ...payload,
      revision: 1,
      createdAt: 'now',
      updatedAt: 'now',
      deletedAt: null,
      archivedAt: null,
      ingredientNames: {},
    };
  });
  const session = new DraftSession(api);
  const first = session.flush({ ...value, title: 'Bir' });
  await vi.waitFor(() => expect(events).toEqual(['write:Bir']));
  const commit = session.commit({ ...value, title: 'İki' });
  const late = session.flush({ ...value, title: 'Eski' });
  const lateCheck = expect(late).rejects.toMatchObject({
    code: 'DRAFT_CONFLICT',
  });
  resolve();
  await first;
  await commit;
  await lateCheck;
  expect(events).toEqual(['write:Bir', 'write:İki', 'commit']);
  expect(api.commit).toHaveBeenCalledWith(
    expect.objectContaining({ title: 'İki' }),
    session.id,
    2,
    false,
  );
});
test('discard waits for pending write and never permits a subsequent write', async () => {
  const api = rendererRecipeClient();
  const session = new DraftSession(api);
  const value = input();
  await session.flush(value);
  await session.discard();
  expect(api.discardDraft).toHaveBeenCalledWith(session.id, 1);
  await expect(session.flush({ ...value, title: 'Geç' })).rejects.toMatchObject(
    { code: 'DRAFT_CONFLICT' },
  );
  expect(api.saveDraft).toHaveBeenCalledTimes(1);
});
test('failed autosave remains retryable and does not invoke explicit commit', async () => {
  const api = rendererRecipeClient();
  vi.mocked(api.saveDraft).mockRejectedValueOnce({
    code: 'STORAGE_BUSY',
    messageKey: 'errors.busy',
    recoverable: true,
  });
  const session = new DraftSession(api);
  const value = input();
  await expect(session.flush(value)).rejects.toMatchObject({
    code: 'STORAGE_BUSY',
  });
  expect(api.commit).not.toHaveBeenCalled();
  await session.flush(value);
  expect(api.saveDraft).toHaveBeenLastCalledWith(session.id, null, value);
});
test('stale shared draft can be explicitly saved through an independent session', async () => {
  const api = rendererRecipeClient();
  const value = input();
  const recovered = {
    id: crypto.randomUUID(),
    revision: 1,
    recipeId: null,
    updatedAt: 'now',
    input: value,
    ingredientNames: {},
  };
  vi.mocked(api.saveDraft).mockRejectedValueOnce({
    code: 'DRAFT_CONFLICT',
    messageKey: 'errors.draftConflict',
    recoverable: true,
  });
  const session = new DraftSession(api, recovered);
  const result = await session.commit(
    { ...value, title: 'Benim metnim' },
    true,
  );
  expect(result.id).not.toBe(value.id);
  expect(api.discardDraft).not.toHaveBeenCalled();
  expect(api.commit).toHaveBeenCalledWith(
    expect.objectContaining({ title: 'Benim metnim', expectedRevision: null }),
    expect.not.stringMatching(recovered.id),
    1,
    false,
  );
});
