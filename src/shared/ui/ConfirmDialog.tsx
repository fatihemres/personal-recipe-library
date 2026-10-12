import { useEffect, useRef } from 'react';
import { Button } from './button';
export function ConfirmDialog({
  title,
  description,
  confirm,
  cancel,
  onConfirm,
  onCancel,
  extra,
  busy = false,
  error,
}: {
  title: string;
  description: string;
  confirm: string;
  cancel: string;
  onConfirm: () => void;
  onCancel: () => void;
  extra?: {label:string;onClick:()=>void};
  busy?: boolean;
  error?: string;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const restoreOnClose = useRef(false);
  useEffect(() => {
    const element = ref.current;
    if (!previousFocus.current) {
      previousFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    }
    element?.showModal();
    cancelRef.current?.focus();
    return () => {
      element?.close();
      if (restoreOnClose.current && previousFocus.current?.isConnected) {
        previousFocus.current.focus();
      }
    };
  }, []);
  const cancelDialog = () => {
    if (busy) return;
    restoreOnClose.current = true;
    onCancel();
  };
  return (
    <dialog
      ref={ref}
      className="confirm-dialog"
      aria-labelledby="confirm-title"
      aria-describedby="confirm-description"
      onCancel={(e) => {
        e.preventDefault();
        cancelDialog();
      }}
    >
      <h2 id="confirm-title">{title}</h2>
      <p id="confirm-description">{description}</p>
      {error && <p role="alert" className="recipe-error">{error}</p>}
      <div className="recipe-actions">
        <Button ref={cancelRef} type="button" autoFocus variant="outline" disabled={busy} onClick={cancelDialog}>
          {cancel}
        </Button>
        {extra && <Button type="button" disabled={busy} onClick={extra.onClick}>{extra.label}</Button>}
        <Button type="button" disabled={busy} onClick={onConfirm}>{confirm}</Button>
      </div>
    </dialog>
  );
}
