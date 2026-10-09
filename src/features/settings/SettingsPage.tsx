import { CatalogStatusPanel } from './CatalogStatusPanel';
import { Check, Monitor, Moon, Sun, Database, Languages } from 'lucide-react';
import type { Bootstrap, Theme } from '../../shared/contracts/foundation';
import { messages as t } from '../../shared/i18n';

const choices = [
  { theme: 'light', Icon: Sun, label: t.light, description: t.lightBody },
  { theme: 'dark', Icon: Moon, label: t.dark, description: t.darkBody },
  { theme: 'system', Icon: Monitor, label: t.system, description: t.systemBody },
] as const;

export function SettingsPage({ data, saving, onTheme }: {
  data: Bootstrap; saving: boolean; onTheme: (theme: Theme) => void;
}) {
  return (
    <>
      <section className="page-heading">
        <p className="eyebrow">{t.workspace}</p><h1>{t.settings}</h1><p>{t.settingsIntro}</p>
      </section>
      <section className="panel settings-panel">
        <h2>{t.appearance}</h2><p className="muted">{t.appearanceBody}</p>
        <fieldset disabled={saving}>
          <legend>{t.themeLabel}</legend>
          <div className="theme-options">{choices.map(({ theme, Icon, label, description }) => (
            <label key={theme} className={`theme-option ${data.preferences.theme === theme ? 'selected' : ''}`}>
              <input
                type="radio" name="theme" value={theme}
                checked={data.preferences.theme === theme} onChange={() => onTheme(theme)}
              />
              <Icon aria-hidden="true" size={24} /><strong>{label}</strong><span>{description}</span>
            </label>
          ))}</div>
        </fieldset>
        <p className="save-status" role="status">
          <Check size={14} aria-hidden="true" />{saving ? t.saving : t.saved}
        </p>
      </section>
      <section className="panel settings-panel language-panel">
        <Languages size={22} aria-hidden="true" />
        <div><h2>{t.language}</h2><p className="muted">{t.languageBody}</p></div>
        <span className="badge">{t.turkish}</span>
      </section>
      <section className="panel settings-panel">
        <div className="section-title"><Database size={22} aria-hidden="true" /><h2>{t.storageTitle}</h2></div>
        <p className="muted">{t.storageBody}</p>
        <dl className="storage-grid">
          <div><dt>{t.schema}</dt><dd>{data.storage.schemaVersion}</dd></div>
          <div><dt>{t.sqlite}</dt><dd>{data.storage.sqliteVersion}</dd></div>
          <div><dt>{t.foreignKeys}</dt><dd>{data.storage.foreignKeys ? t.verified : t.notVerified}</dd></div>
          <div><dt>{t.fts}</dt><dd>{data.storage.fts5 ? t.verified : t.notVerified}</dd></div>
        </dl>
      </section>
      <CatalogStatusPanel />
    </>
  );
}
