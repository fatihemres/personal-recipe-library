import { mediaErrors } from './media';
import { tr } from './tr';
// Locale resources are kept outside presentation components. English can be added here.
export const messages = tr;
export function errorMessage(key: string): string {
  if (Object.prototype.hasOwnProperty.call(mediaErrors,key)) return mediaErrors[key as keyof typeof mediaErrors];
  return Object.prototype.hasOwnProperty.call(tr.errors, key) ? tr.errors[key as keyof typeof tr.errors] : tr.errors['errors.unknown'];
}
