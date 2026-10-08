import { ArrowRight, BookOpen, LockKeyhole } from 'lucide-react';
import { messages as t } from '../../shared/i18n';
import { Button } from '../../shared/ui/button';
export function LibraryPage({ openSettings }: { openSettings: () => void }) {
  return <>
    <section className="page-heading"><p className="eyebrow">{t.libraryEyebrow}</p><h1>{t.library}</h1><p>{t.libraryIntro}</p></section>
    <section className="empty-state panel">
      <div className="book-mark"><BookOpen size={38} strokeWidth={1.4} aria-hidden="true" /></div>
      <span className="badge">{t.foundation}</span><h2>{t.emptyTitle}</h2><p>{t.emptyBody}</p>
      <Button onClick={openSettings}>{t.settingsLink}<ArrowRight size={16} aria-hidden="true" /></Button>
      <div className="next-step">{t.next}</div>
    </section>
    <section className="privacy-note"><LockKeyhole size={22} aria-hidden="true" /><div><h2>{t.privacyTitle}</h2><p>{t.privacyBody}</p></div></section>
  </>;
}
