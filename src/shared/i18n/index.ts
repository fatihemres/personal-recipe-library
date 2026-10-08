import { tr } from './tr';
// Locale resources are kept outside presentation components. English can be added here.
export const messages = tr;
export function errorMessage(key: string): string {
  return Object.prototype.hasOwnProperty.call(tr.errors, key) ? tr.errors[key as keyof typeof tr.errors] : tr.errors['errors.unknown'];
}
