import { render, screen } from '@testing-library/react';
import { expect, test, vi } from 'vitest';
import { ErrorBoundary } from './ErrorBoundary';
function Broken(): never { throw new Error('test-only failure'); }
test('unexpected rendering failure offers a Turkish recovery state', () => {
  vi.spyOn(console, 'error').mockImplementation(() => undefined);
  render(<ErrorBoundary><Broken /></ErrorBoundary>);
  expect(screen.getByRole('alert')).toHaveTextContent('beklenmeyen bir sorun');
  expect(screen.getByRole('button', { name: 'Uygulamayı yeniden yükle' })).toBeVisible();
});
