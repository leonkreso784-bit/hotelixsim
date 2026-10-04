// Jedini put do API-ja iz sučelja. Tipovi su GENERIRANI iz openapi.json; ručno se ne pišu.
import createClient from "openapi-fetch";
import type { components, paths } from "./schema";

export type { components, paths };

export function createApiClient(baseUrl: string) {
  return createClient<paths>({ baseUrl });
}

export type ApiClient = ReturnType<typeof createApiClient>;
