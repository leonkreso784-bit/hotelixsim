#!/usr/bin/env node
// ===== check:docs — integritet dokumentacije =====
// Postoji jer dokumentacija bez brane trune tiho: poveznice umru, dokumenti ostanu bez
// indeksa, "aktivnih" planova postane pet, a CLAUDE.md (učitava se SVAKU sesiju) naraste.
// Raspored ima jedan indeks (docs/README.md), jedan aktivni plan i definicije bez dnevnika;
// ova brana čuva da tako i ostane. Uzor: Sokratov scripts/check-docs.js.
//
// Pada (exit 1) ako:
//    1) relativna poveznica (ili slika) ne vodi na postojeću datoteku/mapu  → mrtve veze
//    2) .md pod docs/ nije spomenut u svom indeksu: docs/<uloga>/*.md u docs/README.md,
//       docs/<uloga>/X/*.md u docs/<uloga>/X.md (koji mora postojati i biti u docs/README.md);
//       dublja razina (X/Y/…) pada                                           → dokument-duh, jedna razina
//    3) docs/plan/*.md ima više od JEDNOG aktivnog plana po **Područje:**
//       (kod | dokumentacija; bez retka = kod; drugo pada)                   → „koji plan vrijedi?"
//    4) docs/product/** ima datoteku s više od 3 ISO datuma                  → definicija ≠ dnevnik
//    5) CLAUDE.md je preko budžeta                                           → trošak svake sesije
//    6) ćirilica u .md izvan koda                                            → kopirano slovo-dvojnik
//    7) docs/plan/*.md i docs/plan/*/*.md proglašava plan iz archive/ AKTIVNIM → arhiva laže da je živa
//    8) podindeks: X.md uz mapu X/ mora postojati i biti naveden; dublja razina pada → provodi se u bloku 2
//    9) .md pod docs/ preko granice redaka (scripts/docs-granice.mjs; prijelazne iznimke izuzete) → veličina
//   10) ADR: ako postoji docs/records/DECISIONS/ — imena ADR-NNN-slug.md, jedinstveni brojevi i
//       DECISIONS.md jednak generiranom indeksu (scripts/adr.mjs); DECISIONS.md bez mape pada → ADR po datoteci
//   11) poveznica koja cilja DECISIONS.md#adr-… (osim u prijelaznim iznimkama)  → stari način navođenja ADR-a
//   12) BILO KOJI .md u repozitoriju preko 1200 redaka (TVRDA_GRANICA; bez prijelaznih iznimaka,
//       izuzeti samo IZUZECI_TVRDE_GRANICE)                                  → nijedan dokument preko 1200
//   (brojevi prate plan dokumentacije)
//
// Pokreni: pnpm check:docs   |   korijen se može zadati: node scripts/check-docs.mjs <dir>
// ili env CHECK_DOCS_ROOT (tako ga testira check-docs.test.mjs nad privremenim mapama).
// `--popis` (uz ili bez korijena) ne provjerava nego ispisuje veličine svih docs/**/*.md
// uz granicu (scripts/docs-granice.mjs), silazno; ❌ uz prekoračenje; uvijek exit 0.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { indeksAdr } from "./adr.mjs";
import { prepisiRelativnePoveznice } from "./docs-alati.mjs";
import { brojRedaka, granicaZa, jeIznimka, jeIzuzetOdTvrde, TVRDA_GRANICA } from "./docs-granice.mjs";

const POPIS = process.argv.includes("--popis");
const ARGUMENTI = process.argv.slice(2).filter((a) => !a.startsWith("--"));

const ROOT = path.resolve(
  ARGUMENTI[0] || process.env.CHECK_DOCS_ROOT || path.join(import.meta.dirname, ".."),
);
const rel = (p) => path.relative(ROOT, p).split(path.sep).join("/");

// Što nikad ne ulazi u provjeru (vanjski/generirani sadržaj, ne naša dokumentacija).
const SKIP_DIRS = new Set(["node_modules", "target", ".git", "dist", ".superpowers"]);
const skipped = (r) => r.split("/").some((s) => SKIP_DIRS.has(s)) || r.startsWith(".claude/worktrees/");

// ── popis .md datoteka ──────────────────────────────────────────────
// Primarno `git ls-files -co --exclude-standard`: ulaze i NOVE necommitane datoteke (dok se
// dokumentacija preslaguje, većina je upravo takva), a gitignorirano ne ulazi. Brana koja
// ovisi o tome što je stroj usput generirao ne mjeri repozitorij.
// Rezerva je običan walk (git nedostupan, ili korijen nije repo — npr. privremena mapa u testu).
// ⚠️ `-c` uključuje i datoteke koje su obrisane/premještene u radnoj kopiji a još su u indeksu
// (usred `git mv`-a) — zato se svaka mora provjeriti na disku.
// ⚠️ spawn bez GIT_* varijabli: GIT_DIR iz git-kuke inače bi usmjerio upit na krivi repo.
function listViaGit() {
  const env = Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith("GIT_")));
  const r = spawnSync("git", ["ls-files", "-co", "--exclude-standard", "-z", "--", "*.md"], {
    cwd: ROOT,
    env,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (r.error || r.status !== 0) return null;
  return r.stdout.split("\0").filter(Boolean);
}

function walk(dir, out) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      if (!SKIP_DIRS.has(e.name)) walk(p, out);
    } else if (e.name.endsWith(".md")) out.push(rel(p));
  }
  return out;
}

const listed = listViaGit() ?? walk(ROOT, []);
const mdFiles = [...new Set(listed)]
  .filter((r) => r.endsWith(".md") && !skipped(r) && fs.existsSync(path.join(ROOT, r)))
  .sort();
const read = (r) => fs.readFileSync(path.join(ROOT, r), "utf8");
// Kod (blokovi i inline) je ilustracija, ne tvrdnja: primjer poveznice ili ćirilični raspon u njemu nije kvar.
const stripCode = (s) => s.replace(/```[\s\S]*?```/g, "").replace(/`[^`\n]*`/g, "");

// Jedno mjesto za „je li X.md indeks uz mapu X/" — koriste ga --popis i pravilo 9.
const granicaDatoteke = (r) => {
  const mapa = path.join(ROOT, r.slice(0, -3));
  const imaMapu = fs.existsSync(mapa) && fs.statSync(mapa).isDirectory();
  return granicaZa(r, imaMapu);
};

// ── --popis: ispis veličina umjesto provjere ────────────────────────
if (POPIS) {
  const redovi = mdFiles
    .filter((r) => r.startsWith("docs/"))
    .map((r) => ({ r, redaka: brojRedaka(read(r)), g: granicaDatoteke(r) }))
    .sort((a, b) => b.redaka - a.redaka || a.r.localeCompare(b.r));
  console.log("redaka  granica  vrsta    datoteka");
  let preko = 0;
  for (const { r, redaka, g } of redovi) {
    const prekoracenje = g && redaka > g.redaka;
    if (prekoracenje) preko++;
    const oznaka = prekoracenje ? (jeIznimka(r) ? " ❌ (prijelazna iznimka)" : " ❌") : "";
    console.log(
      `${String(redaka).padStart(6)}  ${String(g?.redaka ?? "-").padStart(7)}  ${(g?.vrsta ?? "-").padEnd(7)}  ${r}${oznaka}`,
    );
  }
  console.log(`\n${redovi.length} datoteka, ${preko} preko granice`);
  process.exit(0);
}

const problems = [];

// ── 1) mrtve poveznice ──────────────────────────────────────────────
// Svaki relativni cilj `[tekst](cilj)` / `![alt](cilj)` izvan koda mora postojati na disku kao
// datoteka ili mapa — bilo kojeg tipa (.md, .png, .mjs …). Sidro (#…) i upit (?…) se ne provjeravaju,
// samo cilj. Postojanje se pita disk (ne popis). Skener je zajednički (scripts/docs-alati.mjs).
// Isti ciljevi hrane pravilo 11.
let links = 0;
const ciljeviPoDatoteci = new Map();
for (const r of mdFiles) {
  const dir = path.posix.dirname(r);
  const ciljevi = [];
  prepisiRelativnePoveznice(read(r), (cilj) => {
    ciljevi.push(cilj);
    return cilj;
  });
  ciljeviPoDatoteci.set(r, ciljevi);
  for (const link of ciljevi) {
    links++;
    let target = link.replace(/[#?].*$/s, "");
    try {
      target = decodeURIComponent(target);
    } catch {}
    if (!fs.existsSync(path.join(ROOT, path.posix.join(dir, target)))) {
      problems.push(`MRTVA POVEZNICA  ${r}  ->  ${link}`);
    }
  }
}

// ── 2) svaki .md pod docs/ mora biti u svom indeksu ─────────────────
// JEDINI ulaz je docs/README.md. `docs/<uloga>/*.md` mora biti u njemu; `docs/<uloga>/X/*.md` je dio
// pa mora biti u podindeksu `docs/<uloga>/X.md` (koji sam mora postojati i biti u docs/README.md).
// Dublje od jedne razine (X/Y/…) nije dopušteno. Ne-.md datoteke (slike) se ne broje.
// Dovoljno je da se ime datoteke pojavi (poveznica ili spomen), kao u Sokratu.
const docsMd = mdFiles.filter((r) => r.startsWith("docs/") && r !== "docs/README.md");
if (!mdFiles.includes("docs/README.md")) {
  problems.push("NEMA INDEKSA  docs/README.md ne postoji  ->  to je jedini indeks dokumentacije");
} else {
  const idx = read("docs/README.md");
  for (const r of docsMd) {
    const dijelovi = r.split("/");
    if (dijelovi.length <= 3) {
      if (!idx.includes(path.posix.basename(r)))
        problems.push(`NIJE U INDEKSU  ${r}  ->  dodaj redak u docs/README.md`);
    } else if (dijelovi.length === 4) {
      const podindeks = `${dijelovi.slice(0, 3).join("/")}.md`;
      if (!mdFiles.includes(podindeks)) {
        problems.push(`NEMA PODINDEKSA  ${r}  ->  ${podindeks} ne postoji`);
      } else if (!read(podindeks).includes(path.posix.basename(r))) {
        problems.push(`NIJE U PODINDEKSU  ${r}  ->  dodaj redak u ${podindeks}`);
      }
    } else {
      problems.push(
        `PREDUBOKO  ${r}  ->  dopuštena je jedna razina (docs/<uloga>/X/*.md); spoji ili podigni razinu`,
      );
    }
  }
}

// ── 3) najviše jedan aktivni plan po području ───────────────────────
// ROADMAP.md je stalni stanovnik (pregled), ne plan. Plan koji čeka nosi u zaglavlju
// `**Status:** ⏸️ PAUZIRAN` pa ne računa; ispunjen plan ide u docs/archive/. Zaglavlje nosi
// `**Područje:** kod` ili `**Područje:** dokumentacija` (bez retka = kod): dvije sesije smiju
// raditi paralelno samo na različitim područjima. Računaju se samo docs/plan/*.md (taskovi iz
// docs/plan/X/ nisu planovi). Svrha: na pitanje „koji plan vrijedi?" po području jedan odgovor.
const PAUZIRAN = /^\s*\*\*Status:\*\*\s*\u23F8\uFE0F?\s*PAUZIRAN\b/m;
const PODRUCJA = ["kod", "dokumentacija"];
const planFiles = mdFiles.filter((r) => /^docs\/plan\/[^/]+\.md$/.test(r) && r !== "docs/plan/ROADMAP.md");
const aktivniPoPodrucju = new Map();
for (const r of planFiles) {
  const tekst = read(r);
  const zaglavlje = tekst.split(/\r?\n/).slice(0, 10).join("\n");
  const podrucje = /\*\*Područje:\*\*\s*([^\s·]+)/.exec(zaglavlje)?.[1] ?? "kod";
  if (!PODRUCJA.includes(podrucje)) {
    problems.push(
      `NEPOZNATO PODRUCJE  ${r}  ->  ${podrucje}\n      -> dopušteno: ${PODRUCJA.join(" | ")} (bez retka = kod)`,
    );
    continue;
  }
  if (PAUZIRAN.test(tekst)) continue;
  aktivniPoPodrucju.set(podrucje, [...(aktivniPoPodrucju.get(podrucje) ?? []), r]);
}
for (const [podrucje, aktivni] of aktivniPoPodrucju) {
  if (aktivni.length > 1) {
    problems.push(
      `VISE AKTIVNIH PLANOVA u docs/plan/ (podrucje: ${podrucje}): ${aktivni.map((r) => path.posix.basename(r)).join(", ")}` +
        "\n      -> ispunjen plan ide u docs/archive/, plan koji čeka dobiva `**Status:** ⏸️ PAUZIRAN`",
    );
  }
}

// ── 4) product/ je DEFINICIJA, ne dnevnik ───────────────────────────
// Više od 3 datuma znači da se kronologija uvukla u definiciju; ona ide u docs/records/.
for (const r of mdFiles.filter((x) => x.startsWith("docs/product/"))) {
  const dated = read(r).match(/\b20\d\d-\d\d-\d\d\b/g) || [];
  if (dated.length > 3) {
    problems.push(
      `DNEVNIK U DEFINICIJI  ${r}  (${dated.length} datuma)\n      -> product/ opisuje ŠTO proizvod mora znati; kronologija ide u docs/records/`,
    );
  }
}

// ── 5) CLAUDE.md ima BUDŽET ─────────────────────────────────────────
// Datoteka se učitava SVAKU sesiju pa je njezina duljina trošak svakog rada. Brana ne
// zabranjuje pisanje nego prijekutan RAST: budžet se mijenja SVJESNO — podigni konstantu i
// u komentaru napiši datum i razlog (tko ga podiže, mora reći zašto).
// 2026-10-04: početni budžet 12000 znakova.
// ⚠️ Mjeri se BEZ \r: `.length` broji i CR, pa ista datoteka ima ~1 znak više po retku u radnoj
// kopiji s CRLF-om (Windows) nego u CI-ju (LF) — isti commit bio bi crven lokalno, zelen u CI-ju.
const CLAUDE_BUDGET = 12000;
if (fs.existsSync(path.join(ROOT, "CLAUDE.md"))) {
  const size = read("CLAUDE.md").replace(/\r/g, "").length;
  if (size > CLAUDE_BUDGET) {
    problems.push(
      `CLAUDE.md PREKO BUDZETA  ${size} > ${CLAUDE_BUDGET} znakova\n      -> pouke idu u docs/, ovdje ostaje pokazivač; ako je rast opravdan, podigni CLAUDE_BUDGET u scripts/check-docs.mjs i zapiši razlog`,
    );
  }
}

// ── 6) ćirilica ─────────────────────────────────────────────────────
// Hrvatski i ćirilica dijele izgled nekoliko slova, pa se pri kopiranju neprimjetno uvuku
// (renderira isto, ali pretraga ih ne nalazi). Ćirilica UNUTAR koda je legitimna (navođenje
// samog raspona), pa se kod prije provjere uklanja. Raspon je escape da ova skripta sama
// nema ćirilicu.
for (const r of mdFiles) {
  const hits = stripCode(read(r)).match(/[\u0400-\u04FF]+/g);
  if (hits) {
    const uniq = [...new Set(hits)]
      .slice(0, 5)
      .map((h) => JSON.stringify(h))
      .join(", ");
    problems.push(
      `CIRILICA  ${r}  ->  ${uniq}\n      -> hrvatski tekst ne smije sadržavati ćirilične znakove (izgledaju isto, nisu isto)`,
    );
  }
}

// ── 7) arhiva ne smije biti proglašena aktivnom ─────────────────────
// Povod (Sokrat 2b): ROADMAP je tjednima tvrdio da je trenutni plan onaj iz archive/, a
// ispunjen je odavno; svaka svježa sesija krenula bi od te rečenice. Redak koji pokazuje na
// archive/ i uz to viče AKTIVAN laže: arhiva je referenca, ne izvor istine.
const AKTIVNO = /\bAKTIV(?:AN|NO|NI|NA)\b/;
for (const r of mdFiles.filter((x) => /^docs\/plan\/(?:[^/]+\/)?[^/]+\.md$/.test(x))) {
  read(r)
    .split(/\r?\n/)
    .forEach((line, i) => {
      if (line.includes("archive/") && AKTIVNO.test(line)) {
        problems.push(
          `ARHIVA PROGLASENA AKTIVNOM  ${r}:${i + 1}\n      -> ${line.trim().slice(0, 110)}\n      -> dokument u archive/ je REFERENCA, ne izvor istine`,
        );
      }
    });
}

// ── 9) veličina ─────────────────────────────────────────────────────
// Granice (i njihovi brojevi) su samo u scripts/docs-granice.mjs; ovdje se čitaju iz tog modula.
// Veliki dokument se ne reže ručno nego u indeks + mapu istog imena. Prijelazne iznimke su izuzete.
for (const r of mdFiles.filter((x) => x.startsWith("docs/"))) {
  const g = granicaDatoteke(r);
  const redaka = brojRedaka(read(r));
  // Preko tvrde granice javlja pravilo 12 (jednom, bez obzira na iznimke).
  if (g && redaka > g.redaka && redaka <= TVRDA_GRANICA && !jeIznimka(r)) {
    problems.push(
      `PREVELIKA DATOTEKA  ${r}  ${redaka} redaka > granica ${g.redaka} (${g.vrsta})\n      -> razdvoji u indeks + mapu istog imena (vidi docs/README.md)`,
    );
  }
}

// ── 12) tvrda granica: nijedan .md preko 1200 redaka ────────────────
// Vrijedi za cijeli repozitorij (ne samo docs/) i ne poznaje prijelazne iznimke; izuzeti su samo
// generirani/vanjski dokumenti izrijekom popisani u scripts/docs-granice.mjs.
for (const r of mdFiles) {
  const redaka = brojRedaka(read(r));
  if (redaka > TVRDA_GRANICA && !jeIzuzetOdTvrde(r)) {
    problems.push(
      `PREKO ${TVRDA_GRANICA} REDAKA  ${r}  ${redaka} redaka > tvrda granica ${TVRDA_GRANICA}\n      -> razdvoji po smislu u indeks + mapu istog imena (vidi docs/README.md)`,
    );
  }
}

// ── 10) ADR po datoteci + svjež generirani indeks ───────────────────
// Vrijedi kad postoji docs/records/DECISIONS.md ili mapa docs/records/DECISIONS/. DECISIONS.md je
// GENERIRAN (corepack pnpm docs:adr) — ručno uređen ili zastario indeks pada.
{
  const mapaAdr = path.join(ROOT, "docs/records/DECISIONS");
  const imaMapu = fs.existsSync(mapaAdr) && fs.statSync(mapaAdr).isDirectory();
  const imaIndeks = mdFiles.includes("docs/records/DECISIONS.md");
  if (imaMapu) {
    const imena = fs.readdirSync(mapaAdr).sort();
    const IME_ADR = /^ADR-(\d{3})-[a-z0-9-]+\.md$/;
    const vidjeni = new Map();
    for (const ime of imena) {
      const m = IME_ADR.exec(ime);
      if (!m) {
        problems.push(
          `ADR LOSE IME  docs/records/DECISIONS/${ime}  ->  mora biti ADR-NNN-slug.md (mala slova, brojke, crtice)`,
        );
        continue;
      }
      vidjeni.set(m[1], [...(vidjeni.get(m[1]) ?? []), ime]);
    }
    let brojeviOk = true;
    for (const [broj, datoteke] of vidjeni) {
      if (datoteke.length > 1) {
        brojeviOk = false;
        problems.push(`ADR DUPLI BROJ  ${broj}  ->  ${datoteke.join(", ")}`);
      }
    }
    if (brojeviOk && imena.every((ime) => IME_ADR.test(ime))) {
      if (!imaIndeks) {
        problems.push(
          "ADR INDEKS NIJE SVJEZ  docs/records/DECISIONS.md ne postoji  ->  corepack pnpm docs:adr",
        );
      } else {
        try {
          if (read("docs/records/DECISIONS.md").replace(/\r/g, "") !== indeksAdr(ROOT)) {
            problems.push("ADR INDEKS NIJE SVJEZ  docs/records/DECISIONS.md  ->  corepack pnpm docs:adr");
          }
        } catch (e) {
          problems.push(`ADR NECITLJIV  ${e.message}`);
        }
      }
    }
  } else if (imaIndeks) {
    problems.push(
      "ADR JOS NISU PO DATOTEKAMA  docs/records/DECISIONS.md bez mape DECISIONS/  ->  node scripts/adr.mjs razdvoji",
    );
  }
}

// ── 11) stare ADR poveznice ─────────────────────────────────────────
// Poveznice idu ravno na datoteku ADR-a; cilj `…DECISIONS.md#adr-NNN` je stari način. Gleda se CILJ
// poveznice (ne sirovi tekst): navod u inline kodu je spomen, ne poveznica.
for (const [r, ciljevi] of ciljeviPoDatoteci) {
  if (jeIznimka(r)) continue;
  for (const cilj of ciljevi) {
    if (cilj.includes("DECISIONS.md#adr-")) {
      problems.push(
        `STARA ADR POVEZNICA  ${r}  ->  ${cilj}\n      -> poveži ravno na docs/records/DECISIONS/ADR-NNN-….md`,
      );
    }
  }
}

// ── izvještaj ───────────────────────────────────────────────────────
console.log("\n=== check:docs ===");
console.log(`  .md dokumenata : ${mdFiles.length}`);
console.log(`  poveznica      : ${links}`);
if (problems.length === 0) {
  console.log("\n✅ dokumentacija je konzistentna\n");
  process.exit(0);
}
console.log(`\n❌ ${problems.length} problem(a):\n`);
for (const p of problems) console.log(`   • ${p}`);
console.log("");
process.exit(1);
