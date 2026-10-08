import { expect, test } from 'vitest';
import { recipeDraftSchema, unitSchema } from './recipe';
test('future recipe contract preserves Turkish text and unknown quantity', () => {
  const draft = { kind: 'food', title: 'İçli köfte', yield: '2', ingredients: [{ ingredientId: 'bba0d198-7d84-470e-9f29-b53e85a88be3', quantity: null, unitCode: 'g' }], steps: [] };
  expect(recipeDraftSchema.parse(draft).title).toBe('İçli köfte');
  expect(recipeDraftSchema.safeParse({ ...draft, yield: '0' }).success).toBe(false);
  expect(recipeDraftSchema.safeParse({ ...draft, yield: '2,5' }).success).toBe(false);
  expect(unitSchema.safeParse({ code: 'mL', dimension: 'volume' }).success).toBe(true);
});
