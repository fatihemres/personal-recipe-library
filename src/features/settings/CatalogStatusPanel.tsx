import { useEffect, useState } from 'react';
import { z } from 'zod';
import { catalogClient, catalogStatusSchema } from '../../shared/api/catalog';
import { normalizeError } from '../../shared/api/client';
import { messages as t, errorMessage } from '../../shared/i18n';
export function CatalogStatusPanel() {
  const [status, setStatus] = useState<z.infer<typeof catalogStatusSchema> | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    catalogClient.status().then(value => { if (active) setStatus(value); }).catch(reason => { if (active) setError(errorMessage(normalizeError(reason).messageKey)); });
    return () => { active = false; };
  }, []);
  return <section className="panel settings-panel"><h2>{t.catalogTitle}</h2><p className="muted">{t.catalogBody}</p>
    {error ? <p role="alert">{error}</p> : !status ? <p role="status">{t.catalogLoading}</p> : <dl className="storage-grid">
      <div><dt>{t.catalogValidation}</dt><dd>{status.validationDefinitions}</dd></div>
      <div><dt>{t.catalogProduction}</dt><dd>{status.productionDefinitions}</dd></div>
      <div><dt>{t.catalogCollisions}</dt><dd>{status.pendingCollisions}</dd></div>
    </dl>}
  </section>;
}
