import { useState } from 'react';
import {
  BookOpen, Settings, Leaf, Menu, CircleCheck, AlertTriangle, LoaderCircle,
  Wine, Carrot, Package, ShoppingBasket, CalendarDays, FolderHeart, ChartNoAxesCombined,
} from 'lucide-react';
import { foundationClient, type FoundationClient } from '../shared/api/client';
import { errorMessage, messages as t } from '../shared/i18n';
import { Button } from '../shared/ui/button';
import { recipeClient, type RecipeClient } from '../shared/api/recipes';
import { LibraryPage } from '../features/recipes/LibraryPage';
import { SettingsPage } from '../features/settings/SettingsPage';
import { useFoundation } from './useFoundation';
import { useTheme } from './theme';

const futureModules = [
  { label: t.dashboard, Icon: ChartNoAxesCombined },
  { label: t.beverage, Icon: Wine },
  { label: t.ingredients, Icon: Carrot },
  { label: t.pantry, Icon: Package },
  { label: t.shopping, Icon: ShoppingBasket },
  { label: t.planner, Icon: CalendarDays },
  { label: t.collections, Icon: FolderHeart },
];

export function AppShell({ client = foundationClient, recipes = recipeClient }: { client?: FoundationClient; recipes?: RecipeClient }) {
  const { data, error, loading, saving, load, saveTheme } = useFoundation(client);
  const [page, setPage] = useState<'library' | 'settings'>('library');
  const [menuOpen, setMenuOpen] = useState(false);
  useTheme(data?.preferences.theme ?? 'system');

  function navigate(next: typeof page) {
    setPage(next);
    setMenuOpen(false);
    document.getElementById('main-content')?.focus();
  }

  return (
    <div className="app-shell">
      <a className="skip-link" href="#main-content">{t.skip}</a>
      <aside id="sidebar" className={`sidebar ${menuOpen ? 'open' : ''}`}>
        <div className="brand">
          <div className="brand-icon"><Leaf size={23} aria-hidden="true" /></div>
          <div><strong>{t.brand}</strong><span>{t.tagline}</span></div>
        </div>
        <nav aria-label={t.navigation}>
          <p className="nav-heading">{t.workspace}</p>
          <button
            className={`nav-item ${page === 'library' ? 'active' : ''}`}
            aria-current={page === 'library' ? 'page' : undefined}
            onClick={() => navigate('library')}
          >
            <BookOpen size={19} aria-hidden="true" />{t.library}
          </button>
          <button
            className={`nav-item ${page === 'settings' ? 'active' : ''}`}
            aria-current={page === 'settings' ? 'page' : undefined}
            onClick={() => navigate('settings')}
          >
            <Settings size={19} aria-hidden="true" />{t.settings}
          </button>
        </nav>
        <section className="future-modules" aria-label={t.future}>
          <p className="nav-heading">{t.future}</p>
          <p className="future-description">{t.upcoming}</p>
          <ul>{futureModules.map(({ label, Icon }) => (
            <li key={label}><Icon size={17} aria-hidden="true" />{label}</li>
          ))}</ul>
        </section>
        <div className="sidebar-footer">
          <span className="badge">{t.milestone}</span><span>{t.foundation}</span>
        </div>
      </aside>
      <div className="workspace">
        <header className="app-header">
          <div className="header-left">
            <Button
              className="menu-button" variant="ghost" size="icon" aria-label={t.menu}
              aria-controls="sidebar" aria-expanded={menuOpen}
              onClick={() => setMenuOpen(!menuOpen)}
            ><Menu size={20} aria-hidden="true" /></Button>
            <span>{page === 'library' ? t.library : t.settings}</span>
          </div>
          <span className="connection-status">
            {error ? <><AlertTriangle size={15} aria-hidden="true" />{t.storageUnavailable}</>
              : data ? <><CircleCheck size={15} aria-hidden="true" />{t.offline}</>
              : <><LoaderCircle size={15} className={loading ? 'spin' : ''} aria-hidden="true" />{t.connecting}</>}
          </span>
        </header>
        <main id="main-content" tabIndex={-1} className="main-content">
          {loading && (
            <section className="state-panel" role="status">
              <LoaderCircle className="spin" size={28} aria-hidden="true" />
              <h1>{t.connecting}</h1><p>{t.loading}</p>
            </section>
          )}
          {!loading && error && (
            <section className="error-panel" role="alert">
              <AlertTriangle size={22} aria-hidden="true" />
              <div>
                <h2>{data ? t.operationError : t.errorTitle}</h2>
                <p>{errorMessage(error.messageKey)}</p>
                {!data && <p>{t.errorBody}</p>}
                <Button variant="outline" onClick={() => void load()}>{t.retry}</Button>
              </div>
            </section>
          )}
          {data && <><div hidden={loading || page !== 'library'}><LibraryPage client={recipes} /></div>{!loading && page === 'settings' && <SettingsPage data={data} saving={saving} onTheme={(theme) => void saveTheme(theme)} />}</>}
        </main>
      </div>
    </div>
  );
}
