import { expect,test,vi,beforeEach } from 'vitest';
import { invoke,isTauri } from '@tauri-apps/api/core';
import { recipeClient } from './recipes';
import { snapshotSchema,restoreVersionSchema } from '../contracts/recipeHistory';
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn(),isTauri:vi.fn()}));
beforeEach(()=>{vi.mocked(isTauri).mockReturnValue(true);vi.mocked(invoke).mockReset();});
const id='11111111-1111-4111-8111-111111111111';
test('history IPC uses exact bounded args and validates native response',async()=>{
 vi.mocked(invoke).mockResolvedValue([]);await recipeClient.versions(id,5,20);expect(invoke).toHaveBeenCalledWith('list_recipe_versions',{id,before:5,limit:20});expect(()=>recipeClient.versions(id,null,101)).toThrow();vi.mocked(invoke).mockResolvedValue({formatVersion:99});await expect(recipeClient.version(id,1)).rejects.toMatchObject({code:'INVALID_RESPONSE'});
});
test('restore request is strict and versioned snapshot preserves exact quantity',()=>{
 expect(restoreVersionSchema.safeParse({recipeId:id,versionNumber:0,expectedRevision:2}).success).toBe(false);expect(restoreVersionSchema.safeParse({recipeId:id,versionNumber:1,expectedRevision:2,sql:'DELETE'}).success).toBe(false);
 const data={formatVersion:1,recipeId:id,recipeCreatedAt:'now',title:'Su',kind:'beverage',servings:'1',description:null,prepMinutes:null,cookMinutes:null,notes:null,ingredients:[{id,ingredientId:id,quantity:'1.250',unitCode:'mL',note:null}],steps:[],ingredientNames:{[id]:'Gül suyu'}};
 expect(snapshotSchema.parse(data).ingredients[0].quantity).toBe('1.250');expect(snapshotSchema.safeParse({...data,formatVersion:4}).success).toBe(false);
});
