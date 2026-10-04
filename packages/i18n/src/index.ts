import { hr } from "./hr";

export type MessageKey = keyof typeof hr;

export type MessageParams = Record<string, string | number>;

export function isMessageKey(key: string): key is MessageKey {
  return Object.hasOwn(hr, key);
}

/** Prijevod poruke; `{ime}` u tekstu zamjenjuje se vrijednošću iz `params`. */
export function t(key: MessageKey, params?: MessageParams): string {
  const message: string = hr[key];
  if (!params) return message;
  return message.replace(/\{(\w+)\}/g, (placeholder, name: string) => {
    const value = params[name];
    return value === undefined ? placeholder : String(value);
  });
}

/** Naziv indeksa za prikaz; nepoznati kod vraća se kakav jest. */
export function indexLabel(code: string): string {
  const key = `index.${code}`;
  return isMessageKey(key) ? t(key) : code;
}
