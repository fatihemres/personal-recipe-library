import { useState } from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { ConfirmDialog } from './ConfirmDialog';

function Harness({ onConfirm = vi.fn() }: { onConfirm?: () => void }) {
  const [open, setOpen] = useState(false);
  return <>
    <button onClick={() => setOpen(true)}>Aç</button>
    {open && <ConfirmDialog
      title="Kaydedilmemiş değişiklikler"
      description="Bir karar verin"
      cancel="Düzenlemeye devam et"
      confirm="Kaydet ve devam et"
      extra={{ label: 'Vazgeç ve devam et', onClick: () => setOpen(false) }}
      onCancel={() => setOpen(false)}
      onConfirm={onConfirm}
    />}
  </>;
}

test('focuses the safe choice and restores focus when Cancel closes the modal', async () => {
  const user = userEvent.setup();
  render(<Harness />);
  const trigger = screen.getByRole('button', { name: 'Aç' });
  await user.click(trigger);
  const dialog = screen.getByRole('dialog');
  expect(dialog).toHaveAttribute('open');
  const cancel = screen.getByRole('button', { name: 'Düzenlemeye devam et' });
  expect(cancel).toHaveFocus();
  await user.tab();
  expect(screen.getByRole('button', { name: 'Vazgeç ve devam et' })).toHaveFocus();
  await user.tab();
  expect(screen.getByRole('button', { name: 'Kaydet ve devam et' })).toHaveFocus();
  await user.click(cancel);
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(trigger).toHaveFocus();
});

test('Escape cancels safely; Save keeps the dialog open until its owner succeeds', async () => {
  const user = userEvent.setup();
  const save = vi.fn();
  render(<Harness onConfirm={save} />);
  const trigger = screen.getByRole('button', { name: 'Aç' });
  await user.click(trigger);
  fireEvent(screen.getByRole('dialog'), new Event('cancel', { cancelable: true }));
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(trigger).toHaveFocus();
  await user.click(trigger);
  await user.click(screen.getByRole('button', { name: 'Kaydet ve devam et' }));
  expect(save).toHaveBeenCalledOnce();
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Vazgeç ve devam et' }));
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
});

test('an in-flight save disables destructive choices and Escape', () => {
  const cancel = vi.fn();
  render(<ConfirmDialog title="Kaydedilmemiş değişiklikler" description="Bekleyin" confirm="Kaydet" cancel="Devam et" extra={{ label: 'Vazgeç', onClick: vi.fn() }} busy error="Kayıt bekliyor" onConfirm={vi.fn()} onCancel={cancel} />);
  const dialog = screen.getByRole('dialog');
  expect(screen.getByRole('alert')).toHaveTextContent('Kayıt bekliyor');
  for (const button of screen.getAllByRole('button')) expect(button).toBeDisabled();
  fireEvent(dialog, new Event('cancel', { cancelable: true }));
  expect(cancel).not.toHaveBeenCalled();
  expect(dialog).toHaveAttribute('open');
});
