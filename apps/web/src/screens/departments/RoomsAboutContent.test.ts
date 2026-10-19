// Čuvar L1-13b dio 3 (pouka B4: tekst koji student čita mora odgovarati mehanici): popis kvarova prostora vezanih uz
// stanje Smještaja IZVODI se iz sadržaja (content/hotel/v2/situation_templates.json), ne prepisuje se kao konstanta.
// Jezgra veže predložak uz prostor poljem `condition_of`, a inače poljem `department`; uvjet stanja je
// `dept_condition_pts` u `when` ili u `chance.boost` (crates/content/src/templates_principles.rs, templates_instance.rs).
// Tablica ispod daje SAMO ključnu riječ po šifri predloška; šifra koje nema u tablici ruši test (tekst treba provjeriti).
import { isMessageKey, type MessageKey, t } from "@hotelixsim/i18n";
import { describe, expect, it } from "vitest";
import departmentsRaw from "../../../../../content/hotel/v2/departments.json?raw";
import rawContent from "../../../../../content/hotel/v2/situation_templates.json?raw";
import { ROOMS_ABOUT_KEY } from "./roomsAboutKey";

interface Template {
  code: string;
  kind: string;
  department: string;
  condition_of?: string;
  when?: { dept_condition_pts?: { below?: number } };
  chance?: { boost?: { when?: { dept_condition_pts?: unknown } }[] };
}

// Sirovi tekst sadržaja (Vite `?raw`; web nema Node tipove).
const templates = (JSON.parse(rawContent) as { templates: Template[] }).templates;

/** Šifra predloška → ključna riječ u tekstu (kad je kvar vezan uz Smještaj tekst je MORA sadržavati, inače NE SMIJE). */
const KEYWORD: Record<string, RegExp> = {
  "P-KLIMA": /klim/i,
  "P-BOJLER": /bojler/i,
  "P-PRAONICA": /praonic/i,
  "P-DIZALO": /dizal/i,
  "P-BRAVE": /brav/i,
  "P-CURENJE-VODE": /curenj/i,
  "P-DOTRAJALO": /dotrajal/i,
  "P-BAZEN-PUMPA": /pump|bazen/i,
  "P-LOBBY-KAVA": /kav[aeu]\b|aparat/i,
  "P-INSPEKCIJA": /inspekcij/i,
};

const catalog = JSON.parse(departmentsRaw) as Record<string, { class: string; wage_cents: number }>;

/**
 * Prostori kojima predložak čita stanje. Skupine se izvode iz `departments.json` istim pravilom kao jezgra
 * (crates/sim-hotel-v2/src/situations/template.rs, `DeptTarget::members`): `*staffed` = plaća > 0,
 * `*guest_facing` = klase rooms, food_beverage, other_operated. Cijeli hotel bez `condition_of` nema prostor.
 */
function spaces(tpl: Template): string[] {
  if (tpl.condition_of) return [tpl.condition_of];
  const entries = Object.entries(catalog);
  if (tpl.department === "*staffed") return entries.filter(([, d]) => d.wage_cents > 0).map(([k]) => k);
  if (tpl.department === "*guest_facing")
    return entries
      .filter(([, d]) => ["rooms", "food_beverage", "other_operated"].includes(d.class))
      .map(([k]) => k);
  if (tpl.department.startsWith("*")) throw new Error(`nepoznata skupina ${tpl.department}: dopuni čuvara`);
  return tpl.department === "hotel" ? [] : [tpl.department];
}

/** Parsiran predložak, ne sirovi tekst: predložak ne može tiho ispasti iz popisa zbog razmaka u JSON-u. */
function conditionBound(tpl: Template): boolean {
  return tpl.condition_of !== undefined || JSON.stringify(tpl).includes('"dept_condition');
}

const bound = templates.filter((tpl) => spaces(tpl).length > 0 && conditionBound(tpl));
const roomsBound = bound.filter((tpl) => spaces(tpl).includes("rooms"));
const otherBound = bound.filter((tpl) => !spaces(tpl).includes("rooms"));

function text(): string {
  return isMessageKey(ROOMS_ABOUT_KEY) ? t(ROOMS_ABOUT_KEY as MessageKey) : "";
}

describe("Opis Smještaja ↔ sadržaj kvarova", () => {
  it("sadržaj ima kvarove vezane uz Smještaj (čuvar izvođenja nije prazan)", () => {
    expect(roomsBound.length).toBeGreaterThanOrEqual(7);
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

  /** Prag (bodovi) iz kojeg jezgra pušta predložak: `when.dept_condition_pts.below` / 100. */
  function threshold(tpl: Template): number {
    const below = tpl.when?.dept_condition_pts?.below;
    if (typeof below !== "number") throw new Error(`${tpl.code}: nema when.dept_condition_pts.below`);
    return below / 100;
  }

  it('pragovi u tekstu ("… ispod N") odgovaraju pragovima kvarova iz sadržaja i obrnuto', () => {
    const body = text();
    expect(body).not.toBe("");
    const found = [...body.matchAll(/ispod (\d+)/g)];
    const byCode = new Map<string, number>();
    for (const m of found) {
      // Odsječak prije „ispod N": od prethodnog „ispod N", točke, dvotočke ili zagrade.
      const before = body.slice(0, m.index);
      const prev = found.filter((f) => (f.index ?? 0) < (m.index ?? 0)).at(-1);
      const start = Math.max(
        prev ? (prev.index ?? 0) + prev[0].length : 0,
        before.lastIndexOf("."),
        before.lastIndexOf(":"),
        before.lastIndexOf("("),
      );
      const segment = body.slice(start, m.index);
      for (const tpl of roomsBound) if (KEYWORD[tpl.code]?.test(segment)) byCode.set(tpl.code, Number(m[1]));
    }
    const wrong = roomsBound
      .filter((tpl) => byCode.get(tpl.code) !== threshold(tpl))
      .map(
        (tpl) =>
          `${tpl.code}: u tekstu ${byCode.get(tpl.code) ?? "nema praga"}, u sadržaju ${threshold(tpl)}`,
      );
    expect(wrong).toEqual([]);
    // Obrnuto: svaki prag spomenut u tekstu postoji u sadržaju kvarova Smještaja.
    const real = new Set(roomsBound.map(threshold));
    expect([...new Set(found.map((m) => Number(m[1])))].filter((n) => !real.has(n))).toEqual([]);
  });

  it("popis „češće pri nižem stanju” u tekstu = kvarovi Smještaja s chance.boost po stanju", () => {
    const body = text();
    const at = body.indexOf("češće");
    expect(at).toBeGreaterThan(-1);
    const before = body.slice(0, at);
    const lastBelow = [...before.matchAll(/ispod \d+/g)].at(-1);
    const start = Math.max(
      before.lastIndexOf(". "),
      lastBelow ? (lastBelow.index ?? 0) + lastBelow[0].length : 0,
    );
    const clause = body.slice(start, at);
    const mentioned = roomsBound.filter((tpl) => KEYWORD[tpl.code]?.test(clause)).map((tpl) => tpl.code);
    const boosted = roomsBound
      .filter((tpl) => tpl.chance?.boost?.some((b) => b.when?.dept_condition_pts !== undefined))
      .map((tpl) => tpl.code);
    expect(boosted.length).toBeGreaterThan(0);
    expect([...mentioned].sort()).toEqual([...boosted].sort());
  });
});
