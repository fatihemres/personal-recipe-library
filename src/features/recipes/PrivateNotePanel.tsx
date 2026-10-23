import { useEffect, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
import { SafeMarkdown } from './SafeMarkdown';
import { ConfirmDialog } from '../../shared/ui/ConfirmDialog';
import { registerPrivateNoteExitGuard } from '../../app/nativeExit';

export function PrivateNotePanel({recipeId,client,registerLeaveGuard,readOnly=false}:{recipeId:string;client:RecipeClient;registerLeaveGuard?:(guard:(()=>Promise<boolean>)|null)=>void;readOnly?:boolean}) {
  const [markdown,setMarkdown]=useState('');
  const [committed,setCommitted]=useState('');
  const [revision,setRevision]=useState(0);
  const [loading,setLoading]=useState(true);
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState('');
  const [preview,setPreview]=useState(false);
  const [leaving,setLeaving]=useState(false);
  const resolver=useRef<((allowed:boolean)=>void)|null>(null);
  const pending=useRef<Promise<boolean>|null>(null);
  useEffect(()=>()=>{resolver.current?.(false);resolver.current=null;},[]);
  useEffect(()=>{
    let alive=true;
    void client.getPrivateNote(recipeId).then(note=>{
      if(alive){setMarkdown(note.markdown);setCommitted(note.markdown);setRevision(note.revision);setLoading(false);}
    }, e=>{if(alive){setError(errorMessage(normalizeError(e).messageKey));setLoading(false);}});
    return ()=>{alive=false;};
  },[client,recipeId]);
  async function save():Promise<boolean>{
    setBusy(true);setError('');
    try {const note=await client.savePrivateNote({recipeId,expectedRevision:revision,markdown});setRevision(note.revision);setCommitted(note.markdown);return true;}
    catch(e){setError(errorMessage(normalizeError(e).messageKey));return false;}
    finally{setBusy(false);}
  }
  function resolve(allowed:boolean){setLeaving(false);resolver.current?.(allowed);resolver.current=null;}
  const askLeave=()=>{
    if(readOnly) return Promise.resolve(true);
    if(pending.current) return pending.current;
    if(loading || markdown===committed) return Promise.resolve(true);
    pending.current=new Promise<boolean>(done=>{resolver.current=value=>{pending.current=null;done(value);};setLeaving(true);});
    return pending.current;
  };
  useEffect(()=>{registerLeaveGuard?.(askLeave);return()=>registerLeaveGuard?.(null);});
  useEffect(()=>registerPrivateNoteExitGuard(async()=>{
    if(readOnly) return;
    if(markdown!==committed){const ok=await save();if(!ok) throw new Error('PRIVATE_NOTE_SAVE_FAILED');}
  }));
  return <section className="panel private-note" aria-label={t.privateNotes}>
    <h2>{t.privateNotes}</h2>
    <p className="muted">{readOnly?t.noteRestoreToEdit:t.privateNotesHelp}</p>
    {loading ? <p role="status">{t.loading}</p> : <>
      {error && <p role="alert" className="recipe-error">{error}</p>}
      {readOnly ? (markdown ? <SafeMarkdown source={markdown}/> : <p>{t.empty}</p>) : <>
      <div className="recipe-actions" role="group" aria-label={t.privateNotes}>
        <Button variant={!preview?'default':'outline'} aria-pressed={!preview} onClick={()=>setPreview(false)}>{t.noteEdit}</Button>
        <Button variant={preview?'default':'outline'} aria-pressed={preview} onClick={()=>setPreview(true)}>{t.notePreview}</Button>
      </div>
      {preview ? <SafeMarkdown source={markdown} /> : <label className="note-editor">{t.privateNotes}
        <textarea value={markdown} maxLength={50_000} rows={9} onChange={e=>setMarkdown(e.target.value)} /></label>}
      <div className="recipe-actions">
        <Button disabled={busy || markdown===committed} onClick={()=>void save()}>{busy?t.saving:t.noteSave}</Button>
        <Button variant="outline" disabled={busy || !markdown} onClick={()=>setMarkdown('')}>{t.noteClear}</Button>
        {markdown!==committed && <span role="status">{t.noteUnsaved}</span>}
      </div>
      {leaving && <ConfirmDialog title={t.noteLeaveTitle} description={t.noteLeaveHelp} confirm={t.noteSave} cancel={t.noteContinue} busy={busy} error={error}
        extra={{label:t.noteDiscard,onClick:()=>resolve(true)}} onCancel={()=>resolve(false)} onConfirm={()=>{void save().then(ok=>{if(ok)resolve(true);});}} />}
      </>}
    </>}
  </section>;
}
