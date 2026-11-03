import { useEffect,useRef,useState } from 'react';
import type { Recipe } from '../../shared/contracts/recipe';
import type { PurgePreview } from '../../shared/contracts/recipeRelations';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { relationsTr as t } from '../../shared/i18n/recipeRelations';
import { recipesTr as r } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { RecipeLinkList } from './RecipeRelations';
export function RecipePurgeDialog({recipe,client,onCancel,onPurged}:{recipe:Recipe;client:RecipeClient;onCancel:()=>void;onPurged:()=>void}){
 const [preview,setPreview]=useState<PurgePreview|null>(null),[error,setError]=useState(''),[reading,setReading]=useState(true),[busy,setBusy]=useState(false),[reviewed,setReviewed]=useState(false);const mounted=useRef(true),lock=useRef(false);
 useEffect(()=>{let active=true;mounted.current=true;void client.previewPurge(recipe.id,recipe.revision,0,20).then(v=>{if(active){setPreview(v);setReading(false);}},e=>{if(active){setError(errorMessage(normalizeError(e).messageKey));setReading(false);}});return()=>{active=false;mounted.current=false;};},[client,recipe.id,recipe.revision]);
 async function read(more=false){if(lock.current)return;lock.current=true;setReading(true);setError('');try{const v=await client.previewPurge(recipe.id,recipe.revision,more?preview?.links.items.length??0:0,20);if(!mounted.current)return;if(more&&preview&&v.token!==preview.token){setPreview(null);setReviewed(false);throw{messageKey:'errors.purgeReviewStale',code:'PURGE_REVIEW_STALE',recoverable:true};}setPreview(more&&preview?{...v,links:{...v.links,items:[...preview.links.items,...v.links.items]}}:v);setReviewed(false);}catch(e){if(mounted.current)setError(errorMessage(normalizeError(e).messageKey));}finally{lock.current=false;if(mounted.current)setReading(false);}}
 async function purge(){if(lock.current||!preview)return;lock.current=true;setBusy(true);setError('');try{await client.purgeReviewed({recipeId:recipe.id,expectedRevision:recipe.revision,token:preview.token,acknowledgeDetach:reviewed});if(mounted.current)onPurged();}catch(e){if(mounted.current){setError(errorMessage(normalizeError(e).messageKey));setPreview(null);setReviewed(false);}}finally{lock.current=false;if(mounted.current)setBusy(false);}}
 if(reading&&!preview&&!error)return <p role="status">{t.loading}</p>;
 const needsReview=!!preview?.links.total&&!reviewed;
 return <ConfirmDialog focusKey={needsReview?'review':'final'} title={needsReview?t.purgeReview:reviewed?t.finalTitle:r.confirmPurge} description={needsReview?t.purgeIntro:reviewed?t.finalHelp:r.purgeHelp} confirm={needsReview?t.reviewed:reviewed?t.detachPurge:r.purge} cancel={t.cancel} busy={busy} error={error} confirmDisabled={!preview||reading||!!preview.links.hasMore} onCancel={onCancel} onConfirm={()=>{if(needsReview)setReviewed(true);else void purge();}}>
 {preview&&<><p>{t.count}: {preview.links.total}</p><RecipeLinkList items={preview.links.items}/>{preview.links.hasMore&&<Button variant="outline" disabled={reading} onClick={()=>void read(true)}>{t.more}</Button>}</>}
 {(!preview||error)&&<Button variant="outline" disabled={reading||busy} onClick={()=>void read()}>{t.refresh}</Button>}
 </ConfirmDialog>;
}
