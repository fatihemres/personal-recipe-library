import { useCallback, useEffect, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import type { Dashboard } from '../../shared/contracts/recipeWorkspace';
import { Button } from '../../shared/ui/button';

type Destination = {kind?:'food'|'beverage';favoriteOnly?:boolean;collectionId?:string;recipeId?:string};
export function DashboardPage({client,onNavigate}:{client:RecipeClient;onNavigate:(destination:Destination)=>void}) {
  const [data,setData]=useState<Dashboard|null>(null);
  const [error,setError]=useState('');
  const [loading,setLoading]=useState(true);
  const load=useCallback(async()=>{
    setLoading(true);setError('');
    try{setData(await client.dashboard());}
    catch(e){setData(null);setError(errorMessage(normalizeError(e).messageKey));}
    finally{setLoading(false);}
  },[client]);
  useEffect(()=>{void load();},[load]);
  return <section>
    <div className="page-heading"><div><p className="eyebrow">{t.brand}</p><h1>{t.dashboardTitle}</h1><p>{t.dashboardIntro}</p></div></div>
    {loading && <p role="status">{t.loading}</p>}
    {error && <div role="alert" className="recipe-error"><p>{error}</p><Button variant="outline" onClick={()=>void load()}>{t.retry}</Button></div>}
    {!loading && data && <>
      <div className="dashboard-metrics">
        {([
          [t.dashboardActive,data.activeTotal,{}],
          [t.dashboardFood,data.foodCount,{kind:'food'}],
          [t.dashboardBeverage,data.beverageCount,{kind:'beverage'}],
          [t.dashboardFavorites,data.favoriteCount,{favoriteOnly:true}],
        ] as const).map(([label,count,destination])=><Button key={label} className="dashboard-metric panel" variant="outline" onClick={()=>onNavigate(destination)}><span>{label}</span><strong>{count}</strong></Button>)}
      </div>
      {!data.activeTotal && <div className="panel empty-state"><p>{t.dashboardEmpty}</p><Button onClick={()=>onNavigate({})}>{t.dashboardOpenLibrary}</Button></div>}
      <div className="dashboard-sections">
        <section className="panel"><h2>{t.dashboardRecentCreated}</h2>{data.recentCreated.length ? <ul>{data.recentCreated.map(item=><li key={item.id}><Button variant="ghost" onClick={()=>onNavigate({recipeId:item.id})}>{item.title}</Button></li>)}</ul> : <p>{t.empty}</p>}</section>
        <section className="panel"><h2>{t.dashboardRecentModified}</h2>{data.recentModified.length ? <ul>{data.recentModified.map(item=><li key={item.id}><Button variant="ghost" onClick={()=>onNavigate({recipeId:item.id})}>{item.title}</Button></li>)}</ul> : <p>{t.empty}</p>}</section>
        <section className="panel"><h2>{t.dashboardCollections}</h2>{data.collections.length ? <ul>{data.collections.map(group=><li key={group.id}><Button variant="ghost" onClick={()=>onNavigate({collectionId:group.id})}>{group.title} · {t.collectionCount.replace('{count}',String(group.activeCount))}</Button></li>)}</ul> : <p>{t.collectionEmpty}</p>}</section>
      </div>
    </>}
  </section>;
}
