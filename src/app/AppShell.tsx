import { useEffect, useRef, useState } from 'react';
import {
  BookOpen, Settings, Carrot, Leaf, Menu, CircleCheck, AlertTriangle, LoaderCircle,
  Wine, Package, ShoppingBasket, CalendarDays, FolderHeart, ChartNoAxesCombined,
} from 'lucide-react';
import { foundationClient, type FoundationClient } from '../shared/api/client';
import { errorMessage, messages as t } from '../shared/i18n';
import { Button } from '../shared/ui/button';
import { recipeClient, type RecipeClient } from '../shared/api/recipes';
import { LibraryPage } from '../features/recipes/LibraryPage';
import { DashboardPage } from '../features/recipes/DashboardPage';
import { AboutPanel, SettingsPage } from '../features/settings/SettingsPage';
import { useFoundation } from './useFoundation';
import { installNativeExitHandler } from './nativeExit';
import { IngredientLibrary } from '../features/ingredients';
import { useTheme } from './theme';
import { useAppVersion } from './useAppVersion';

const futureModules = [
  { label: t.beverage, Icon: Wine },
  { label: t.pantry, Icon: Package },
  { label: t.shopping, Icon: ShoppingBasket },
  { label: t.planner, Icon: CalendarDays },
];

export function AppShell({ client = foundationClient, recipes = recipeClient }: { client?: FoundationClient; recipes?: RecipeClient }) {
  const { data, error, loading, saving, load, saveTheme } = useFoundation(client);
  const appVersion = useAppVersion();
  const [page, setPage] = useState<'dashboard' | 'library' | 'collections' | 'settings' | 'ingredients'>('library');
  const [navigationRequest,setNavigationRequest]=useState<{sequence:number;kind?:'food'|'beverage';favoriteOnly?:boolean;collectionId?:string;recipeId?:string}>();
  const [menuOpen, setMenuOpen] = useState(false);
  const [exitError, setExitError] = useState('');
  const ingredientLeaveGuard = useRef<(()=>Promise<boolean>)|null>(null);
  const recipeLeaveGuard = useRef<(()=>Promise<boolean>)|null>(null);
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    void installNativeExitHandler(() => setExitError(t.closeFailed)).then(
      (cleanup) => {
        if (active) unlisten = cleanup;
        else cleanup();
      },
      () => { if (active) setExitError(t.closeFailed); },
    );
    return () => { active = false; unlisten?.(); };
  }, []);
  useTheme(data?.preferences.theme ?? 'system');

  async function navigate(next: typeof page) {
    if (page === 'ingredients' && next !== page && ingredientLeaveGuard.current && !await ingredientLeaveGuard.current()) return;
    if ((page === 'library' || page === 'collections') && next !== page && recipeLeaveGuard.current && !await recipeLeaveGuard.current()) return;
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
          <button className={`nav-item ${page === 'dashboard' ? 'active' : ''}`} aria-current={page === 'dashboard' ? 'page' : undefined} onClick={() => void navigate('dashboard')}>
            <ChartNoAxesCombined size={19} aria-hidden="true" />{t.dashboard}
          </button>
          <button
            className={`nav-item ${page === 'library' ? 'active' : ''}`}
            aria-current={page === 'library' ? 'page' : undefined}
            onClick={() => void navigate('library')}
          >
            <BookOpen size={19} aria-hidden="true" />{t.library}
          </button>
          <button
            className={`nav-item ${page === 'settings' ? 'active' : ''}`}
            aria-current={page === 'settings' ? 'page' : undefined}
            onClick={() => void navigate('settings')}
          >
            <Settings size={19} aria-hidden="true" />{t.settings}
          </button>
          <button
            className={`nav-item ${page === 'ingredients' ? 'active' : ''}`}
            aria-current={page === 'ingredients' ? 'page' : undefined}
            onClick={() => void navigate('ingredients')}
          >
            <Carrot size={19} aria-hidden="true" />{t.ingredients}
          </button>
          <button className={`nav-item ${page === 'collections' ? 'active' : ''}`}
            aria-current={page === 'collections' ? 'page' : undefined}
            onClick={() => void navigate('collections')}>
            <FolderHeart size={19} aria-hidden="true" />{t.collections}
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
          <span>{t.brand} · {appVersion ? `v${appVersion}` : t.versionUnavailable}</span>
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
            <span>{page === 'dashboard' ? t.dashboard : page === 'library' ? t.library : page === 'collections' ? t.collections : page === 'ingredients' ? t.ingredients : t.settings}</span>
          </div>
          <span className="connection-status">
            {error ? <><AlertTriangle size={15} aria-hidden="true" />{t.storageUnavailable}</>
              : data ? <><CircleCheck size={15} aria-hidden="true" />{t.offline}</>
              : <><LoaderCircle size={15} className={loading ? 'spin' : ''} aria-hidden="true" />{t.connecting}</>}
          </span>
        </header>
        <main id="main-content" tabIndex={-1} className="main-content">
          {exitError && <p role="alert" className="recipe-error">{exitError}</p>}
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
                <p>{t.errorCode}: <code>{/^[A-Z][A-Z0-9_]{0,63}$/.test(error.code) ? error.code : 'UNKNOWN'}</code></p>
                {!data && <p>{t.errorBody}</p>}
                <Button variant="outline" onClick={() => void load()}>{t.retry}</Button>
              </div>
            </section>
          )}
          {!loading && !data && page === 'settings' && <AboutPanel appVersion={appVersion} />}
          {data && <><div hidden={loading || (page !== 'library' && page !== 'collections')}><LibraryPage client={recipes} section={page === 'collections' ? 'collections' : 'library'} navigationRequest={navigationRequest} registerLeaveGuard={guard=>{recipeLeaveGuard.current=guard;}} /></div>{!loading && page === 'dashboard' && <DashboardPage client={recipes} onNavigate={destination=>{setNavigationRequest({sequence:Date.now(),...destination});setPage('library');}} />}{!loading && page === 'ingredients' && <IngredientLibrary client={recipes} registerLeaveGuard={guard => { ingredientLeaveGuard.current = guard; }} />}{!loading && page === 'settings' && <SettingsPage data={data} appVersion={appVersion} saving={saving} onTheme={(theme) => void saveTheme(theme)} />}</>}
        </main>
      </div>
    </div>
  );
}
