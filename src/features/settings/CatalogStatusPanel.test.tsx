import { render, screen } from '@testing-library/react';
import { expect, test, vi } from 'vitest';
import { CatalogStatusPanel } from './CatalogStatusPanel';
import { catalogClient } from '../../shared/api/catalog';
vi.mock('../../shared/api/catalog', () => ({ catalogClient: { status: vi.fn() } }));
test('shows actual supplied counts without presenting validation as production', async () => {
  vi.mocked(catalogClient.status).mockResolvedValue({ definitions: 8, validationDefinitions: 8, productionDefinitions: 0, pendingCollisions: 1 });
  render(<CatalogStatusPanel />);
  expect(await screen.findByText('8')).toBeInTheDocument();
  expect(screen.getByText('Üretim kayıtları').nextElementSibling).toHaveTextContent('0');
  expect(screen.getByText(/Doğrulama kayıtları üretim kataloğu değildir/)).toBeInTheDocument();
});
test('shows storage errors without invented counts', async () => {
  vi.mocked(catalogClient.status).mockRejectedValue({ code: 'STORAGE_UNAVAILABLE', messageKey: 'errors.storage', recoverable: true });
  render(<CatalogStatusPanel />);
  expect(await screen.findByRole('alert')).toHaveTextContent('Depolama açılamadı');
});
