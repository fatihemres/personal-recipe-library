import { useEffect, useRef } from 'react';
import { Button } from './button';
export function ConfirmDialog({
  title,
  description,
  confirm,
  cancel,
  onConfirm,
  onCancel,
}: {
  title: string;
  description: string;
  confirm: string;
  cancel: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const element = ref.current;
    element?.showModal();
    return () => element?.close();
  }, []);
  return (
    <dialog
      ref={ref}
      className="confirm-dialog"
      aria-labelledby="confirm-title"
      aria-describedby="confirm-description"
      onCancel={(e) => {
        e.preventDefault();
        onCancel();
      }}
    >
      <h2 id="confirm-title">{title}</h2>
      <p id="confirm-description">{description}</p>
      <div className="recipe-actions">
        <Button autoFocus variant="outline" onClick={onCancel}>
          {cancel}
        </Button>
        <Button onClick={onConfirm}>{confirm}</Button>
      </div>
    </dialog>
  );
}
