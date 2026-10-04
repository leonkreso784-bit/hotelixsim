import { hr } from "./hr";

export type MessageKey = keyof typeof hr;

export function t(key: MessageKey): string {
  return hr[key];
}
