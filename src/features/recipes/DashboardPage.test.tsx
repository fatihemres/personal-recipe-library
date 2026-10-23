import { render,screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import { rendererRecipeClient } from '../../test/recipeClient';
import { DashboardPage } from './DashboardPage';

it('shows only SQLite-backed counts from IPC and navigates with the correct filter',async()=>{
  const client=rendererRecipeClient();
  vi.mocked(client.dashboard).mockResolvedValue({activeTotal:3,foodCount:2,beverageCount:1,favoriteCount:1,recentCreated:[],recentModified:[],collections:[]});
  const onNavigate=vi.fn();render(<DashboardPage client={client} onNavigate={onNavigate}/>);
  await screen.findByRole('button',{name:/Favori tarif\s*1/});
  await userEvent.click(screen.getByRole('button',{name:/Favori tarif\s*1/}));
  expect(onNavigate).toHaveBeenCalledWith({favoriteOnly:true});
});

it('distinguishes storage errors from empty data',async()=>{
  const client=rendererRecipeClient();vi.mocked(client.dashboard).mockRejectedValue({code:'STORAGE_UNAVAILABLE',messageKey:'errors.storage',recoverable:true});
  render(<DashboardPage client={client} onNavigate={vi.fn()}/>);
  expect(await screen.findByRole('alert')).toBeInTheDocument();
  expect(screen.queryByText(/Henüz etkin tarif yok/)).toBeNull();
});

it('does not show stale counts after a failed refresh',async()=>{
  const client=rendererRecipeClient();
  vi.mocked(client.dashboard).mockResolvedValue({activeTotal:1,foodCount:1,beverageCount:0,favoriteCount:0,recentCreated:[],recentModified:[],collections:[]});
  const view=render(<DashboardPage client={client} onNavigate={vi.fn()}/>);
  await screen.findByRole('button',{name:/Etkin tarif\s*1/});
  const failingClient=rendererRecipeClient();
  vi.mocked(failingClient.dashboard).mockRejectedValue({code:'STORAGE_UNAVAILABLE',messageKey:'errors.storage',recoverable:true});
  view.rerender(<DashboardPage client={failingClient} onNavigate={vi.fn()}/>);
  expect(await screen.findByRole('alert')).toBeInTheDocument();
  expect(screen.queryByRole('button',{name:/Etkin tarif\s*1/})).toBeNull();
});
