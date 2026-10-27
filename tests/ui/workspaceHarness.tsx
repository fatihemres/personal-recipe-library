import React from 'react';
import { createRoot } from 'react-dom/client';
import '../../src/App.css';
import { AppShell } from '../../src/app/AppShell';
import type { FoundationClient } from '../../src/shared/api/client';
import type { RecipeClient } from '../../src/shared/api/recipes';
import type { Recipe } from '../../src/shared/contracts/recipe';
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
const recipes={
  searchRecipes:async(search:RecipeSearchQuery)=>({items:search.scope==='active'?[item]:[],total:search.scope==='active'?1:0,hasMore:false,suggestions:[]}),
  drafts:async()=>[], collections:async()=>[collection], savedFilters:async()=>filters,
  get:async()=>recipe, getOrganization:async()=>({recipeId,revision:1,favorite:false,toTry:false,rating:null,collectionIds:[]}),
  getPrivateNote:async()=>({recipeId,formatVersion:1 as const,markdown:Array.from({length:70},(_,i)=>`## Uzun not satırı ${i+1}`).join('\n'),revision:1,updatedAt:'2026-10-23T12:00:00Z'}),
  dashboard:async()=>({activeTotal:1,foodCount:1,beverageCount:0,favoriteCount:0,recentCreated:[item],recentModified:[item],collections:[collection]}),
} as RecipeClient;

createRoot(document.getElementById('root')!).render(<AppShell client={foundation} recipes={recipes}/>);
