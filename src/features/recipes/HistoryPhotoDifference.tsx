import { stepPhotosTr } from '../../shared/i18n/stepPhotos';
import { z } from 'zod';
import type { RecipeClient } from '../../shared/api/recipes';
import { historyTr as t } from '../../shared/i18n/recipeHistory';
import { CoverImage } from './RecipeMedia';

export function HistoryPhotoDifference({ client, before, after }: {
  client: RecipeClient; before: unknown; after: unknown;
}) {
  return <div className="history-photo-difference">
    {[{ label: t.before, value: before }, { label: t.after, value: after }].map(({ label, value }) => {
      const id = z.uuid().safeParse(value);
      return <div key={label}><p>{label}</p>{id.success
        ? <CoverImage client={client} assetId={id.data} variant="thumb" alt={label} unavailableText={stepPhotosTr.unavailable} />
        : <p>{t.missing}</p>}</div>;
    })}
  </div>;
}
