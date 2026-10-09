import { useEffect, useId, useRef, useState } from 'react';
import type { RecipeClient } from '../../shared/api/recipes';
import type {
  PersonalIngredient,
  IngredientSearch,
} from '../../shared/contracts/recipe';
import { normalizeError } from '../../shared/api/client';
import { errorMessage } from '../../shared/i18n';
import { recipesTr as t } from '../../shared/i18n/recipes';
import { Button } from '../../shared/ui/button';
export function IngredientSearchControl({
  client,
  onSelect,
  onBusy,
  personalOnly = true,
}: {
  client: RecipeClient;
  personalOnly?: boolean;
  onSelect: (
    ingredient: PersonalIngredient | { id: string; name: string },
  ) => void;
  onBusy?: (busy: boolean) => void;
}) {
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState('');
  const [result, setResult] = useState<IngredientSearch>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [creating, setCreating] = useState(false);
  const [active, setActive] = useState(-1);
  const [expanded, setExpanded] = useState(true);
  const [retry, setRetry] = useState(0);
  const generation = useRef(0);
  const creatingRef = useRef(false);
  useEffect(() => {
    let alive = true;
    const current = ++generation.current;
    const timer = setTimeout(() => {
      setLoading(true);
      setError('');
      void (personalOnly ? client.searchPersonal : client.searchAvailable ?? client.searchPersonal)(query).then(
        (value) => {
          if (alive && generation.current === current) {
            setResult(value);
            setActive(-1);
            setLoading(false);
          }
        },
        (e) => {
          if (alive) {
            setError(errorMessage(normalizeError(e).messageKey));
            setLoading(false);
          }
        },
      );
    }, 150);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [client, query, retry, personalOnly]);
  async function create() {
    if (creatingRef.current || !query.trim()) return;
    creatingRef.current = true;
    setCreating(true);
    onBusy?.(true);
    try {
      const ingredient = await client.createIngredient(query);
      onSelect(ingredient);
      setRetry((n) => n + 1);
      setError('');
    } catch (e) {
      setError(errorMessage(normalizeError(e).messageKey));
    } finally {
      creatingRef.current = false;
      setCreating(false);
      onBusy?.(false);
    }
  }
  const items = loading ? [] : (result?.items ?? []);
  return (
    <div className="ingredient-search">
      <label htmlFor={`${id}-input`}>{t.searchIngredient}</label>
      <input
        ref={input}
        id={`${id}-input`}
        maxLength={200}
        value={query}
        role="combobox"
        aria-autocomplete="list"
        aria-expanded={expanded && items.length > 0}
        aria-controls={`${id}-results`}
        aria-activedescendant={
          expanded && active >= 0 ? `${id}-${active}` : undefined
        }
        onChange={(e) => {
          setQuery(e.target.value);
          setLoading(true);
          setExpanded(true);
          setActive(-1);
        }}
        onFocus={() => setExpanded(true)}
        onKeyDown={(e) => {
          if (e.key === 'Escape') {
            setExpanded(false);
            setActive(-1);
          }
          if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
            e.preventDefault();
            setExpanded(true);
            setActive((index) =>
              items.length
                ? (index + (e.key === 'ArrowDown' ? 1 : -1) + items.length) %
                  items.length
                : -1,
            );
          }
          if (e.key === 'Enter') {
            e.preventDefault();
            if (expanded && items.length) {
              onSelect(items[active < 0 ? 0 : active]);
              setExpanded(false);
              setActive(-1);
            }
          }
        }}
      />
      <p className="muted">
        {personalOnly ? t.ingredientHelp : t.availableIngredientHelp} {t.ingredientKeyboard}
      </p>
      {loading && <p role="status">{t.ingredientSearching}</p>}
      {error && (
        <div role="alert">
          <p>
            {t.ingredientSearchFailed} {error}
          </p>
          <Button
            type="button"
            variant="outline"
            onClick={() => setRetry((n) => n + 1)}
          >
            {t.retry}
          </Button>
        </div>
      )}
      {!loading && !error && result && items.length === 0 && (
        <p role="status">
          {result.total === 0 ? personalOnly ? t.emptyCatalog : t.emptyAvailableCatalog : t.emptyIngredientSearch}
        </p>
      )}
      <ul
        id={`${id}-results`}
        role="listbox"
        aria-label={t.chooseIngredient}
        className="ingredient-results"
        hidden={!expanded}
      >
        {items.map((ingredient, index) => (
          <li key={ingredient.id} role="presentation">
            <Button
              id={`${id}-${index}`}
              type="button"
              role="option"
              aria-selected={active === index}
              variant="outline"
              onClick={() => {
                onSelect(ingredient);
                input.current?.focus();
                setExpanded(false);
                setActive(-1);
              }}
            >
              {ingredient.name}
            </Button>
          </li>
        ))}
      </ul>
      {result?.hasMore && <p>{t.ingredientMore}</p>}
      <Button
        type="button"
        variant="outline"
        disabled={creating || !query.trim()}
        onClick={() => void create()}
      >
        {t.createIngredient}
      </Button>
    </div>
  );
}
