import { isMessageKey } from "@hotelixsim/i18n";
import { describe, expect, it } from "vitest";
import { ApiError, isNoGame, problemMessage, shouldRetry, unwrap } from "./errors";

const res = (status: number) => new Response(null, { status });

describe("unwrap", () => {
  it("vraća podatke", () => {
    expect(unwrap({ data: { a: 1 }, response: res(200) })).toEqual({ a: 1 });
  });
  it("baca ApiError sa statusom i kodom problema", () => {
    try {
      unwrap({ error: { type: "x", title: "x", status: 409, code: "awaiting_reply" }, response: res(409) });
      expect.unreachable();
    } catch (e) {
      expect(e).toBeInstanceOf(ApiError);
      expect((e as ApiError).status).toBe(409);
      expect((e as ApiError).code).toBe("awaiting_reply");
    }
  });
  it("tijelo bez koda daje code null", () => {
    expect(() => unwrap({ error: "Bad Gateway", response: res(502) })).toThrowError(ApiError);
  });
});

describe("shouldRetry", () => {
  it("4xx se ne ponavlja, 5xx i mreža jednom", () => {
    expect(shouldRetry(0, new ApiError(404, null))).toBe(false);
    expect(shouldRetry(0, new ApiError(503, "storage_unavailable"))).toBe(true);
    expect(shouldRetry(1, new ApiError(503, null))).toBe(false);
    expect(shouldRetry(0, new TypeError("Failed to fetch"))).toBe(true);
  });
});

describe("problemMessage i isNoGame", () => {
  it("poznati kod daje hrvatsku rečenicu", () => {
    expect(problemMessage(new ApiError(422, "price_out_of_range"))).toBe(
      "Cijena je izvan dopuštenog raspona.",
    );
  });
  it("nema obračunatog kruga: tjedna igra dobiva inačicu bez općenitog „kruga“", () => {
    const err = new ApiError(409, "no_resolved_round");
    expect(problemMessage(err)).toBe("Račun dobiti postoji tek nakon prvog obračunatog kruga.");
    expect(problemMessage(err, true)).toBe("Račun dobiti postoji tek nakon prvog obračunatog tjedna.");
  });

  it("nepoznat kod: 5xx i mreža → poslužitelj, ostalo → općenito", () => {
    expect(problemMessage(new ApiError(500, "novi_kod"))).toBe(
      "Poslužitelj trenutno ne odgovara. Pokušaj ponovno za koju sekundu.",
    );
    expect(problemMessage(new TypeError("Failed to fetch"))).toBe(
      "Poslužitelj trenutno ne odgovara. Pokušaj ponovno za koju sekundu.",
    );
    expect(problemMessage(new ApiError(400, null))).toBe("Nešto nije u redu. Pokušaj ponovno.");
  });
  it("W-1: 409 situation_mandatory (obvezna se ne prepušta) ima svoju hrvatsku rečenicu", () => {
    expect(isMessageKey("problem.situation_mandatory")).toBe(true);
    const text = problemMessage(new ApiError(409, "situation_mandatory"));
    expect(text).not.toBe("Nešto nije u redu. Pokušaj ponovno.");
    expect(text).not.toMatch(/situation_mandatory/);
  });
  it("401 i 404 na /me/hotel znače: gost još nema igru", () => {
    expect(isNoGame(new ApiError(401, "guest_required"))).toBe(true);
    expect(isNoGame(new ApiError(404, "not_in_game"))).toBe(true);
    expect(isNoGame(new ApiError(500, null))).toBe(false);
  });
});
