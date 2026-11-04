import { useEffect, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { mediaTr as t } from '../../shared/i18n/media';
import { Button } from '../../shared/ui/button';
export function CoverImage({assetId,variant,client}:{assetId?:string|null;variant:'thumb'|'preview';client:RecipeClient}) {
 const [failed,setFailed]=useState(false);const [attempt,setAttempt]=useState(0);
 useEffect(()=>{setFailed(false);},[assetId]);
 if(!assetId)return null;
 if(failed)return <div className={`cover-unavailable cover-${variant}`}><p role="status">{t.unavailable}</p>{variant==='preview'&&<Button type="button" variant="outline" onClick={()=>{setFailed(false);setAttempt(n=>n+1);}}>{t.retry}</Button>}</div>;
 return <img key={`${assetId}:${attempt}`} className={`recipe-cover cover-${variant}`} src={client.coverUrl(assetId,variant)} alt={variant==='thumb'?'':t.cover} loading="lazy" decoding="async" onError={()=>setFailed(true)} />;
}
export function RecipeCoverEditor({assetId,client,onChange,onBusy}:{assetId?:string|null;client:RecipeClient;onChange:(id:string|null)=>void;onBusy:(busy:boolean)=>void}) {
 const [busy,setBusy]=useState(false);const [error,setError]=useState('');const mounted=useRef(true);const pending=useRef(false);
 useEffect(()=>{mounted.current=true;return()=>{mounted.current=false;};},[]);
 async function choose(){
  if(pending.current)return;pending.current=true;setBusy(true);setError('');onBusy(true);
  try{const cover=await client.pickCover();if(mounted.current&&cover)onChange(cover.id);}
  catch(e){if(mounted.current)setError(errorMessage(normalizeError(e).messageKey));}
  finally{pending.current=false;if(mounted.current){setBusy(false);onBusy(false);}}
 }
 return <section className="panel recipe-form cover-editor" aria-label={t.cover} aria-busy={busy}>
  <h2>{t.cover}</h2><p>{t.help}</p><details><summary>{t.privacy}</summary><p>{t.retention}</p></details><CoverImage assetId={assetId} variant="preview" client={client}/>
  <div className="recipe-actions"><Button type="button" variant="outline" disabled={busy} onClick={()=>void choose()}>{assetId?t.replace:t.add}</Button>{assetId&&<Button type="button" variant="ghost" disabled={busy} onClick={()=>{setError('');onChange(null);}}>{t.remove}</Button>}</div>
  {busy&&<p role="status">{t.importing}</p>}{error&&<p role="alert">{error}</p>}<p>{t.saveHelp}</p>
 </section>;
}
