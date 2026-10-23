import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import { rendererRecipeClient } from '../../test/recipeClient';
import { PrivateNotePanel } from './PrivateNotePanel';

it('saves a private note and prevents leaving when a save fails',async()=>{
  const client=rendererRecipeClient();const user=userEvent.setup();
  let guard:(()=>Promise<boolean>)|null=null;
  render(<PrivateNotePanel recipeId={crypto.randomUUID()} client={client} registerLeaveGuard={value=>{guard=value;}}/>);
  const editor=await screen.findByRole('textbox',{name:'Özel Markdown notu'});
  await user.type(editor,'# Gizli');
  vi.mocked(client.savePrivateNote).mockRejectedValueOnce({code:'CONFLICT',messageKey:'errors.conflict',recoverable:true});
  let pending:Promise<boolean>|undefined;
  act(()=>{pending=guard!();});
  await user.click(within(await screen.findByRole('dialog')).getByRole('button',{name:'Notu kaydet'}));
  expect((await screen.findAllByRole('alert')).length).toBeGreaterThan(0);
  await user.click(screen.getByRole('button',{name:'Düzenlemeye devam et'}));
  expect(await pending).toBe(false);
  expect(editor).toHaveValue('# Gizli');
  await user.click(screen.getByRole('button',{name:'Notu kaydet'}));
  expect(client.savePrivateNote).toHaveBeenCalledWith(expect.objectContaining({markdown:'# Gizli'}));
});

it('shows trashed-recipe notes without editable controls',async()=>{
  const client=rendererRecipeClient();
  vi.mocked(client.getPrivateNote).mockResolvedValue({recipeId:crypto.randomUUID(),formatVersion:1,markdown:'# Korunan not',revision:1,updatedAt:null});
  render(<PrivateNotePanel recipeId={crypto.randomUUID()} client={client} readOnly/>);
  expect(await screen.findByRole('heading',{name:'Korunan not'})).toBeVisible();
  expect(screen.getByText(/önce tarifi geri yükleyin/)).toBeVisible();
  expect(screen.queryByRole('textbox')).toBeNull();
  expect(screen.queryByRole('button',{name:'Notu kaydet'})).toBeNull();
});
