import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { CatalogPresentationEditor } from './CatalogPresentationEditor';
import type { IngredientLibraryClient } from '../../shared/api/ingredientLibrary';
import type { LibraryDetail } from '../../shared/contracts/ingredientLibrary';
import { rendererRecipeClient } from '../../test/recipeClient';

test('catalog presentation leave guard retains edits after failure and only navigates after a successful save', async () => {
  const id=crypto.randomUUID();
  const detail: LibraryDetail={
    item:{id,origin:'catalog',recipeId:id,name:'Toz şeker',englishName:'Granulated sugar',notes:null,preferredUnit:'g',revision:2,ingredientType:'food',categories:[],favorite:false,lastUsedAt:null},
    canonicalTr:'Toz şeker',personalAlias:null,aliases:[],dimensions:['mass'],sources:[],metadata:[],catalogVersion:5,
  };
  const presentation=vi.fn().mockRejectedValueOnce({code:'CONFLICT',messageKey:'errors.conflict',recoverable:true}).mockResolvedValue(detail);
  const library={presentation} as unknown as IngredientLibraryClient;
  const client=rendererRecipeClient();const user=userEvent.setup();let guard:(()=>Promise<boolean>)|null=null;
  render(<CatalogPresentationEditor detail={detail} client={client} library={library} onDone={vi.fn()} registerLeaveGuard={value=>{guard=value;}}/>);
  const name=screen.getByRole('textbox',{name:'Tercih ettiğiniz ad'});
  await user.clear(name);await user.type(name,'Bizim şeker');
  let decision!:Promise<boolean>;act(()=>{decision=guard!();});
  expect(guard!()).toBe(decision);
  await user.click(await screen.findByRole('button',{name:'Kaydet ve devam et'}));
  expect(await within(screen.getByRole('dialog')).findByRole('alert')).toHaveTextContent('başka bir işlemde değiştirildi');
  expect(name).toHaveValue('Bizim şeker');
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  await user.click(screen.getByRole('button',{name:'Kaydet ve devam et'}));
  expect(await decision).toBe(true);
  expect(presentation).toHaveBeenLastCalledWith(expect.objectContaining({id,revision:2,name:'Bizim şeker'}));
});

test('unmount cancels a pending catalog presentation leave decision', async () => {
  const id=crypto.randomUUID();
  const detail: LibraryDetail={item:{id,origin:'catalog',recipeId:id,name:'Toz şeker',englishName:'Granulated sugar',notes:null,preferredUnit:'g',revision:2,ingredientType:'food',categories:[],favorite:false,lastUsedAt:null},canonicalTr:'Toz şeker',personalAlias:null,aliases:[],dimensions:['mass'],sources:[],metadata:[],catalogVersion:8};
  let guard:(()=>Promise<boolean>)|null=null;
  const user=userEvent.setup();
  const view=render(<CatalogPresentationEditor detail={detail} client={rendererRecipeClient()} library={{presentation:vi.fn()} as unknown as IngredientLibraryClient} onDone={vi.fn()} registerLeaveGuard={value=>{guard=value;}}/>);
  await user.type(screen.getByRole('textbox',{name:'Tercih ettiğiniz ad'}),' özel');
  let decision!:Promise<boolean>;act(()=>{decision=guard!();});
  expect(await screen.findByRole('dialog')).toBeVisible();
  view.unmount();
  expect(await decision).toBe(false);
  expect(guard).toBeNull();
});
