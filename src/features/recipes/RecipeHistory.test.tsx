import { render,screen,waitFor,within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect,test,vi } from 'vitest';
import { RecipeHistory } from './RecipeHistory';
import { rendererRecipeClient } from '../../test/recipeClient';
import type { Recipe } from '../../shared/contracts/recipe';
const recipe:Recipe={id:crypto.randomUUID(),revision:2,title:'İçli köfte',kind:'food',servings:'1',description:null,prepMinutes:null,cookMinutes:null,notes:null,ingredients:[],steps:[],ingredientNames:{},createdAt:'2026-11-02T12:00:00Z',updatedAt:'2026-11-02T12:00:00Z',deletedAt:null,archivedAt:null};
const summary={recipeId:recipe.id,versionNumber:1,formatVersion:1,sourceRevision:1,provenance:'upgrade_baseline' as const,restoredFromVersion:null,capturedAt:recipe.createdAt,contentFingerprint:'a'.repeat(64),snapshotChecksum:'b'.repeat(64)};
const snapshot={formatVersion:1 as const,recipeId:recipe.id,recipeCreatedAt:recipe.createdAt,title:recipe.title,description:null,kind:'food' as const,servings:'1',prepMinutes:null,cookMinutes:null,notes:null,ingredients:[],steps:[],ingredientNames:{}};
function fixture(inactive=false){const client=rendererRecipeClient();vi.mocked(client.versions).mockResolvedValue([summary]);vi.mocked(client.version).mockResolvedValue({summary,snapshot});vi.mocked(client.compareVersions).mockResolvedValue({currentRevision:2,changes:[{path:['ingredients','0','quantity'],before:'1.250',after:'1.25'}]});const onRestored=vi.fn();const guard=vi.fn().mockResolvedValue(true);render(<RecipeHistory recipe={{...recipe,archivedAt:inactive?recipe.createdAt:null}} client={client} onRestored={onRestored} beforeRestore={guard}/>);return{client,onRestored,guard};}
async function open(){const user=userEvent.setup();await user.click(screen.getByRole('button',{name:'Geçmişi aç'}));await user.click(await screen.findByRole('button',{name:'V1 · Sürümü görüntüle'}));await screen.findByRole('region',{name:'Seçili sürüm'});return user;}
test('history is lazy, baseline honest, exact comparison visible and restore explicitly confirmed',async()=>{
 const {client,onRestored,guard}=fixture();expect(client.versions).not.toHaveBeenCalled();const user=await open();expect(screen.getByText(/tarifin ilk oluşturulduğu hâli/)).toBeVisible();
 await user.click(screen.getByRole('button',{name:'Karşılaştır'}));expect(await screen.findByText('"1.250"')).toBeVisible();expect(screen.getByText('"1.25"')).toBeVisible();
 vi.mocked(client.restoreVersion).mockResolvedValue({recipe:{...recipe,revision:3},changed:true,versionNumber:3});
 await user.click(screen.getByRole('button',{name:'Bu sürümü geri yükle'}));expect(guard).toHaveBeenCalledOnce();const dialog=screen.getByRole('dialog');expect(within(dialog).getByRole('button',{name:'Vazgeç'})).toHaveFocus();expect(client.restoreVersion).not.toHaveBeenCalled();await user.click(within(dialog).getByRole('button',{name:'Bu sürümü geri yükle'}));
 await waitFor(()=>expect(onRestored).toHaveBeenCalledWith(expect.objectContaining({id:recipe.id,revision:3})));expect(client.restoreVersion).toHaveBeenCalledWith({recipeId:recipe.id,versionNumber:1,expectedRevision:2});expect(await screen.findByText('İçerik yeni bir sürüm olarak geri yüklendi.')).toBeVisible();
});
test('draft/conflict error keeps confirmation recoverable and cancel retains recipe',async()=>{
 const {client,onRestored}=fixture();vi.mocked(client.restoreVersion).mockRejectedValue({code:'RECIPE_RESTORE_DRAFT',messageKey:'errors.recipeRestoreDraft',recoverable:true});const user=await open();await user.click(screen.getByRole('button',{name:'Bu sürümü geri yükle'}));await user.click(within(screen.getByRole('dialog')).getByRole('button',{name:'Bu sürümü geri yükle'}));expect(await within(screen.getByRole('dialog')).findByRole('alert')).toHaveTextContent('açık düzenleme taslağı');expect(onRestored).not.toHaveBeenCalled();await user.click(within(screen.getByRole('dialog')).getByRole('button',{name:'Vazgeç'}));expect(screen.queryByRole('dialog')).toBeNull();
});
test('inactive history is readable without restore and unsupported snapshot fails explicitly',async()=>{
 const {client}=fixture(true);await open();expect(screen.queryByRole('button',{name:'Bu sürümü geri yükle'})).toBeNull();expect(screen.getByText(/Geçmiş salt okunur/)).toBeVisible();vi.mocked(client.version).mockRejectedValue({code:'RECIPE_VERSION_UNSUPPORTED',messageKey:'errors.recipeVersionUnsupported',recoverable:true});await userEvent.click(screen.getByRole('button',{name:'V1 · Sürümü görüntüle'}));expect(await screen.findByRole('alert')).toHaveTextContent('daha yeni bir biçim');expect(screen.queryByRole('region',{name:'Seçili sürüm'})).toBeNull();
});
test('no-change outcome is explicit and unsaved private note guard can cancel',async()=>{
 const {client,guard}=fixture();const user=await open();guard.mockResolvedValueOnce(false);await user.click(screen.getByRole('button',{name:'Bu sürümü geri yükle'}));expect(screen.queryByRole('dialog')).toBeNull();vi.mocked(client.restoreVersion).mockResolvedValue({recipe,changed:false,versionNumber:2});await user.click(screen.getByRole('button',{name:'Bu sürümü geri yükle'}));await user.click(within(screen.getByRole('dialog')).getByRole('button',{name:'Bu sürümü geri yükle'}));expect(await screen.findByText('Seçilen içerik zaten güncel. Yeni sürüm oluşturulmadı.')).toBeVisible();
});
test('metadata pagination is keyset bounded; empty and loading failures are honest',async()=>{
 const {client}=fixture();vi.mocked(client.versions).mockResolvedValueOnce(Array.from({length:20},(_,i)=>({...summary,versionNumber:20-i}))).mockResolvedValueOnce([]);const user=userEvent.setup();await user.click(screen.getByRole('button',{name:'Geçmişi aç'}));await user.click(await screen.findByRole('button',{name:'Daha eski sürümler'}));expect(client.versions).toHaveBeenLastCalledWith(recipe.id,1,20);await waitFor(()=>expect(screen.queryByRole('button',{name:'Daha eski sürümler'})).toBeNull());
});
