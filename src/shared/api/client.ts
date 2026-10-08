import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { appErrorSchema, bootstrapSchema, preferencesSchema, type AppError, type Bootstrap, type Preferences } from '../contracts/foundation';
export interface FoundationClient {
  bootstrap(): Promise<Bootstrap>;
  savePreferences(preferences: Preferences): Promise<Preferences>;
}
export function normalizeError(error: unknown): AppError {
  const parsed = appErrorSchema.safeParse(error);
  return parsed.success ? parsed.data : { code: 'UNKNOWN', messageKey: 'errors.unknown', recoverable: true };
}
export async function request<T>(command: string, schema: z.ZodType<T>, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw { code: 'DESKTOP_REQUIRED', messageKey: 'errors.desktop', recoverable: false } satisfies AppError;
  const result = await invoke<unknown>(command, args);
  const parsed = schema.safeParse(result);
  if (!parsed.success) throw { code: 'INVALID_RESPONSE', messageKey: 'errors.protocol', recoverable: false } satisfies AppError;
  return parsed.data;
}
export const foundationClient: FoundationClient = {
  bootstrap: () => request('bootstrap', bootstrapSchema),
  savePreferences: (preferences) => {
    const parsed = preferencesSchema.safeParse(preferences);
    if (!parsed.success) return Promise.reject({ code: 'INVALID_INPUT', messageKey: 'errors.validation', recoverable: true } satisfies AppError);
    return request('save_preferences', preferencesSchema, { preferences: parsed.data });
  },
};
