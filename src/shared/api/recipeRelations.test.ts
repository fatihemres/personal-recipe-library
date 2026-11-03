import {expect,test,vi} from 'vitest';
import {invoke,isTauri} from '@tauri-apps/api/core';
import {recipeClient} from './recipes';
import {parentWriteSchema,relatedWriteSchema,reviewedPurgeSchema} from '../contracts/recipeRelations';
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn(),isTauri:vi.fn()}));
const id='11111111-1111-4111-8111-111111111111';
test('relations use bounded typed IPC and malformed responses fail closed',async()=>{vi.mocked(isTauri).mockReturnValue(true);vi.mocked(invoke).mockResolvedValue({graphRevision:1,items:[],total:0,hasMore:false});await recipeClient.links(id,0,20);expect(invoke).toHaveBeenCalledWith('recipe_links',{id,offset:0,limit:20});expect(()=>recipeClient.links(id,0,101)).toThrow();vi.mocked(invoke).mockResolvedValue({token:'bad'});await expect(recipeClient.previewPurge(id,2,0,20)).rejects.toMatchObject({code:'INVALID_RESPONSE'});});
test('write schemas reject unknown SQL, invalid IDs and missing detach acknowledgement',()=>{expect(parentWriteSchema.safeParse({childId:id,parentId:null,expectedGraphRevision:1}).success).toBe(true);expect(relatedWriteSchema.safeParse({recipeId:id,targetId:'SQL',expectedGraphRevision:1,linked:true}).success).toBe(false);expect(reviewedPurgeSchema.safeParse({recipeId:id,expectedRevision:2,token:'a'.repeat(64)}).success).toBe(false);expect(reviewedPurgeSchema.safeParse({recipeId:id,expectedRevision:2,token:'a'.repeat(64),acknowledgeDetach:true,sql:'DELETE'}).success).toBe(false);});
