// Čuvar L1-13b dio 3 (pouka B4: tekst koji student čita mora odgovarati mehanici): popis kvarova prostora vezanih uz
// stanje Smještaja IZVODI se iz sadržaja (content/hotel/v2/situation_templates.json), ne prepisuje se kao konstanta.
// Jezgra veže predložak uz prostor poljem `condition_of`, a inače poljem `department`; uvjet stanja je
// `dept_condition_pts` u `when` ili u `chance.boost` (crates/content/src/templates_principles.rs, templates_instance.rs).
// Tablica ispod daje SAMO ključnu riječ po šifri predloška; šifra koje nema u tablici ruši test (tekst treba provjeriti).
import { isMessageKey, type MessageKey, t } from "@hotelixsim/i18n";
import { describe, expect, it } from "vitest";
import rawContent from "../../../../../content/hotel/v2/situation_templates.json?raw";
import { ROOMS_ABOUT_KEY } from "./roomsAboutKey";

interface Template {
  code: string;
  kind: string;
  department: string;
  condition_of?: string;
}

// Sirovi tekst sadržaja (Vite `?raw`; web nema Node tipove).
const raw = rawContent;
const templates = (JSON.parse(raw) as { templates: Template[] }).templates;

/** Šifra predloška → ključna riječ u tekstu (kad je kvar vezan uz Smještaj tekst je MORA sadržavati, inače NE SMIJE). */
const KEYWORD: Record<string, RegExp> = {
  "P-KLIMA": /klim/i,
  "P-BOJLER": /bojler/i,
  "P-PRAONICA": /praonic/i,
  "P-DIZALO": /dizal/i,
  "P-BRAVE": /brav/i,
  "P-CURENJE-VODE": /curenj/i,
  "P-BAZEN-PUMPA": /pump|bazen/i,
  "P-LOBBY-KAVA": /kav[aeu]\b|aparat/i,
  "P-INSPEKCIJA": /inspekcij/i,
};

/** Prostor kojem predložak čita stanje; skupine (`*…`) i `hotel` bez `condition_of` nemaju jedan prostor pa se ne provjeravaju. */
function space(tpl: Template): string | null {
  if (tpl.condition_of) return tpl.condition_of;
  return tpl.department.startsWith("*") || tpl.department === "hotel" ? null : tpl.department;
}

function conditionBound(tpl: Template): boolean {
  const start = raw.indexOf(`"code": "${tpl.code}"`);
  const next = raw.indexOf(`"code": "`, start + 1);
  const block = raw.slice(start, next === -1 ? undefined : next);
  return block.includes("dept_condition_pts") || tpl.condition_of !== undefined;
}

const bound = templates.filter((tpl) => space(tpl) !== null && conditionBound(tpl));
const roomsBound = bound.filter((tpl) => space(tpl) === "rooms");
const otherBound = bound.filter((tpl) => space(tpl) !== "rooms");

function text(): string {
  return isMessageKey(ROOMS_ABOUT_KEY) ? t(ROOMS_ABOUT_KEY as MessageKey) : "";
}

describe("Opis Smještaja ↔ sadržaj kvarova", () => {
  it("sadržaj ima kvarove vezane uz Smještaj (čuvar izvođenja nije prazan)", () => {
    expect(roomsBound.length).toBeGreaterThanOrEqual(5);
  });

  it("svaka šifra iz sadržaja ima ključnu riječ u tablici (nova šifra = pregledaj tekst)", () => {
    const missing = bound.map((tpl) => tpl.code).filter((code) => !(code in KEYWORD));
    expect(missing).toEqual([]);
  });

  it("opis spominje svaki kvar prostora vezan uz stanje Smještaja", () => {
    const body = text();
    expect(body).not.toBe("");
    const notMentioned = roomsBound.filter((tpl) => !KEYWORD[tpl.code]?.test(body)).map((tpl) => tpl.code);
    expect(notMentioned).toEqual([]);
  });

  it("opis ne spominje kvar koji nije vezan uz Smještaj", () => {
    const body = text();
    expect(body).not.toBe("");
    const wrong = otherBound.filter((tpl) => KEYWORD[tpl.code]?.test(body)).map((tpl) => tpl.code);
    expect(wrong).toEqual([]);
  });
});
