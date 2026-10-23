import { expect, test } from 'vitest';
import { collectionWriteSchema, organizationWriteSchema, recipeOrganizationSchema } from './recipeOrganization';
import { recipeSearchQuerySchema } from './recipeSearch';

const id=crypto.randomUUID();
test('organization contract preserves NULL rating and rejects invalid stars or extra fields',()=>{
  expect(recipeOrganizationSchema.parse({recipeId:id,revision:1,favorite:false,toTry:true,rating:null,collectionIds:[]}).rating).toBeNull();
  expect(organizationWriteSchema.safeParse({recipeId:id,expectedRevision:1,favorite:true,toTry:false,rating:0}).success).toBe(false);
  expect(organizationWriteSchema.safeParse({recipeId:id,expectedRevision:1,favorite:true,toTry:false,rating:6}).success).toBe(false);
  expect(organizationWriteSchema.safeParse({recipeId:id,expectedRevision:1,favorite:true,toTry:false,rating:5,sql:'DROP TABLE recipes'}).success).toBe(false);
});
test('collection and library filters validate IDs and title bounds',()=>{
  expect(collectionWriteSchema.safeParse({id,expectedRevision:null,title:'İçli Tarifler'}).success).toBe(true);
  expect(collectionWriteSchema.safeParse({id,expectedRevision:null,title:'  '}).success).toBe(false);
  const base={text:'',scope:'active',kind:null,ingredientId:null,createdFrom:null,createdTo:null,modifiedFrom:null,modifiedTo:null,sort:'modified',offset:0,limit:24};
  expect(recipeSearchQuerySchema.parse(base)).toMatchObject({favoriteOnly:false,toTryOnly:false,collectionId:null});
  expect(recipeSearchQuerySchema.safeParse({...base,collectionId:"' OR 1=1 --"}).success).toBe(false);
});
