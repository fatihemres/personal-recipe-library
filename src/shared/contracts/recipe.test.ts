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

test('legacy drafts stay readable and advanced committed values require valid numbers and same-recipe links', async()=>{
 const {draftInputSchema}=await import('./recipe');
 const input={id:crypto.randomUUID(),expectedRevision:null,title:'Tarif',description:null,kind:'food',servings:'1',prepMinutes:null,cookMinutes:null,notes:null,ingredients:[],steps:[{id:crypto.randomUUID(),instructions:'Karıştırın'}]};
 expect(draftInputSchema.parse(input).steps[0].details).toBeUndefined();
 const details={formatVersion:1,title:null,durationMinutes:'1.',temperatureValue:'-',temperatureUnit:null,heatLevel:null,equipment:null,technique:null,tips:null};
 const advanced={...input,steps:[{...input.steps[0],details}]};
 expect(draftInputSchema.safeParse(advanced).success).toBe(true);
 expect(recipeInputSchema.safeParse(advanced).success).toBe(false);
 expect(recipeInputSchema.safeParse({...input,steps:[{...input.steps[0],ingredientLineIds:[crypto.randomUUID()]}]}).success).toBe(false);
 expect(recipeInputSchema.safeParse({...advanced,steps:[{...input.steps[0],details:{...details,durationMinutes:'0',temperatureValue:'180',temperatureUnit:'C'}}]}).success).toBe(true);
});
