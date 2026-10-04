import { describe, expect, it } from "vitest";
import { indexLabel, isMessageKey, t } from "./index";

describe("t", () => {
  it("vraća tekst bez parametara", () => {
    expect(t("status.db.ok")).toBe("Baza: spojena");
  });

  it("zamjenjuje {ime} vrijednostima, i brojevima", () => {
    expect(t("status.events", { count: 0 })).toBe("Događaja u dnevniku: 0");
    expect(t("profile.heading", { profile: "C" })).toBe("Početni indeksi — profil C");
  });
});

describe("isMessageKey", () => {
  it("razlikuje postojeće i nepostojeće ključeve", () => {
    expect(isMessageKey("index.G")).toBe(true);
    expect(isMessageKey("index.X")).toBe(false);
    expect(isMessageKey("toString")).toBe(false);
  });
});

describe("indexLabel", () => {
  it("prevodi poznati kod", () => {
    expect(indexLabel("G")).toBe("Zadovoljstvo gostiju");
  });

  it("nepoznati kod vraća kakav jest", () => {
    expect(indexLabel("X")).toBe("X");
  });
});
