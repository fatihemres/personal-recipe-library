import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { IngredientLibrary } from './IngredientLibrary';
import type { IngredientLibraryClient } from '../../shared/api/ingredientLibrary';
import type { LibraryItem, LibraryPage, LibraryDetail } from '../../shared/contracts/ingredientLibrary';
import { rendererRecipeClient } from '../../test/recipeClient';
// Renderer-only fixtures. Real data/counts/persistence are verified by Rust and native checks.
const item: LibraryItem = { id: crypto.randomUUID(), origin: 'catalog', name: 'Toz şeker', englishName: 'Granulated sugar', categories: [], recipeId: null, notes: null, preferredUnit: 'g', revision: null, ingredientType: 'food', favorite:false, lastUsedAt:null };
const category = { id: crypto.randomUUID(), parentId: null, key: 'sweeteners', tr: 'Tatlandırıcılar', en: 'Sweeteners' };
const page: LibraryPage = { items: [item], total: 508, catalogCount: 508, personalCount: 0, categories: [{ category, count: 7 }], offset: 0, limit: 30 };
const detail: LibraryDetail = { item, aliases: [{ locale: 'tr', name: 'Şeker' }], canonicalTr: 'Toz şeker', dimensions: ['mass'], sources: [], metadata: [], catalogVersion: 5, personalAlias:null };
function api(): IngredientLibraryClient { return { browse: vi.fn().mockResolvedValue(page), detail: vi.fn().mockResolvedValue(detail), favorite:vi.fn().mockResolvedValue(true), presentation:vi.fn().mockResolvedValue(detail), candidates:vi.fn().mockResolvedValue([]) }; }
test('view, category and search filters compose; paging is delegated to IPC', async () => {
  const library = api(); const user = userEvent.setup(); render(<IngredientLibrary client={rendererRecipeClient()} library={library} />);
  await screen.findByText('508 eşleşme');
  await user.click(screen.getByRole('button', { name: 'Hazır Katalog 508' }));
  await waitFor(() => expect(screen.getByRole('combobox')).toBeEnabled());
  await user.selectOptions(screen.getByRole('combobox'), category.id);
  await user.type(screen.getByRole('searchbox'), 'ŞEK');
  await waitFor(() => expect(library.browse).toHaveBeenLastCalledWith({ search: 'ŞEK', origin: 'catalog', categoryId: category.id, offset: 0, limit: 30, view: null, uncategorized:false }));
  await user.click(screen.getByRole('button', { name: 'Sonraki sayfa' }));
  await waitFor(() => expect(library.browse).toHaveBeenLastCalledWith(expect.objectContaining({ offset: 30 })));
  await user.click(screen.getByRole('button', { name: 'Filtreleri temizle' }));
  expect(screen.getByRole('searchbox')).toHaveValue('');
});
test('uncategorized filter combines with personal origin and Turkish search', async () => {
  const library=api(); const user=userEvent.setup(); render(<IngredientLibrary client={rendererRecipeClient()} library={library}/>);
  await screen.findByText('508 eşleşme');
  await user.click(screen.getByRole('button',{name:'Kişisel Malzemeler 0'}));
  await waitFor(()=>expect(screen.getByRole('combobox')).toBeEnabled());
  await user.selectOptions(screen.getByRole('combobox'), 'uncategorized');
  await user.type(screen.getByRole('searchbox'), 'İçli');
  await waitFor(()=>expect(library.browse).toHaveBeenLastCalledWith(expect.objectContaining({origin:'personal',uncategorized:true,categoryId:null,search:'İçli'})));
});
test('catalog details are readonly and explicitly distinguish unknown metadata', async () => {
  const library = api(); const user = userEvent.setup(); render(<IngredientLibrary client={rendererRecipeClient()} library={library} />);
  await user.click(await screen.findByRole('button', { name: /Toz şeker.*Granulated sugar/ }));
  expect(await screen.findByRole('heading', { name: 'Toz şeker' })).toBeVisible();
  expect(screen.getByText(/Bilgi eksikliği sıfır değer/)).toBeVisible();
  expect(screen.queryByRole('button', { name: 'Düzenle' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Kütüphaneye dön' }));
  expect(screen.getByRole('searchbox')).toBeVisible();
});
test('database failures remain separate from empty search results and allow retry', async () => {
  const library = api(); vi.mocked(library.browse).mockRejectedValueOnce({ code: 'STORAGE_UNAVAILABLE', messageKey: 'errors.storage', recoverable: true });
  const user = userEvent.setup(); render(<IngredientLibrary client={rendererRecipeClient()} library={library} />);
  expect(await screen.findByRole('alert')).toHaveTextContent('Depolama açılamadı');
  expect(screen.queryByText('Eşleşen malzeme yok')).not.toBeInTheDocument();
  vi.mocked(library.browse).mockResolvedValue({ ...page, items: [], total: 0 });
  await user.click(screen.getByRole('button', { name: 'Yeniden dene' }));
  expect(await screen.findByRole('heading', { name: 'Eşleşen malzeme yok' })).toBeVisible();
  expect(screen.getAllByRole('button', { name: 'Kişisel malzeme ekle' })).toHaveLength(2);
});
test('personal details expose revision-safe editing and conflict errors retain entered text', async () => {
  const personal = { ...item, origin: 'personal' as const, revision: 2, recipeId: item.id };
  const library = api(); vi.mocked(library.detail).mockResolvedValue({ ...detail, item: personal });
  const client = rendererRecipeClient(); vi.mocked(client.editIngredient).mockRejectedValue({ code: 'CONFLICT', messageKey: 'errors.conflict', recoverable: true });
  const user = userEvent.setup(); render(<IngredientLibrary client={client} library={library} />);
  await user.click(await screen.findByRole('button', { name: /Toz şeker.*Granulated sugar/ }));
  await user.click(await screen.findByRole('button', { name: 'Düzenle' }));
  const name = screen.getByRole('textbox', { name: 'Malzeme adı' }); await user.clear(name); await user.type(name, 'Aile şekerim');
  await user.click(screen.getByRole('button', { name: 'Malzemeyi kaydet' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('başka bir işlemde değiştirildi');
  expect(name).toHaveValue('Aile şekerim');
  expect(client.editIngredient).toHaveBeenCalledWith(expect.objectContaining({ id: item.id, revision: 2, name: 'Aile şekerim' }));
});
test('favorites remain connected between card, filter and detail, and catalog presentation is submitted separately', async () => {
  const library=api(); const user=userEvent.setup();
  vi.mocked(library.browse).mockImplementation(async query => ({...page,items:[{...item,favorite:query.view==='favorites',recipeId:item.id,revision:2}],total:query.view==='favorites'?1:508}));
  vi.mocked(library.detail).mockResolvedValue({...detail,item:{...item,recipeId:item.id,revision:2}});
  render(<IngredientLibrary client={rendererRecipeClient()} library={library}/>);
  await user.click(await screen.findByRole('button',{name:'Toz şeker: Favorilere ekle'}));
  await waitFor(()=>expect(library.favorite).toHaveBeenCalledWith(item.id,'catalog',true));
  await user.click(screen.getByRole('button',{name:'Favoriler'}));
  await waitFor(()=>expect(library.browse).toHaveBeenLastCalledWith(expect.objectContaining({view:'favorites'})));
  await user.click(await screen.findByRole('button',{name:/Toz şeker.*Granulated sugar/}));
  await user.click(await screen.findByRole('button',{name:'Görünümü kişiselleştir'}));
  await user.clear(screen.getByLabelText('Tercih ettiğiniz ad'));
  await user.type(screen.getByLabelText('Tercih ettiğiniz ad'),'Bizim şeker');
  await user.click(screen.getByRole('button',{name:'Tercihleri kaydet'}));
  await waitFor(()=>expect(library.presentation).toHaveBeenCalledWith(expect.objectContaining({id:item.id,name:'Bizim şeker',revision:2})));
});
test('creation warns for canonical alias collision before mutation and offers existing record', async () => {
  const library=api();vi.mocked(library.candidates).mockResolvedValue([{id:item.id,origin:'catalog',name:'Toz şeker',reason:'possible_synonym'}]);
  const recipe=rendererRecipeClient();const user=userEvent.setup();render(<IngredientLibrary client={recipe} library={library}/>);
  await user.click(screen.getByRole('button',{name:'Kişisel malzeme ekle'}));
  await user.type(screen.getByLabelText('Yeni malzemenin adı'),'Şeker');
  await user.click(screen.getAllByRole('button',{name:'Kişisel malzeme ekle'})[1]);
  expect(await screen.findByText(/Olası eşleşmeler bulundu/)).toBeVisible();
  expect(recipe.createIngredient).not.toHaveBeenCalled();
  expect(screen.getByText(/Diğer adla olası eşleşme/)).toBeVisible();
  await user.click(screen.getByRole('button',{name:'Kaydı görüntüle'}));
  expect(await screen.findByRole('heading',{name:'Toz şeker'})).toBeVisible();
});
test('exact collision cannot be acknowledged as a new personal ingredient', async () => {
  const library=api();vi.mocked(library.candidates).mockResolvedValue([{id:item.id,origin:'catalog',name:'Toz şeker',reason:'exact_name'}]);
  const recipe=rendererRecipeClient();const user=userEvent.setup();render(<IngredientLibrary client={recipe} library={library}/>);
  await user.click(screen.getByRole('button',{name:'Kişisel malzeme ekle'}));
  await user.type(screen.getByLabelText('Yeni malzemenin adı'),'Toz şeker');
  await user.click(screen.getAllByRole('button',{name:'Kişisel malzeme ekle'})[1]);
  expect((await screen.findAllByText(/Bu ad mevcut bir ad veya diğer adla eşleşiyor/)).length).toBeGreaterThan(0);
  expect(recipe.createIngredient).not.toHaveBeenCalled();
  expect(screen.queryByRole('button',{name:'Farklı malzeme olarak devam et'})).not.toBeInTheDocument();
});
test('unfinished personal creation can continue editing or explicitly discard before navigation', async () => {
  const library=api();const user=userEvent.setup();let guard:(()=>Promise<boolean>)|null=null;
  render(<IngredientLibrary client={rendererRecipeClient()} library={library} registerLeaveGuard={value=>{guard=value;}}/>);
  await user.click(screen.getByRole('button',{name:'Kişisel malzeme ekle'}));
  await user.type(screen.getByLabelText('Yeni malzemenin adı'),'Yeni baharat');
  await waitFor(()=>expect(guard).not.toBeNull());
  const first=guard!();
  await user.click(await screen.findByRole('button',{name:'Düzenlemeye devam et'}));
  expect(await first).toBe(false);
  expect(screen.getByLabelText('Yeni malzemenin adı')).toHaveValue('Yeni baharat');
  const second=guard!();
  await user.click(await screen.findByRole('button',{name:'Vazgeç ve devam et'}));
  expect(await second).toBe(true);
});
