import { expect, test } from 'vitest';
import { decimalSchema, recipeInputSchema, unitSchema } from './recipe';
test('decimal quantities preserve precision and reject unsafe representations', () => {
  for (const value of ['2.50', '0.000001', '999999999999.123456'])
    expect(decimalSchema.parse(value)).toBe(value);
  for (const value of ['0', '-1', '01', '1e3', '1,5', '1.0000001', '1.2.3'])
    expect(decimalSchema.safeParse(value).success).toBe(false);
});
test('recipe contract preserves Turkish text and unknown quantities', () => {
  const draft = {
    id: crypto.randomUUID(),
    expectedRevision: null,
    title: 'İçli köfte',
    description: null,
    kind: 'food',
    servings: '2.5',
    prepMinutes: null,
    cookMinutes: null,
    notes: null,
    ingredients: [
      {
        id: crypto.randomUUID(),
        ingredientId: crypto.randomUUID(),
        quantity: null,
        unitCode: 'cc',
        note: null,
      },
    ],
    steps: [],
  };
  expect(recipeInputSchema.parse(draft).title).toBe('İçli köfte');
  expect(recipeInputSchema.safeParse({ ...draft, servings: '0' }).success).toBe(
    false,
  );
  expect(
    unitSchema.safeParse({
      code: 'cc',
      dimension: 'volume',
      canonicalCode: 'mL',
      factor: 1,
    }).success,
  ).toBe(true);
});
