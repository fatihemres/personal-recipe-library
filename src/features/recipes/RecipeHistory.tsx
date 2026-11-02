import { useEffect, useRef, useState } from 'react';
import type { Recipe } from '../../shared/contracts/recipe';
import type { RecipeVersion, RecipeVersionSummary, VersionComparison } from '../../shared/contracts/recipeHistory';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { historyTr as t } from '../../shared/i18n/recipeHistory';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { RecipeDetails } from './RecipeDetails';
export function RecipeHistory({recipe,client,onRestored,beforeRestore}: {recipe:Recipe;client:RecipeClient;onRestored:(recipe:Recipe)=>void;beforeRestore:()=>Promise<boolean>}) {
 const [open,setOpen]=useState(false),[rows,setRows]=useState<RecipeVersionSummary[]>([]),[more,setMore]=useState(false);
 const [selected,setSelected]=useState<RecipeVersion|null>(null),[target,setTarget]=useState('current'),[comparison,setComparison]=useState<VersionComparison|null>(null);
 const [error,setError]=useState(''),[busy,setBusy]=useState(false),[confirm,setConfirm]=useState(false),[notice,setNotice]=useState(''),[changeLimit,setChangeLimit]=useState(50);
 const mounted=useRef(true),locked=useRef(false);
 useEffect(()=>{mounted.current=true;return()=>{mounted.current=false;};},[]);
 async function run(action:()=>Promise<void>) {
  if(locked.current)return;locked.current=true;setBusy(true);setError('');
  try{await action();}catch(e){if(mounted.current)setError(errorMessage(normalizeError(e).messageKey));}
  finally{locked.current=false;if(mounted.current)setBusy(false);}
 }
 function load(older=false) {void run(async()=>{
  const next=await client.versions(recipe.id,older?rows[rows.length-1]?.versionNumber??null:null,20);
  if(!mounted.current)return;setRows(previous=>older?[...previous,...next]:next);setMore(next.length===20);
 });}
 function choose(version:number) {void run(async()=>{
  setSelected(null);setComparison(null);
  const result=await client.version(recipe.id,version);
  if(!mounted.current)return;setSelected(result);setComparison(null);setNotice('');setTarget('current');
 });}
 const date=(value:string)=>new Intl.DateTimeFormat('tr-TR',{dateStyle:'medium',timeStyle:'short'}).format(new Date(value));
 return <section className="panel workspace-panel recipe-history" aria-label={t.heading}>
  <div className="workspace-heading"><h2>{t.heading}</h2><Button variant="outline" aria-expanded={open} disabled={busy} onClick={()=>{setOpen(!open);if(!open)load();}}>{open?t.close:t.open}</Button></div>
  {open&&<>
   <p>{t.intro}</p>
   {busy&&<p role="status">{t.loading}</p>}
   {notice&&<p role="status">{notice}</p>}
   {error&&<div role="alert"><p>{error}</p><Button variant="outline" disabled={busy} onClick={()=>load()}>{t.retry}</Button></div>}
   {!busy&&!rows.length&&!error&&<p>{t.empty}</p>}
   <ol className="history-list">{rows.map(row=><li key={row.versionNumber}>
    <div><strong>V{row.versionNumber} · {t[row.provenance]}</strong><p><time dateTime={row.capturedAt}>{date(row.capturedAt)}</time> · {t.sourceRevision}: {row.sourceRevision}{row.restoredFromVersion!==null&&` · ${t.restoredFrom}: V${row.restoredFromVersion}`}</p>
    {row.provenance==='upgrade_baseline'&&<p>{t.baselineHelp}</p>}</div>
    <Button variant="outline" disabled={busy} aria-label={`V${row.versionNumber} · ${t.preview}`} onClick={()=>choose(row.versionNumber)}>{t.preview}</Button>
   </li>)}</ol>
   {more&&<Button disabled={busy} variant="outline" onClick={()=>load(true)}>{t.more}</Button>}
   {selected&&<section aria-label={t.selected} className="history-preview">
    <h3>V{selected.summary.versionNumber} · {selected.snapshot.title}</h3>
    {selected.snapshot.description&&<p className="preserve-text">{selected.snapshot.description}</p>}
    <RecipeDetails recipe={selected.snapshot}/>
    <details><summary>{t.identities}</summary><ul className="history-identities">{selected.snapshot.ingredients.map((line,i)=><li key={line.id}>{t.line} {i+1}: <code>{line.id}</code> · <code>{line.ingredientId}</code></li>)}{selected.snapshot.steps.map((step,i)=><li key={step.id}>{t.step} {i+1}: <code>{step.id}</code></li>)}</ul></details>
    <div className="history-controls">
     <label>{t.compareWith}<select value={target} disabled={busy} onChange={e=>{setTarget(e.target.value);setComparison(null);}}><option value="current">{t.current}</option>{rows.map(row=><option key={row.versionNumber} value={row.versionNumber}>V{row.versionNumber}</option>)}</select></label>
     <Button disabled={busy} onClick={()=>void run(async()=>{const result=await client.compareVersions(recipe.id,selected.summary.versionNumber,target==='current'?null:Number(target));if(mounted.current){setComparison(result);setChangeLimit(50);}})}>{t.compare}</Button>
    </div>
    {comparison&&<section aria-label={t.compare}>
     {!comparison.changes.length?<p>{t.noChanges}</p>:<><p>{t.before} → {target==='current'?t.current:`V${target}`}</p><ul className="history-differences">{comparison.changes.slice(0,changeLimit).map((change,i)=><li key={i}><strong>{change.path.map(part=>/^\d+$/.test(part)?String(Number(part)+1):t.fields[part]??part).join(' / ')}</strong><dl><dt>{t.before}</dt><dd><code>{JSON.stringify(change.before)}</code></dd><dt>{t.after}</dt><dd><code>{JSON.stringify(change.after)}</code></dd></dl></li>)}</ul>{comparison.changes.length>changeLimit&&<Button onClick={()=>setChangeLimit(changeLimit+50)}>{t.moreChanges}</Button>}</>}
    </section>}
    {recipe.archivedAt||recipe.deletedAt?<p>{t.inactive}</p>:<Button disabled={busy} onClick={()=>void (async()=>{let allowed=false;await run(async()=>{allowed=await beforeRestore();});if(allowed&&mounted.current)setConfirm(true);})()}>{t.restore}</Button>}
   </section>}
  </>}
  {confirm&&selected&&<ConfirmDialog title={t.confirmTitle} description={`${t.confirmBody} (V${selected.summary.versionNumber})`} confirm={t.restore} cancel={t.cancel} busy={busy} error={error} onCancel={()=>{setConfirm(false);setError('');}} onConfirm={()=>void run(async()=>{
   const result=await client.restoreVersion({recipeId:recipe.id,versionNumber:selected.summary.versionNumber,expectedRevision:recipe.revision});
   if(!mounted.current)return;setConfirm(false);setNotice(result.changed?t.restoredNotice:t.unchanged);setComparison(null);
   onRestored(result.recipe);
   const next=await client.versions(recipe.id,null,20);if(mounted.current){setRows(next);setMore(next.length===20);}
  })}/>} 
 </section>;
}
