import { useState } from 'react';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { RecipeSteps } from './RecipeSteps';
import { RecipeEditor } from './RecipeEditor';
import { StepPhoto, StepPhotoEditor } from './StepPhoto';
import { RecipeHistory } from './RecipeHistory';
import { rendererRecipeClient } from '../../test/recipeClient';
import { draftInputSchema, recipeInputSchema, type Recipe, type RecipeInput } from '../../shared/contracts/recipe';
import { snapshotSchema } from '../../shared/contracts/recipeHistory';

const exit = vi.hoisted(() => ({ guard: null as (() => Promise<void>) | null }));
vi.mock('../../app/nativeExit', () => ({ registerDraftExitGuard: (guard: () => Promise<void>) => {
  exit.guard = guard; return () => { if (exit.guard === guard) exit.guard = null; };
} }));
const a='aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', b='bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb';
const initial:RecipeInput['steps']=[{id:crypto.randomUUID(),instructions:'İlk aşama',photoAssetId:a},{id:crypto.randomUUID(),instructions:'Sonraki aşama'}];
function input():RecipeInput{return {id:crypto.randomUUID(),expectedRevision:null,title:'M5C2 — İ/ı',description:null,kind:'food',servings:'1.250',prepMinutes:0,cookMinutes:null,notes:null,ingredients:[],steps:structuredClone(initial)};}
function recipe():Recipe {const {expectedRevision,...i}=input();void expectedRevision;return {...i,revision:2,createdAt:'2026-11-06T09:00:00Z',updatedAt:'2026-11-06T09:00:00Z',archivedAt:null,deletedAt:null,ingredientNames:{}};}

test('photographed step deletion requires confirmation; Cancel keeps exact draft and confirm remains unsaved',async()=>{
 const api=rendererRecipeClient(),changed=vi.fn(),user=userEvent.setup();
 function Harness(){const [steps,setSteps]=useState(structuredClone(initial));return <RecipeSteps steps={steps} client={api} onChange={next=>{changed(next);setSteps(next);}}/>;}
 const view=render(<Harness/>);const row=()=>view.container.querySelector(`[data-step-id="${initial[0].id}"]`) as HTMLElement;
 await user.click(within(row()).getByRole('button',{name:'Kaldır'}));
 const dialog=screen.getByRole('dialog');expect(within(dialog).getByRole('button',{name:'Vazgeç'})).toHaveFocus();await user.click(within(dialog).getByRole('button',{name:'Vazgeç'}));expect(screen.queryByRole('dialog')).not.toBeInTheDocument();expect(changed).not.toHaveBeenCalled();expect(row()).toBeInTheDocument();expect(initial[0].photoAssetId).toBe(a);
 await user.click(within(row()).getByRole('button',{name:'Kaldır'}));await user.click(screen.getByRole('button',{name:'Adımı taslaktan kaldır'}));expect(row()).not.toBeInTheDocument();expect(changed).toHaveBeenLastCalledWith([initial[1]]);expect(screen.getByRole('button',{name:'Adım ekle'})).toHaveFocus();expect(api.commit).not.toHaveBeenCalled();
});

test('draft-only imported photo survives step reorder and expanding controls; no persisted content write until Save',async()=>{
 const api=rendererRecipeClient(),user=userEvent.setup();vi.mocked(api.pickStepPhoto).mockResolvedValue({id:b,width:48,height:32});
 function Harness(){const [steps,setSteps]=useState<RecipeInput['steps']>([{...initial[0],photoAssetId:null},initial[1]]);return <RecipeSteps steps={steps} client={api} onChange={setSteps}/>;}
 const view=render(<Harness/>);const first=()=>view.container.querySelector(`[data-step-id="${initial[0].id}"]`) as HTMLElement;
 expect(api.pickStepPhoto).not.toHaveBeenCalled();expect(screen.queryByRole('img')).not.toBeInTheDocument();
 await user.click(within(first()).getByText('Gelişmiş adım bilgileri', {exact:false}));await user.click(await within(first()).findByRole('button',{name:'Fotoğraf ekle'}));expect(await within(first()).findByRole('img')).toHaveAttribute('src',expect.stringContaining(b));
 await user.click(within(first()).getByRole('button',{name:/Aşağı/}));const rows=view.container.querySelectorAll('[data-step-id]');expect(rows[1]).toHaveAttribute('data-step-id',initial[0].id);expect(within(rows[1] as HTMLElement).getByRole('img')).toHaveAttribute('src',expect.stringContaining(b));
 await user.click(within(first()).getByRole('button',{name:'Kaldır'}));await user.click(screen.getByRole('button',{name:'Vazgeç'}));expect(within(first()).getByRole('img')).toHaveAttribute('src',expect.stringContaining(b));expect(api.commit).not.toHaveBeenCalled();
});

test('thumbnail is lazy, preview opens only deliberately and dialog cancellation restores keyboard focus; missing files stay recoverable',async()=>{
 const api=rendererRecipeClient(),user=userEvent.setup();render(<StepPhoto assetId={a} position={2} client={api}/>);
 const image=screen.getByRole('img',{name:'2. adım fotoğrafı'});expect(image).toHaveAttribute('loading','lazy');expect(image).toHaveAttribute('src',expect.stringContaining('/thumb'));
 const preview=screen.getByRole('button',{name:'2. adım fotoğrafını önizle'});preview.focus();await user.keyboard('{Enter}');const dialog=screen.getByRole('dialog');expect(within(dialog).getByRole('img')).toHaveAttribute('src',expect.stringContaining('/preview'));expect(within(dialog).getByRole('button',{name:'Önizlemeyi kapat'})).toHaveFocus();
 fireEvent.error(within(dialog).getByRole('img'));expect(within(dialog).getByRole('status')).toHaveTextContent('Kayıtlı bağlantı korunuyor');await user.click(within(dialog).getByRole('button',{name:'Görseli yeniden yükle'}));expect(within(dialog).getByRole('img')).toBeVisible();fireEvent(dialog,new Event('cancel',{cancelable:true}));expect(screen.queryByRole('dialog')).not.toBeInTheDocument();expect(preview).toHaveFocus();
});

test('pending step import blocks editor Save and native exit; completion enters durable draft and explicit Save',async()=>{
 const api=rendererRecipeClient(),user=userEvent.setup();let finish!:(v:{id:string;width:number;height:number}|null)=>void;
 vi.mocked(api.pickStepPhoto).mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));const r=recipe();render(<RecipeEditor recipe={r} client={api} onSaved={vi.fn()} onCancel={vi.fn()}/>);
 await user.click(screen.getAllByText('Gelişmiş adım bilgileri',{exact:false})[0]);await user.click(await screen.findByRole('button',{name:'Fotoğrafı değiştir'}));expect(screen.getByRole('button',{name:'Tarifi kaydet'})).toBeDisabled();expect(exit.guard).not.toBeNull();await act(async()=>{await expect(exit.guard!()).rejects.toMatchObject({code:'MEDIA_BUSY'});});
 await act(async()=>finish({id:b,width:48,height:32}));await waitFor(()=>expect(screen.getByRole('button',{name:'Tarifi kaydet'})).toBeEnabled());await user.click(screen.getByRole('button',{name:'Tarifi kaydet'}));await waitFor(()=>expect(api.commit).toHaveBeenCalled());expect(vi.mocked(api.commit).mock.calls[0][0].steps[0].photoAssetId).toBe(b);
});

test('picker cancellation and errors leave association unchanged; a late completion cannot change an unmounted editor',async()=>{
 const api=rendererRecipeClient(),change=vi.fn(),busy=vi.fn(),user=userEvent.setup();vi.mocked(api.pickStepPhoto).mockResolvedValueOnce(null).mockRejectedValueOnce({code:'MEDIA_UNSUPPORTED',messageKey:'errors.mediaUnsupported',recoverable:true});
 const view=render(<StepPhotoEditor assetId={a} position={1} client={api} onChange={change} onBusy={busy}/>);await user.click(screen.getByRole('button',{name:'Fotoğrafı değiştir'}));expect(change).not.toHaveBeenCalled();await user.click(screen.getByRole('button',{name:'Fotoğrafı değiştir'}));expect(await screen.findByRole('alert')).toHaveTextContent('HEIC/HEIF');expect(change).not.toHaveBeenCalled();
 let finish!:(v:{id:string;width:number;height:number})=>void;vi.mocked(api.pickStepPhoto).mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));await user.click(screen.getByRole('button',{name:'Fotoğrafı değiştir'}));view.unmount();await act(async()=>finish({id:b,width:48,height:32}));expect(change).not.toHaveBeenCalled();expect(busy).toHaveBeenLastCalledWith(false);
});

test('strict contracts accept legacy drafts and null photo values, reject paths/duplicates/future snapshots and keep 500-step bound',()=>{
 const i=input();expect(draftInputSchema.safeParse({...i,steps:[{id:initial[1].id,instructions:''}]}).success).toBe(true);expect(recipeInputSchema.safeParse({...i,steps:[{...initial[0],photoAssetId:'../../private.png'}]}).success).toBe(false);expect(recipeInputSchema.safeParse({...i,steps:[initial[0],initial[0]]}).success).toBe(false);
 const r=recipe();const {id:recipeId,revision,createdAt,updatedAt,archivedAt,deletedAt,...content}=r;void revision;void updatedAt;void archivedAt;void deletedAt;
 const snap={...content,recipeId,recipeCreatedAt:createdAt};for(const formatVersion of [1,2,3])expect(snapshotSchema.safeParse({...snap,formatVersion}).success).toBe(false);expect(snapshotSchema.safeParse({...snap,formatVersion:4}).success).toBe(true);expect(snapshotSchema.safeParse({...snap,formatVersion:5}).success).toBe(false);
 const steps=Array.from({length:500},()=>({...initial[0],id:crypto.randomUUID()}));expect(recipeInputSchema.safeParse({...i,steps}).success).toBe(true);expect(recipeInputSchema.safeParse({...i,steps:[...steps,{...initial[0],id:crypto.randomUUID()}]}).success).toBe(false);
});

test('history preview and comparison render the matching immutable photos; old restore warns about clearing current photos',async()=>{
 const api=rendererRecipeClient(),user=userEvent.setup(),r=recipe();const {id:recipeId,revision,createdAt,updatedAt,archivedAt,deletedAt,...content}=r;void revision;void updatedAt;void archivedAt;void deletedAt;
 const summary={recipeId,versionNumber:1,formatVersion:4,sourceRevision:1,provenance:'created' as const,restoredFromVersion:null,capturedAt:createdAt,contentFingerprint:'a'.repeat(64),snapshotChecksum:'b'.repeat(64)};
 vi.mocked(api.versions).mockResolvedValue([summary]);vi.mocked(api.version).mockResolvedValue({summary,snapshot:{...content,formatVersion:4,recipeId,recipeCreatedAt:createdAt}});vi.mocked(api.compareVersions).mockResolvedValue({currentRevision:2,changes:[{path:['steps','0','photoAssetId'],before:a,after:b}]});
 render(<RecipeHistory recipe={r} client={api} onRestored={vi.fn()} beforeRestore={async()=>true}/>);await user.click(screen.getByRole('button',{name:'Geçmişi aç'}));await user.click(await screen.findByRole('button',{name:'V1 · Sürümü görüntüle'}));expect(await screen.findByRole('img',{name:'1. adım fotoğrafı'})).toHaveAttribute('src',expect.stringContaining(a));await user.click(screen.getByRole('button',{name:'Karşılaştır'}));const comparison=await screen.findByRole('region',{name:'Karşılaştır'});expect(within(comparison).getAllByRole('img')).toHaveLength(2);
 const legacy={...content,steps:content.steps.map(s=>({...s,photoAssetId:null})),formatVersion:3 as const,recipeId,recipeCreatedAt:createdAt};vi.mocked(api.version).mockResolvedValue({summary:{...summary,formatVersion:3},snapshot:legacy});await user.click(screen.getByRole('button',{name:'V1 · Sürümü görüntüle'}));await user.click(screen.getByRole('button',{name:'Bu sürümü geri yükle'}));expect(await screen.findByRole('dialog')).toHaveTextContent('Bu eski sürüm adım fotoğrafları içermez');fireEvent(screen.getByRole('dialog'),new Event('cancel',{cancelable:true}));expect(screen.queryByRole('dialog')).not.toBeInTheDocument();expect(api.restoreVersion).not.toHaveBeenCalled();
});

test('collapsing advanced details during import preserves the selected photo in the same draft step',async()=>{
 const api=rendererRecipeClient(),user=userEvent.setup();let finish!:(v:{id:string;width:number;height:number})=>void;
 vi.mocked(api.pickStepPhoto).mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));const onChange=vi.fn();
 function Harness(){const [steps,setSteps]=useState<RecipeInput['steps']>([{...initial[0],photoAssetId:null}]);return <RecipeSteps steps={steps} client={api} onChange={next=>{onChange(next);setSteps(next);}}/>;}
 render(<Harness/>);const summary=screen.getByText('Gelişmiş adım bilgileri',{exact:false});await user.click(summary);await user.click(await screen.findByRole('button',{name:'Fotoğraf ekle'}));await user.click(summary);await act(async()=>finish({id:b,width:48,height:32}));await user.click(summary);expect(await screen.findByRole('img')).toHaveAttribute('src',expect.stringContaining(b));expect(onChange).toHaveBeenLastCalledWith([expect.objectContaining({id:initial[0].id,photoAssetId:b})]);
});
