import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { LibraryPage } from './LibraryPage';
import { rendererRecipeClient } from '../../test/recipeClient';
import type { Recipe } from '../../shared/contracts/recipe';
// Renderer tests isolate interaction; real SQLite and native tests independently prove persistence.
const client = rendererRecipeClient;
test('named filter is saved without pagination and applying it resets to first page',async()=>{
  const api=client();const user=userEvent.setup();
  const saved={id:crypto.randomUUID(),title:'İçecekler',revision:1,formatVersion:1,available:true,
    query:{text:'',scope:'active' as const,kind:'beverage' as const,ingredientId:null,favoriteOnly:false,toTryOnly:false,collectionId:null,createdFrom:null,createdTo:null,modifiedFrom:null,modifiedTo:null,sort:'modified' as const,offset:0 as const,limit:24 as const}};
  vi.mocked(api.savedFilters).mockResolvedValue([saved]);
  vi.mocked(api.resolveFilter).mockResolvedValue(saved.query);
  render(<LibraryPage client={api}/>);
  await screen.findByRole('button',{name:'İçecekler · Uygula'});
  await user.type(screen.getByLabelText('Filtre adı'),'İçli tarifler');
  await user.click(screen.getByRole('button',{name:'Filtreyi kaydet'}));
  expect(api.saveFilter).toHaveBeenCalledWith(expect.objectContaining({title:'İçli tarifler',query:expect.objectContaining({offset:0,limit:24})}));
  await user.click(screen.getByRole('button',{name:'İçecekler · Uygula'}));
  expect(api.resolveFilter).toHaveBeenCalledWith(saved.id);
  await waitFor(()=>expect(screen.getByLabelText('Tarif türü')).toHaveValue('beverage'));
});
test('dashboard metric navigation leaves an old recipe detail and shows filtered results',async()=>{
  const api=client();const user=userEvent.setup();
  const recipe:Recipe={id:crypto.randomUUID(),revision:1,title:'Ayran',kind:'beverage',servings:'1',
    description:null,prepMinutes:null,cookMinutes:null,notes:null,createdAt:'now',updatedAt:'now',
    deletedAt:null,archivedAt:null,ingredients:[],steps:[],ingredientNames:{}};
  vi.mocked(api.list).mockResolvedValue([recipe]);vi.mocked(api.get).mockResolvedValue(recipe);
  const view=render(<LibraryPage client={api}/>);
  await user.click(await screen.findByRole('button',{name:/Ayran/}));
  expect(await screen.findByRole('button',{name:'Kütüphaneye dön'})).toBeVisible();
  view.rerender(<LibraryPage client={api} navigationRequest={{sequence:1,kind:'beverage'}}/>);
  await waitFor(()=>expect(screen.queryByRole('button',{name:'Kütüphaneye dön'})).toBeNull());
  expect(screen.getByLabelText('Tarif türü')).toHaveValue('beverage');
});
test('create recipe with personal ingredient, decimal comma and reordered steps', async () => {
  const api = client();
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  expect(
    await screen.findByRole('heading', { name: 'Henüz tarif yok' }),
  ).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Yeni tarif' }));
  await user.type(screen.getByLabelText('Tarif adı'), 'İçli köfte');
  await user.clear(screen.getByLabelText('Porsiyon sayısı'));
  await user.type(screen.getByLabelText('Porsiyon sayısı'), '2,50');
  await user.type(screen.getByLabelText('Malzeme ara'), 'İçme suyu');
  await user.click(
    screen.getByRole('button', {
      name: 'Kişisel malzeme oluştur / mevcut olanı seç',
    }),
  );
  await user.type(await screen.findByLabelText('Miktar'), '0,123456');
  await user.selectOptions(screen.getByLabelText('Birim'), 'cc');
  await user.click(screen.getByRole('button', { name: 'Adım ekle' }));
  await user.type(screen.getByLabelText('1. Talimat'), 'Hazırla');
  await user.click(screen.getByRole('button', { name: 'Adım ekle' }));
  await user.type(screen.getByLabelText('2. Talimat'), 'Pişir');
  await user.click(screen.getAllByRole('button', { name: /Yukarı taşı$/ })[1]);
  await user.click(screen.getByRole('button', { name: 'Tarifi kaydet' }));
  await screen.findByRole('heading', { name: 'İçli köfte' });
  expect(api.save).toHaveBeenCalledWith(
    expect.objectContaining({
      title: 'İçli köfte',
      servings: '2.50',
      ingredients: [
        expect.objectContaining({ quantity: '0.123456', unitCode: 'cc' }),
      ],
      steps: [
        expect.objectContaining({ instructions: 'Pişir' }),
        expect.objectContaining({ instructions: 'Hazırla' }),
      ],
    }),
  );
  expect(screen.getByText(/0.123456 cc/)).toBeVisible();
  expect(screen.getByText('İçme suyu')).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Düzenle' }));
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('İçli köfte');
});
test('recipe organization controls are accessible and distinct from ingredient favorites', async () => {
  const api = client();
  const recipe: Recipe = { id:crypto.randomUUID(), revision:1,title:'Börek',kind:'food',servings:'2',
    description:null,prepMinutes:null,cookMinutes:null,notes:null,createdAt:'now',updatedAt:'now',
    deletedAt:null,archivedAt:null,ingredients:[],steps:[],ingredientNames:{} };
  vi.mocked(api.list).mockResolvedValue([recipe]);
  vi.mocked(api.get).mockResolvedValue(recipe);
  vi.mocked(api.getOrganization).mockResolvedValue({recipeId:recipe.id,revision:1,favorite:false,toTry:false,rating:null,collectionIds:[]});
  const user=userEvent.setup();
  render(<LibraryPage client={api}/>);
  await user.click(await screen.findByRole('button',{name:'Favorilere ekle'}));
  expect(api.setOrganization).toHaveBeenCalledWith({recipeId:recipe.id,expectedRevision:1,favorite:true,toTry:false,rating:null});
  await user.click(screen.getByRole('button',{name:/Börek/}));
  await user.click(screen.getByRole('button',{name:'5 yıldız ver'}));
  expect(api.setOrganization).toHaveBeenCalledWith({recipeId:recipe.id,expectedRevision:1,favorite:false,toTry:false,rating:5});
  expect(screen.getByRole('button',{name:'5 yıldız ver'})).toHaveAttribute('aria-pressed');
});

test('collections use a named form, real count, membership and guarded deletion', async () => {
  const api=client(); const user=userEvent.setup();
  const group={id:crypto.randomUUID(),title:'İçli Tarifler',revision:2,activeCount:3};
  vi.mocked(api.collections).mockResolvedValue([group]);
  render(<LibraryPage client={api} section="collections"/>);
  expect(await screen.findByRole('button',{name:/İçli Tarifler · 3 etkin tarif/})).toBeVisible();
  await user.type(screen.getByLabelText('Koleksiyon adı'),'Yeni koleksiyon');
  await user.click(screen.getByRole('button',{name:'Koleksiyon oluştur'}));
  expect(api.saveCollection).toHaveBeenCalledWith(expect.objectContaining({title:'Yeni koleksiyon',expectedRevision:null}));
  await user.click(screen.getByRole('button',{name:'İçli Tarifler: Koleksiyonu sil'}));
  expect(screen.getByRole('dialog')).toBeVisible();
  await user.click(within(screen.getByRole('dialog')).getByRole('button',{name:'Vazgeç'}));
  expect(api.deleteCollection).not.toHaveBeenCalled();
});
test('safe deletion requires confirmation and trash offers restore', async () => {
  const api = client();
  const recipe: Recipe = {
    id: crypto.randomUUID(),
    revision: 1,
    title: 'Ayran',
    kind: 'beverage',
    servings: '1',
    description: null,
    prepMinutes: 1,
    cookMinutes: null,
    notes: null,
    createdAt: 'now',
    updatedAt: 'now',
    deletedAt: null,
    archivedAt: null,
    ingredients: [],
    steps: [],
    ingredientNames: {},
  };
  vi.mocked(api.list).mockResolvedValue([recipe]);
  vi.mocked(api.get).mockResolvedValue(recipe);
  vi.mocked(api.setDeleted).mockResolvedValue({
    ...recipe,
    deletedAt: 'now',
    revision: 2,
  });
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(await screen.findByRole('button', { name: /Ayran/ }));
  await user.click(screen.getByRole('button', { name: 'Çöp kutusuna taşı' }));
  expect(api.setDeleted).not.toHaveBeenCalled();
  const dialog = screen.getByRole('dialog');
  await user.click(within(dialog).getByRole('button', { name: 'Vazgeç' }));
  expect(api.setDeleted).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Çöp kutusuna taşı' }));
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', {
      name: 'Çöp kutusuna taşı',
    }),
  );
  await waitFor(() =>
    expect(api.setDeleted).toHaveBeenCalledWith(recipe.id, 1, true),
  );
  vi.mocked(api.get).mockResolvedValue({
    ...recipe,
    deletedAt: 'now',
    revision: 2,
  });
  await user.click(screen.getByRole('button', { name: 'Çöp kutusu' }));
  await user.click(await screen.findByRole('button', { name: /Ayran/ }));
  await user.click(screen.getByRole('button', { name: 'Geri yükle' }));
  await waitFor(() =>
    expect(api.setDeleted).toHaveBeenCalledWith(recipe.id, 2, false),
  );
});
test('save errors keep unfinished edits and cancel requires explicit discard', async () => {
  const api = client();
  vi.mocked(api.save).mockRejectedValue({
    code: 'STORAGE_BUSY',
    messageKey: 'errors.busy',
    recoverable: true,
  });
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(screen.getByRole('button', { name: 'Yeni tarif' }));
  await user.type(screen.getByLabelText('Tarif adı'), 'Çorba');
  await user.click(screen.getByRole('button', { name: 'Tarifi kaydet' }));
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'Veritabanı başka',
  );
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('Çorba');
  await user.click(screen.getByRole('button', { name: 'Vazgeç' }));
  expect(screen.getByRole('dialog')).toBeVisible();
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', {
      name: 'Düzenlemeye devam et',
    }),
  );
  expect(screen.getByLabelText('Tarif adı')).toHaveValue('Çorba');
});
test('archive scopes, duplication and permanent deletion are explicit UI actions', async () => {
  const api = client();
  const recipe: Recipe = {
    id: crypto.randomUUID(), revision: 1, title: 'Çay', kind: 'beverage',
    servings: '1', description: null, prepMinutes: null, cookMinutes: null,
    notes: null, createdAt: 'now', updatedAt: 'now', deletedAt: null,
    archivedAt: null, ingredients: [], steps: [], ingredientNames: {},
  };
  vi.mocked(api.scope).mockResolvedValue([recipe]);
  vi.mocked(api.get).mockResolvedValue(recipe);
  vi.mocked(api.archive).mockResolvedValue({ ...recipe, revision: 2, archivedAt: 'now' });
  vi.mocked(api.duplicate).mockResolvedValue({ ...recipe, id: crypto.randomUUID() });
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  await user.click(await screen.findByRole('button', { name: /Çay/ }));
  await user.click(screen.getByRole('button', { name: 'Arşivle' }));
  await screen.findByRole('button', { name: 'Arşivden çıkar' });
  expect(api.archive).toHaveBeenCalledWith(recipe.id, 1, true);
  await user.click(screen.getByRole('button', { name: 'Tarifi çoğalt' }));
  await screen.findByText('Tarif bağımsız bir kopya olarak oluşturuldu.');
  expect(api.duplicate).toHaveBeenCalledWith(recipe.id, 2);
  await user.click(screen.getByRole('button', { name: 'Kütüphaneye dön' }));
  await user.click(screen.getByRole('button', { name: 'Arşiv' }));
  await waitFor(() => expect(api.scope).toHaveBeenCalledWith('archived', null));
  vi.mocked(api.get).mockResolvedValue({ ...recipe, deletedAt: 'now', revision: 3 });
  await user.click(screen.getByRole('button', { name: 'Çöp kutusu' }));
  await user.click(await screen.findByRole('button', { name: /Çay/ }));
  await user.click(screen.getByRole('button', { name: 'Kalıcı olarak sil' }));
  expect(api.purge).not.toHaveBeenCalled();
  expect(screen.getByRole('dialog')).toHaveTextContent('geri alınamaz');
  expect(screen.getByRole('dialog')).toHaveTextContent('saklanan içerik sürümleri kalıcı olarak silinir');
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Vazgeç' }));
  expect(api.purge).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Kalıcı olarak sil' }));
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Kalıcı olarak sil' }));
  await waitFor(() => expect(api.purge).toHaveBeenCalledWith(recipe.id, 3));
});

test('search-first library keeps filters and card/list context with keyboard access', async () => {
  const api = client();
  const recipe: Recipe = {
    id: crypto.randomUUID(), revision: 1, title: 'İncirli tatlı', kind: 'food',
    servings: '2', description: 'Ev yapımı', prepMinutes: null, cookMinutes: null,
    notes: null, createdAt: '2026-10-01T12:00:00Z', updatedAt: '2026-10-02T12:00:00Z',
    deletedAt: null, archivedAt: null, ingredients: [], steps: [], ingredientNames: {},
  };
  vi.mocked(api.list).mockResolvedValue([recipe]);
  vi.mocked(api.get).mockResolvedValue(recipe);
  const user = userEvent.setup();
  render(<LibraryPage client={api} />);
  expect(await screen.findByRole('button', { name: /İncirli tatlı/ })).toBeVisible();
  const search = screen.getByRole('searchbox', { name: 'Tariflerde ara' });
  await user.type(search, 'incir');
  await waitFor(() => expect(api.searchRecipes).toHaveBeenLastCalledWith(
    expect.objectContaining({ text: 'incir', scope: 'active', limit: 24 }),
  ));
  await user.click(screen.getByRole('button', { name: 'Liste' }));
  expect(screen.getByRole('button', { name: /İncirli tatlı/ }).closest('.recipe-list-row')).toBeTruthy();
  search.focus();
  await user.keyboard('{ArrowDown}');
  expect(screen.getByRole('button', { name: /İncirli tatlı/ })).toHaveFocus();
  await user.click(screen.getByRole('button', { name: /İncirli tatlı/ }));
  expect(await screen.findByRole('button', { name: 'Kütüphaneye dön' })).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Kütüphaneye dön' }));
  expect(screen.getByRole('searchbox', { name: 'Tariflerde ara' })).toHaveValue('incir');
  expect(screen.getByRole('button', { name: 'Liste' })).toHaveAttribute('aria-pressed', 'true');
  await user.click(screen.getByRole('button', { name: 'Filtreleri temizle' }));
  expect(screen.getByRole('searchbox', { name: 'Tariflerde ara' })).toHaveValue('');
});
