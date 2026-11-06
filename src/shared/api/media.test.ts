import {expect,test,vi} from 'vitest';
const invoke=vi.fn();
vi.mock('@tauri-apps/api/core',()=>({isTauri:()=>true,invoke:(...args:unknown[])=>invoke(...args),convertFileSrc:(path:string,protocol:string)=>`http://${protocol}.localhost/${encodeURIComponent(path)}`}));
import {recipeClient} from './recipes';
const id='aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
test('media URLs encode only a validated UUID and keep the variant separate, including Windows protocol form',()=>{
 expect(recipeClient.coverUrl(id,'thumb')).toBe(`http://recipe-media.localhost/${id}/thumb`);expect(()=>recipeClient.coverUrl('../library.sqlite3','preview')).toThrow();expect(()=>recipeClient.coverUrl(id,'original' as 'preview')).toThrow();
});
test('picker IPC receives no renderer paths and validates cancellation, metadata and pixel bounds',async()=>{
 invoke.mockResolvedValueOnce(null);expect(await recipeClient.pickCover()).toBeNull();expect(invoke).toHaveBeenLastCalledWith('pick_recipe_cover',undefined);invoke.mockResolvedValueOnce({id,width:6000,height:5000});expect(await recipeClient.pickCover()).toEqual({id,width:6000,height:5000});invoke.mockResolvedValueOnce({id,width:10000,height:10000});await expect(recipeClient.pickCover()).rejects.toMatchObject({code:'INVALID_RESPONSE'});
});

test('step picker reuses bounded media metadata with no renderer source path',async()=>{
 invoke.mockResolvedValueOnce({id,width:48,height:32});expect(await recipeClient.pickStepPhoto()).toEqual({id,width:48,height:32});expect(invoke).toHaveBeenLastCalledWith('pick_recipe_step_photo',undefined);
 invoke.mockResolvedValueOnce(null);expect(await recipeClient.pickStepPhoto()).toBeNull();invoke.mockResolvedValueOnce({id:'../private',width:1,height:1});await expect(recipeClient.pickStepPhoto()).rejects.toMatchObject({code:'INVALID_RESPONSE'});
});
