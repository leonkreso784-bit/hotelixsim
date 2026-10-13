#!/usr/bin/env node
// ===== Alat `adr` — jedan ADR = jedna datoteka =====
// DECISIONS.md je prerastao u jednu veliku datoteku; svaki ADR sad živi u
// docs/records/DECISIONS/ADR-0NN-<slug>.md, a DECISIONS.md je GENERIRANI indeks (ne uređuje se ručno).
//
//   node scripts/adr.mjs razdvoji            jednokratno: ## ADR-0NN blokovi iz DECISIONS.md -> datoteke
//   node scripts/adr.mjs indeks [--provjeri] generira DECISIONS.md (--provjeri: samo uspoređuje, exit 1 ako je star)
//   node scripts/adr.mjs poveznice [dat…]    DECISIONS.md#adr-0NN -> ravno na datoteku ADR-a (idempotentno)
//
// Korijen repozitorija: argument/zastavica `--korijen=DIR` ili env ADR_KORIJEN (testovi), inače mapa iznad skripte.
// Izvezeno: slug, razdvoji, procitajAdr, indeksAdr, prepisiPoveznice (check:docs ih koristi). Uvoz nema nuspojava.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { prepisiRelativnePoveznice, slug as slugAlat, uOgradi } from "./docs-alati.mjs";
import { jeIznimka } from "./docs-granice.mjs";

/** Slug ADR-a: ASCII, najviše 6 riječi (ista implementacija kao za planove — `docs-alati.mjs`). */
export const slug = (naslov, rijeci = 6) => slugAlat(naslov, rijeci);

const STARI_INDEKS = "docs/records/DECISIONS.md";
const MAPA_ADR = "docs/records/DECISIONS";
const NASLOV_BLOKA = /^## ADR-(\d{3}) — (.+?)\r?$/;
const NASLOV_DATOTEKE = /^# ADR-(\d{3}) — (.+?)\r?$/;
const IME_DATOTEKE = /^ADR-(\d{3})(?:-[a-z0-9-]+)?\.md$/;

const imeAdr = (broj, naslov) => {
  const s = slug(naslov);
  return s ? `ADR-${broj}-${s}.md` : `ADR-${broj}.md`;
};

// Jedan `../` dublje na svaki relativni cilj (blok seli iz records/ u records/DECISIONS/).
const dublje = (tekst) => prepisiRelativnePoveznice(tekst, (cilj) => `../${cilj}`);
const plicje = (tekst) =>
  prepisiRelativnePoveznice(tekst, (cilj) => (cilj.startsWith("../") ? cilj.slice(3) : cilj));

/** Bez praznih redaka i `---` separatora na kraju bloka. */
function obrezi(retci) {
  const r = [...retci];
  const prazan = (x) => x.trim() === "" || x.trim() === "---";
  while (r.length && prazan(r[r.length - 1])) r.pop();
  return r;
}

/**
 * Jednokratno razdvajanje. Vraća broj napisanih datoteka. Svaki blok se provjerava
 * „spoji natrag = izvornik" i ni jedan neprazan redak (osim `---`) ne smije nestati.
 */
export function razdvoji(rootDir) {
  const izvor = fs.readFileSync(path.join(rootDir, STARI_INDEKS), "utf8");
  const retci = izvor.split("\n");
  const ograde = uOgradi(retci);
  const pocetci = [];
  retci.forEach((redak, i) => {
    if (!ograde[i] && NASLOV_BLOKA.test(redak)) pocetci.push(i);
  });
  if (pocetci.length === 0) throw new Error(`${STARI_INDEKS}: nema ADR blokova (## ADR-0NN — …)`);

  const blokovi = pocetci.map((od, k) => {
    const [, broj, naslov] = NASLOV_BLOKA.exec(retci[od]);
    const izvorni = obrezi(retci.slice(od, pocetci[k + 1] ?? retci.length));
    return { broj, naslov, izvorni };
  });
  const brojevi = blokovi.map((b) => b.broj);
  if (new Set(brojevi).size !== brojevi.length) throw new Error("dvostruki broj ADR-a u DECISIONS.md");

  const pisanje = blokovi.map((b) => {
    const tekst = `# ${b.izvorni[0].slice(3)}\n${dublje(b.izvorni.slice(1).join("\n"))}`;
    const natrag = `## ${tekst.split("\n")[0].slice(2)}\n${plicje(tekst.split("\n").slice(1).join("\n"))}`;
    if (natrag !== b.izvorni.join("\n")) {
      throw new Error(`ADR-${b.broj}: spajanje natrag ne daje izvornik (izgubljen ili izmijenjen sadržaj)`);
    }
    return { ime: imeAdr(b.broj, b.naslov), sadrzaj: `${tekst}\n` };
  });

  const neprazni = (r) => r.filter((x) => x.trim() !== "" && x.trim() !== "---").length;
  const prije = neprazni(retci.slice(pocetci[0]));
  const poslije = pisanje.reduce((zbroj, p) => zbroj + neprazni(p.sadrzaj.split("\n")), 0);
  if (prije !== poslije)
    throw new Error(`razdvajanje gubi retke: ${prije} u izvorniku, ${poslije} u datotekama`);

  const mapa = path.join(rootDir, MAPA_ADR);
  fs.mkdirSync(mapa, { recursive: true });
  for (const p of pisanje) fs.writeFileSync(path.join(mapa, p.ime), p.sadrzaj);
  return pisanje.length;
}

/** Status: tekst iza `**Status:**` do prvog `·` ili `(`, bez `**`. */
function izvuciStatus(redak) {
  const m = /\*\*Status:\*\*\s*([^·(]*)/.exec(redak);
  return m ? m[1].replaceAll("**", "").trim() : null;
}

/** Pročita docs/records/DECISIONS/ADR-*.md -> [{ broj, naslov, status, datum, datoteka }], uzlazno po broju. */
export function procitajAdr(rootDir) {
  const mapa = path.join(rootDir, MAPA_ADR);
  if (!fs.existsSync(mapa)) return [];
  return fs
    .readdirSync(mapa)
    .filter((ime) => IME_DATOTEKE.test(ime))
    .sort()
    .map((datoteka) => {
      const retci = fs.readFileSync(path.join(mapa, datoteka), "utf8").split("\n");
      const n = NASLOV_DATOTEKE.exec(retci[0]);
      if (!n) throw new Error(`${MAPA_ADR}/${datoteka}: prvi redak mora biti "# ADR-0NN — naslov"`);
      if (n[1] !== IME_DATOTEKE.exec(datoteka)[1])
        throw new Error(`${datoteka}: broj u imenu i naslovu se razlikuje`);
      const zaglavlje = retci.slice(1, 6);
      const datum = /\*\*Datum:\*\*\s*(\d{4}-\d{2}-\d{2})/.exec(zaglavlje.join("\n"))?.[1];
      const status = zaglavlje.map(izvuciStatus).find((s) => s !== null);
      if (!datum) throw new Error(`${datoteka}: nema "**Datum:** YYYY-MM-DD" u zaglavlju`);
      if (!status) throw new Error(`${datoteka}: nema "**Status:**" u zaglavlju`);
      return { broj: n[1], naslov: n[2], status, datum, datoteka };
    });
}

const celija = (t) => t.replaceAll("|", "\\|");

/** Puni sadržaj DECISIONS.md (generiran). Deterministički, kraj retka `\n`. */
export function indeksAdr(rootDir) {
  const adr = procitajAdr(rootDir).sort((a, b) => b.broj.localeCompare(a.broj));
  const redci = adr.map(
    (a) =>
      `| <a id="adr-${a.broj}"></a>[ADR-${a.broj}](DECISIONS/${a.datoteka}) | ${celija(a.naslov)} | ${celija(a.status)} | ${a.datum} |`,
  );
  return [
    "# Arhitektonske i proizvodne odluke (ADR)",
    "",
    "ADR je jedna zabilježena odluka (kontekst → odluka → posljedice); svaka živi u svojoj datoteci u `docs/records/DECISIONS/`. Ovo je POVIJEST — što vrijedi sad piše u `CLAUDE.md` i u `docs/product`, `docs/architecture`.",
    "Novi ADR: napravi `docs/records/DECISIONS/ADR-0NN-<slug>.md` (naslov `# ADR-0NN — …`, ispod redak `**Datum:** … · **Status:** …`) i pokreni `corepack pnpm docs:adr`.",
    "Ovaj popis je generiran iz tih datoteka — ne uređuj ručno (`check:docs` pada ako nije svjež). Najnoviji je gore.",
    "",
    "| ADR | Naslov | Status | Datum |",
    "|---|---|---|---|",
    ...redci,
    "",
  ].join("\n");
}

const posix = (p) => p.split(path.sep).join("/");
const POCETAK_SIDRA = "\u0000adr-"; // privremena oznaka za `(#adr-0NN)` unutar ADR datoteka

/**
 * Prepisuje `…DECISIONS.md#adr-0NN` (i u ADR datotekama `#adr-0NN`) u relativnu putanju do datoteke ADR-a.
 * `relPath` = putanja datoteke od korijena (posix), `mapaAdr` = Map broj("012") -> ime datoteke.
 * Nepoznat broj se ne dira, a poruka ide u polje `upozorenja` (ako je zadano).
 */
export function prepisiPoveznice(text, relPath, mapaAdr, upozorenja = []) {
  const dir = path.posix.dirname(relPath);
  const uAdrMapi = dir === MAPA_ADR;
  const nadi = (broj) => {
    const datoteka = mapaAdr.get(broj);
    if (!datoteka) upozorenja.push(`${relPath}: ADR-${broj} ne postoji (poveznica ostavljena)`);
    return datoteka;
  };
  const doDatoteke = (datoteka) =>
    path.posix.relative(`/${dir === "." ? "" : dir}`, `/${MAPA_ADR}/${datoteka}`) || datoteka;

  // `(#adr-0NN)` nije "relativan" cilj pa ga prepisivač ne bi ni pozvao: u ADR datotekama ga
  // privremeno pretvaramo u oznaku, a što prepisivač ne dotakne (ograde, inline kod) vraćamo bajt-točno.
  const priprema = uAdrMapi ? text.replace(/\]\(#adr-(\d{3})(?=[\s)])/g, `](${POCETAK_SIDRA}$1`) : text;
  const prepisano = prepisiRelativnePoveznice(priprema, (cilj) => {
    if (cilj.startsWith(POCETAK_SIDRA)) {
      const datoteka = nadi(cilj.slice(POCETAK_SIDRA.length));
      return datoteka ? doDatoteke(datoteka) : `#adr-${cilj.slice(POCETAK_SIDRA.length)}`;
    }
    const m = /^(.*?)DECISIONS\.md#adr-(\d{3})$/.exec(cilj);
    if (!m) return cilj;
    const cijel = path.posix.normalize(path.posix.join(dir, `${m[1]}DECISIONS.md`));
    if (cijel !== STARI_INDEKS) return cilj; // neki drugi DECISIONS.md
    const datoteka = nadi(m[2]);
    return datoteka ? doDatoteke(datoteka) : cilj;
  });
  return uAdrMapi ? prepisano.replaceAll(POCETAK_SIDRA, "#adr-") : prepisano;
}

// ── popis .md datoteka (isto kao check-docs: git, rezerva walk) ─────────
const SKIP_DIRS = new Set(["node_modules", "target", ".git", "dist", ".superpowers"]);

function popisMd(root) {
  // ⚠️ bez GIT_* varijabli: GIT_DIR iz git-kuke inače bi usmjerio upit na krivi repo
  const env = Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith("GIT_")));
  const r = spawnSync("git", ["ls-files", "-co", "--exclude-standard", "-z", "--", "*.md"], {
    cwd: root,
    env,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const popis = !r.error && r.status === 0 ? r.stdout.split("\0").filter(Boolean) : walk(root, root);
  return popis.filter(
    (p) => !p.split("/").some((s) => SKIP_DIRS.has(s)) && fs.existsSync(path.join(root, p)),
  );
}

function walk(root, dir, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP_DIRS.has(e.name)) continue;
    const abs = path.join(dir, e.name);
    if (e.isDirectory()) walk(root, abs, out);
    else if (e.name.endsWith(".md")) out.push(posix(path.relative(root, abs)));
  }
  return out;
}

function main(argv) {
  const zastavice = argv.filter((a) => a.startsWith("--"));
  const argumenti = argv.filter((a) => !a.startsWith("--"));
  const naredba = argumenti.shift();
  const korijenZastavica = zastavice.find((z) => z.startsWith("--korijen="))?.slice(10);
  const root = path.resolve(
    korijenZastavica || process.env.ADR_KORIJEN || path.join(import.meta.dirname, ".."),
  );

  if (naredba === "razdvoji") {
    const n = razdvoji(root);
    console.log(`Razdvojeno: ${n} ADR datoteka u ${MAPA_ADR}/. Sljedeće: indeks, pa poveznice.`);
    return 0;
  }
  if (naredba === "indeks") {
    const novi = indeksAdr(root);
    const putanja = path.join(root, STARI_INDEKS);
    if (zastavice.includes("--provjeri")) {
      const stari = fs.existsSync(putanja) ? fs.readFileSync(putanja, "utf8").replaceAll("\r\n", "\n") : null;
      if (stari === novi) {
        console.log("DECISIONS.md je svjež.");
        return 0;
      }
      console.error("❌ DECISIONS.md nije svjež — pokreni: corepack pnpm docs:adr");
      return 1;
    }
    fs.writeFileSync(putanja, novi);
    console.log(`Indeks napisan: ${procitajAdr(root).length} ADR-ova.`);
    return 0;
  }
  if (naredba === "poveznice") {
    const mapaAdr = new Map(procitajAdr(root).map((a) => [a.broj, a.datoteka]));
    const datoteke = argumenti.length
      ? argumenti.map((a) => posix(path.relative(root, path.resolve(root, a))))
      : popisMd(root).filter((p) => p !== STARI_INDEKS && !jeIznimka(p));
    const upozorenja = [];
    let promijenjeno = 0;
    for (const rel of datoteke) {
      if (rel === STARI_INDEKS || jeIznimka(rel)) continue;
      const abs = path.join(root, rel);
      const stari = fs.readFileSync(abs, "utf8");
      const novi = prepisiPoveznice(stari, rel, mapaAdr, upozorenja);
      if (novi !== stari) {
        fs.writeFileSync(abs, novi);
        promijenjeno++;
        console.log(`prepisano: ${rel}`);
      }
    }
    console.log(`Gotovo: ${promijenjeno} datoteka promijenjeno, ${datoteke.length} pregledano.`);
    for (const u of upozorenja) console.error(`⚠️ ${u}`);
    return upozorenja.length ? 1 : 0;
  }
  console.error(
    "Upotreba: node scripts/adr.mjs razdvoji | indeks [--provjeri] | poveznice [datoteke…]  [--korijen=DIR]",
  );
  return 2;
}

if (process.argv[1] && path.resolve(process.argv[1]) === import.meta.filename) {
  process.exitCode = main(process.argv.slice(2));
}
