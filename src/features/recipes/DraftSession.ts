import { normalizeError } from '../../shared/api/client';
import type { RecipeClient } from '../../shared/api/recipes';
import type { Draft, Recipe, RecipeInput } from '../../shared/contracts/recipe';
export class DraftSession {
  readonly id: string;
  private revision: number | null;
  private snapshot: string | null;
  private tail: Promise<unknown> = Promise.resolve();
  private closed = false;
  constructor(
    private client: RecipeClient,
    recovered?: Draft,
  ) {
    this.id = recovered?.id ?? crypto.randomUUID();
    this.revision = recovered?.revision ?? null;
    this.snapshot = recovered ? JSON.stringify(recovered.input) : null;
  }
  private queue<T>(job: () => Promise<T>): Promise<T> {
    const result = this.tail.catch(() => undefined).then(job);
    this.tail = result;
    return result;
  }
  private async persist(input: RecipeInput) {
    if (this.closed)
      throw {
        code: 'DRAFT_CONFLICT',
        messageKey: 'errors.draftConflict',
        recoverable: true,
      };
    const snapshot = JSON.stringify(input);
    if (snapshot === this.snapshot) return;
    const saved = await this.client.saveDraft(this.id, this.revision, input);
    this.revision = saved.revision;
    this.snapshot = snapshot;
  }
  matches(input: RecipeInput) {
    return !this.closed && this.snapshot === JSON.stringify(input);
  }
  flush(input: RecipeInput) {
    const captured = structuredClone(input);
    return this.queue(() => this.persist(captured));
  }
  commit(input: RecipeInput, asCopy = false): Promise<Recipe> {
    const captured = structuredClone(input);
    return this.queue(async () => {
      try {
        await this.persist(captured);
      } catch (error) {
        if (!asCopy || normalizeError(error).code !== 'DRAFT_CONFLICT')
          throw error;
        const fork = {
          ...captured,
          id: crypto.randomUUID(),
          expectedRevision: null,
          ingredients: captured.ingredients.map((line) => ({
            ...line,
            id: crypto.randomUUID(),
          })),
          steps: captured.steps.map((step) => ({
            ...step,
            id: crypto.randomUUID(),
          })),
        };
        return new DraftSession(this.client).commit(fork);
      }
      let result;
      try {
        result = await this.client.commit(
          captured,
          this.id,
          this.revision!,
          asCopy,
        );
      } catch (error) {
        if (!asCopy || normalizeError(error).code !== 'DRAFT_CONFLICT')
          throw error;
        const fork = {
          ...captured,
          id: crypto.randomUUID(),
          expectedRevision: null,
          ingredients: captured.ingredients.map((line) => ({
            ...line,
            id: crypto.randomUUID(),
          })),
          steps: captured.steps.map((step) => ({
            ...step,
            id: crypto.randomUUID(),
          })),
        };
        return new DraftSession(this.client).commit(fork);
      }
      this.closed = true;
      return result;
    });
  }
  discard() {
    return this.queue(async () => {
      if (this.closed) return;
      if (this.revision !== null)
        await this.client.discardDraft(this.id, this.revision);
      this.closed = true;
    });
  }
}
