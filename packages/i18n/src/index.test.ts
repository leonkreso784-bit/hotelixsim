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

describe("hr: tekst prema studentu bez roda (UI_PRAVILA §3.3)", () => {
  // Muški glagolski pridjev uz „si“ („Potrošio si“, „odlučio si“) i imenice/pridjevi koji biraju rod.
  const gendered = [
    /\p{L}+(?:o|la) si\b/u,
    /\bsi (?:\p{L}+ )?\p{L}+(?:o|la)\b/u,
    /\bupravitelj/iu,
    /\b(?:siguran|sigurna|spreman|spremna|dobrodošao|dobrodošla)\b/iu,
  ];
  it("nijedan tekst ne koristi rodno obilježen oblik", async () => {
    const { hr } = await import("./hr");
    const hits = Object.entries(hr).filter(([, text]) => gendered.some((re) => re.test(text)));
    expect(hits).toEqual([]);
  });
  it("prijašnji rodni tekstovi su neutralni", () => {
    expect(t("hotel.you")).toBe("Ti vodiš ovaj hotel");
    expect(t("chat.limit")).toBe("Poruke za ovaj razgovor su potrošene. Sad odgovori.");
    expect(t("problem.chat_limit")).toBe("Poruke za ovaj razgovor su potrošene.");
  });
});
