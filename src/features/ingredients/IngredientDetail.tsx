import type { LibraryDetail } from '../../shared/contracts/ingredientLibrary';
import { ingredientsTr as t } from '../../shared/i18n/ingredients';
import { recipesTr } from '../../shared/i18n/recipes';
export function IngredientDetail({ detail }: {
    detail: LibraryDetail;
}) {
    const { item } = detail;
    return <div className="ingredient-detail panel">
    <span className="ingredient-badge">{item.origin === 'catalog' ? t.catalog : t.personal}</span>
    <h2>{item.name}</h2>
    {item.englishName && <p className="ingredient-secondary">{item.englishName}</p>}
    {item.origin === 'catalog' && <p>{t.builtinHelp}</p>}
    {!item.recipeId && <p role="status">{t.unavailable}</p>}
    <h3>{t.names}</h3>
    <dl><dt>{t.canonical}</dt><dd>{detail.canonicalTr ?? item.name}</dd><dt>{t.english}</dt><dd>{item.englishName ?? t.unknown}</dd></dl>
    <h3>{t.aliases}</h3><p>{detail.aliases.length ? detail.aliases.map(a => a.name).join(' · ') : t.noAliases}</p>
    <h3>{t.categories}</h3><p>{item.categories.length ? item.categories.map(c => c.tr).join(' · ') : t.unknownCategory}</p>
    <h3>{t.measurements}</h3><dl><dt>{t.unit}</dt><dd>{item.preferredUnit ?? t.unknown}</dd><dt>{t.dimensions}</dt><dd>{detail.dimensions.map(d => ({ mass: t.mass, volume: t.volume, count: t.count })[d as 'mass' | 'volume' | 'count'] ?? d).join(' · ') || t.unknown}</dd></dl>
    <h3>{recipesTr.notes}</h3><p className="preserve-lines">{item.notes ?? t.noNotes}</p>
    <h3>{t.metadata}</h3>{detail.metadata.length ? <ul>{detail.metadata.map((m, index) => <li key={index}>{m.kind} · {m.code}: {m.value} {m.unit} ({m.basis})</li>)}</ul> : <p>{t.unknownMetadata}</p>}
    <h3>{t.sources}</h3>{detail.catalogVersion !== null && <p>{t.sourceHelp}</p>}
    {!detail.sources.length && <p>{t.personalSource}</p>}
    {detail.sources.map(s => <details key={`${s.sourceId}:${s.version}:${s.externalId}`} className="ingredient-source"><summary>{s.name}</summary><dl><dt>{t.sourceId}</dt><dd>{s.sourceId}</dd><dt>{t.version}</dt><dd>{s.version}</dd><dt>{t.externalId}</dt><dd>{s.externalId}</dd><dt>{t.license}</dt><dd>{s.license}</dd></dl><p>{s.url}</p><p>{s.attribution}</p><details><summary>{t.provenance}</summary><p className="ingredient-source-text">{s.description}</p></details></details>)}
  </div>;
}
