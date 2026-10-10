import { isMessageKey, type MessageKey, t } from "@hotelixsim/i18n";

/** Greška API-ja: HTTP status i stabilan kod problema (`Problem.code`) ako ga tijelo ima. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string | null,
  ) {
    super(`API ${status}${code ? ` ${code}` : ""}`);
    this.name = "ApiError";
  }
}

function problemCode(body: unknown): string | null {
  if (typeof body === "object" && body !== null && "code" in body && typeof body.code === "string")
    return body.code;
  return null;
}

/** Rezultat openapi-fetch poziva → podaci ili ApiError. */
export function unwrap<T>(r: { data?: T; error?: unknown; response: Response }): T {
  if (r.data !== undefined) return r.data;
  throw new ApiError(r.response.status, problemCode(r.error));
}

/** 4xx je odluka poslužitelja i ne ponavlja se; 5xx i mreža jednom. */
export function shouldRetry(failureCount: number, error: unknown): boolean {
  if (error instanceof ApiError && error.status < 500) return false;
  return failureCount < 1;
}

/** Hrvatska rečenica za grešku; za tjednu igru (`weekly`) bira se inačica "<kod>.week" ako postoji. */
export function problemMessage(error: unknown, weekly = false): string {
  if (error instanceof ApiError) {
    const key = `problem.${error.code ?? ""}`;
    if (error.code && weekly && isMessageKey(`${key}.week`)) return t(`${key}.week` as MessageKey);
    if (error.code && isMessageKey(key)) return t(key);
    return error.status >= 500 ? t("error.server") : t("error.generic");
  }
  return t("error.server");
}

export function isNoGame(error: unknown): boolean {
  return error instanceof ApiError && (error.status === 401 || error.status === 404);
}
