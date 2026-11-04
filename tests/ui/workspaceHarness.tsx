import { RecipePurgeDialog } from '../../src/features/recipes/RecipePurgeDialog';
import { createRoot } from 'react-dom/client';
import '../../src/App.css';
import { AppShell } from '../../src/app/AppShell';
import type { FoundationClient } from '../../src/shared/api/client';
import type { RecipeClient } from '../../src/shared/api/recipes';
import type { Recipe, RecipeInput } from '../../src/shared/contracts/recipe';
import type { RecipeSearchItem, RecipeSearchQuery } from '../../src/shared/contracts/recipeSearch';
import type { RecipeCollection } from '../../src/shared/contracts/recipeOrganization';
import type { SavedFilter } from '../../src/shared/contracts/recipeWorkspace';

// Browser-only presentation data. This renders the real AppShell and recipe components;
// Rust/SQLite integration tests remain the proof of durable behavior.
const recipeId='11111111-1111-4111-8111-111111111111';
const collectionId='22222222-2222-4222-8222-222222222222';
const longTitle='Kış sofraları için yöresel sebzeli tarifler ve uzun açıklamalı içecek eşlikçileri';
const longFilter='Kış menüsü · yöresel yemekler, içecekler ve uzun açıklamalı denenecek tarifler';
const recipe:Recipe={id:recipeId,title:longTitle,description:'Uzun Türkçe açıklama: '.repeat(12),kind:'food',servings:'2',
  prepMinutes:15,cookMinutes:30,notes:'Tarif notu '.repeat(12),createdAt:'2026-10-23T12:00:00Z',updatedAt:'2026-10-23T12:00:00Z',
  deletedAt:null,archivedAt:null,revision:1,ingredients:[],steps:[],ingredientNames:{}};
if(new URLSearchParams(location.search).has('steps')) {
  recipe.ingredients=[{id:'44444444-4444-4444-8444-444444444444',ingredientId:'55555555-5555-4555-8555-555555555555',quantity:'1.250',unitCode:'g',note:'İnce doğranmış uzun Türkçe malzeme satırı açıklaması'}];
  recipe.ingredientNames={'55555555-5555-4555-8555-555555555555':'Şişli yöresinden ince doğranmış mantar ve uzun malzeme adı'};
  recipe.steps=[{id:'66666666-6666-4666-8666-666666666666',instructions:'Malzemeleri hazırlayın ve dikkatlice karıştırın. '.repeat(20)}, {id:'77777777-7777-4777-8777-777777777777',instructions:'Kısık ateşte pişirin'}];
}
if(new URLSearchParams(location.search).has('alternatives')){
 recipe.ingredients=[{id:'44444444-4444-4444-8444-444444444444',ingredientId:'55555555-5555-4555-8555-555555555555',quantity:'1.250',unitCode:'g',note:null,alternatives:[{id:'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',ingredientId:'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',quantity:'0.123456',unitCode:'mL'},{id:'cccccccc-cccc-4ccc-8ccc-cccccccccccc',ingredientId:'dddddddd-dddd-4ddd-8ddd-dddddddddddd',quantity:null,unitCode:null}]},{id:'12121212-1212-4212-8212-121212121212',ingredientId:'55555555-5555-4555-8555-555555555555',quantity:null,unitCode:'g',note:'İkinci bağımsız satır'}];
 recipe.ingredientNames={'55555555-5555-4555-8555-555555555555':'Ana malzeme — uzun Türkçe açıklama','bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb':'Alternatif — Şişli yöresinden çok uzun açıklamalı malzeme adı','dddddddd-dddd-4ddd-8ddd-dddddddddddd':'İsteğe bağlı miktarsız alternatif'};
}
const item:RecipeSearchItem={id:recipe.id,title:recipe.title,description:recipe.description,kind:recipe.kind,servings:recipe.servings,
  createdAt:recipe.createdAt,updatedAt:recipe.updatedAt,organizationRevision:1,favorite:false,toTry:false,rating:null};
const collection:RecipeCollection={id:collectionId,title:longTitle,revision:1,activeCount:1};
const query:RecipeSearchQuery={text:'',scope:'active',kind:null,ingredientId:null,favoriteOnly:false,toTryOnly:false,
  collectionId:null,createdFrom:null,createdTo:null,modifiedFrom:null,modifiedTo:null,sort:'modified',offset:0,limit:24};
const filters:SavedFilter[]=[{id:'33333333-3333-4333-8333-333333333333',title:longFilter,revision:1,formatVersion:1,query,available:true}];
const foundation:FoundationClient={
  bootstrap:async()=>({preferences:{theme:new URLSearchParams(location.search).get('theme')==='dark'?'dark':'light',locale:'tr'},
    storage:{schemaVersion:8,sqliteVersion:'3.47.0',foreignKeys:true,fts5:true}}),
  savePreferences:async preferences=>preferences,
};
const unsupported = async () => { throw new Error('Unsupported browser presentation fixture operation'); };
const snapshot=()=>({formatVersion:(new URLSearchParams(location.search).has('alternatives')?2:1) as 1|2,recipeId:recipe.id,recipeCreatedAt:recipe.createdAt,title:recipe.title,description:recipe.description,kind:recipe.kind,servings:recipe.servings,prepMinutes:recipe.prepMinutes,cookMinutes:recipe.cookMinutes,notes:recipe.notes,ingredients:recipe.ingredients,steps:recipe.steps,ingredientNames:recipe.ingredientNames});
const summary=(n:number)=>({recipeId,versionNumber:n,formatVersion:new URLSearchParams(location.search).has('alternatives')?2:1,sourceRevision:n,provenance:n===1?'created' as const:'edited' as const,restoredFromVersion:null,capturedAt:'2026-11-02T12:00:00Z',contentFingerprint:'a'.repeat(64),snapshotChecksum:'b'.repeat(64)});
const recipes: RecipeClient={
  links:async()=>({graphRevision:1,items:[{id:'88888888-8888-4888-8888-888888888888',title:'Uzun Türkçe üst tarif adı — '+longTitle,revision:1,status:'archived',relationship:'parent'}],total:1,hasMore:false}),createVariation:unsupported,setParent:async()=>null,setRelated:async()=>null,
  previewPurge:async(id,revision)=>({recipeId:id,recipeRevision:revision,token:'a'.repeat(64),links:{graphRevision:1,items:[],total:0,hasMore:false}}),purgeReviewed:async()=>null,
  versions:async()=>[summary(2),summary(1)],
  version:async(_id,version)=>({summary:summary(version),snapshot:snapshot()}),
  compareVersions:async()=>({currentRevision:1,changes:[{path:['ingredients','0','quantity'],before:'1.250',after:'1.25'},{path:['steps','0','details','title'],before:'Uzun Türkçe aşama başlığı '.repeat(6),after:'Yeni aşama'}]}),
  restoreVersion:async()=>({recipe:{...recipe,revision:2},changed:true,versionNumber:3}),
  savePrivateNote:unsupported,resolveFilter:unsupported,saveFilter:unsupported,deleteFilter:unsupported,
  setOrganization:unsupported,saveCollection:unsupported,deleteCollection:unsupported,setMembership:unsupported,
  list:unsupported,save:unsupported,setDeleted:unsupported,searchIngredients:unsupported,createIngredient:unsupported,
  getDraft:unsupported,duplicate:unsupported,scope:unsupported,archive:unsupported,purge:unsupported,
  editIngredient:unsupported,deleteIngredient:unsupported,
  searchRecipes:async(search:RecipeSearchQuery)=>({items:search.scope==='active'?[search.limit===10?{...item,id:'99999999-9999-4999-8999-999999999999',title:'Etkin hedef tarif — '+longTitle}:item]:[],total:search.scope==='active'?1:0,hasMore:false,suggestions:[]}),
  units:async()=>[{code:'g',dimension:'mass',canonicalCode:'g',factor:1},{code:'mL',dimension:'volume',canonicalCode:'mL',factor:1}],
  searchPersonal:async()=>({items:[],total:0,hasMore:false}),
  saveDraft:async(id:string,revision:number|null,input:RecipeInput)=>({id,revision:(revision??0)+1,recipeId:input.id,input,updatedAt:'now',ingredientNames:recipe.ingredientNames}),
  discardDraft:async()=>null,
  commit:async(input:RecipeInput)=>({...recipe,...input,revision:2}),
  drafts:async()=>[], collections:async()=>[collection], savedFilters:async()=>filters,
  get:async(id)=>({...recipe,id,title:id===recipeId?recipe.title:'Hedef tarif'}), getOrganization:async()=>({recipeId,revision:1,favorite:false,toTry:false,rating:null,collectionIds:[]}),
  getPrivateNote:async()=>({recipeId,formatVersion:1 as const,markdown:Array.from({length:70},(_,i)=>`## Uzun not satırı ${i+1}`).join('\n'),revision:1,updatedAt:'2026-10-23T12:00:00Z'}),
  dashboard:async()=>({activeTotal:1,foodCount:1,beverageCount:0,favoriteCount:0,recentCreated:[item],recentModified:[item],collections:[collection]}),
};

const purgeFixture:RecipeClient={...recipes,previewPurge:async(id,revision,offset)=>({recipeId:id,recipeRevision:revision,token:'a'.repeat(64),links:{graphRevision:2,total:2,hasMore:offset===0,items:[{id:offset===0?'88888888-8888-4888-8888-888888888888':'99999999-9999-4999-8999-999999999999',title:longTitle,revision:1,status:'active',relationship:offset===0?'child':'related'}]}}),purgeReviewed:async()=>{throw {code:'PURGE_REVIEW_STALE',messageKey:'errors.purgeReviewStale',recoverable:true};}};
createRoot(document.getElementById('root')!).render(<><AppShell client={foundation} recipes={recipes}/>{new URLSearchParams(location.search).has('purge')&&<RecipePurgeDialog recipe={{...recipe,deletedAt:'2026-11-03T12:00:00Z'}} client={purgeFixture} onCancel={()=>{}} onPurged={()=>{}}/>}</>);
