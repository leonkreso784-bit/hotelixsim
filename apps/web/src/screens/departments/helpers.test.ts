import { describe, expect, it } from "vitest";
import { resultText } from "./helpers";

describe("resultText", () => {
  it("strelica i predznak po prikazanim eurima", () => {
    expect(resultText(789_600)).toBe("▲ +7.896\u00a0€");
    expect(resultText(-80_000)).toBe("▼ −800\u00a0€");
  });
  it("ispod pola eura nema strelice ni „▼ 0\u00a0€“", () => {
    expect(resultText(0)).toBe("0\u00a0€");
    expect(resultText(-40)).toBe("0\u00a0€");
    expect(resultText(40)).toBe("0\u00a0€");
  });
});
