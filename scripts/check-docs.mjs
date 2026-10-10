#!/usr/bin/env node
// ===== check:docs — integritet dokumentacije =====
// Postoji jer dokumentacija bez brane trune tiho: poveznice umru, dokumenti ostanu bez
// indeksa, "aktivnih" planova postane pet, a CLAUDE.md (učitava se SVAKU sesiju) naraste.
// Raspored ima jedan indeks (docs/README.md), jedan aktivni plan i definicije bez dnevnika;
// ova brana čuva da tako i ostane. Uzor: Sokratov scripts/check-docs.js.
//
// Pada (exit 1) ako:
//   1) relativna .md poveznica (ili poveznica na mapu) ne vodi nikamo     → mrtve veze
//   2) .md pod docs/ nije spomenut u docs/README.md (ili ga nema)         → dokument-duh
//   3) docs/plan/ ima više od JEDNOG aktivnog plana                        → „koji plan vrijedi?"
//   4) docs/product/** ima datoteku s više od 3 ISO datuma                 → definicija ≠ dnevnik
//   5) CLAUDE.md je preko budžeta                                          → trošak svake sesije
//   6) ćirilica u .md izvan koda                                           → kopirano slovo-dvojnik
//   7) docs/plan/*.md proglašava plan iz archive/ AKTIVNIM                 → arhiva laže da je živa
//
// Pokreni: pnpm check:docs   |   korijen se može zadati: node scripts/check-docs.mjs <dir>
// ili env CHECK_DOCS_ROOT (tako ga testira check-docs.test.mjs nad privremenim mapama).
// `--popis` (uz ili bez korijena) ne provjerava nego ispisuje veličine svih docs/**/*.md
// uz granicu (scripts/docs-granice.mjs), silazno; ❌ uz prekoračenje; uvijek exit 0.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { brojRedaka, granicaZa, jeIznimka } from "./docs-granice.mjs";

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

// ── --popis: ispis veličina umjesto provjere ────────────────────────
if (POPIS) {
  const redovi = mdFiles
    .filter((r) => r.startsWith("docs/"))
    .map((r) => {
      const mapa = path.join(ROOT, r.slice(0, -3));
      const imaMapu = fs.existsSync(mapa) && fs.statSync(mapa).isDirectory();
      const g = granicaZa(r, imaMapu);
      return { r, redaka: brojRedaka(read(r)), g };
    })
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
// Vrijede `[tekst](put.md#sidro)` i poveznice na mapu (`../research/`). Sidro se ne provjerava,
// samo cilj. Postojanje se pita disk (ne popis), da poveznica na mapu i na gitignorirano ne lažu.
let links = 0;
const LINK_RE = /\]\(([^)\s]+?(?:\.md|\/))(#[^)\s]*)?\)/g;
for (const r of mdFiles) {
  const dir = path.posix.dirname(r);
  for (const m of stripCode(read(r)).matchAll(LINK_RE)) {
    const link = m[1];
    if (/^[a-z][a-z0-9+.-]*:/i.test(link)) continue; // http://, mailto: …
    links++;
    let target = link;
    try {
      target = decodeURIComponent(link);
    } catch {}
    const abs = path.join(ROOT, path.posix.join(dir, target));
    const ok = target.endsWith("/")
      ? fs.existsSync(abs) && fs.statSync(abs).isDirectory()
      : fs.existsSync(abs) && fs.statSync(abs).isFile();
    if (!ok) problems.push(`MRTVA POVEZNICA  ${r}  ->  ${link}`);
  }
}

// ── 2) svaki .md pod docs/ mora biti u docs/README.md ───────────────
// JEDINI indeks je pravilo rasporeda; dokument kojeg indeks ne spominje nitko ne nađe.
// Dovoljno je da se ime datoteke pojavi (poveznica ili spomen), kao u Sokratu.
const docsMd = mdFiles.filter((r) => r.startsWith("docs/") && r !== "docs/README.md");
if (!mdFiles.includes("docs/README.md")) {
  problems.push("NEMA INDEKSA  docs/README.md ne postoji  ->  to je jedini indeks dokumentacije");
} else {
  const idx = read("docs/README.md");
  for (const r of docsMd) {
    if (!idx.includes(path.posix.basename(r)))
      problems.push(`NIJE U INDEKSU  ${r}  ->  dodaj redak u docs/README.md`);
  }
}

// ── 3) najviše jedan aktivni plan ───────────────────────────────────
// ROADMAP.md je stalni stanovnik (pregled), ne plan. Plan koji čeka nosi u zaglavlju
// `**Status:** ⏸️ PAUZIRAN` pa ne računa; ispunjen plan ide u docs/archive/. Svrha: na pitanje
// „koji plan vrijedi?" smije postojati samo jedan odgovor.
const PAUZIRAN = /^\s*\*\*Status:\*\*\s*\u23F8\uFE0F?\s*PAUZIRAN\b/m;
const planFiles = mdFiles.filter((r) => /^docs\/plan\/[^/]+\.md$/.test(r) && r !== "docs/plan/ROADMAP.md");
const aktivni = planFiles.filter((r) => !PAUZIRAN.test(read(r)));
if (aktivni.length > 1) {
  problems.push(
    `VISE AKTIVNIH PLANOVA u docs/plan/: ${aktivni.map((r) => path.posix.basename(r)).join(", ")}` +
      "\n      -> ispunjen plan ide u docs/archive/, plan koji čeka dobiva `**Status:** ⏸️ PAUZIRAN`",
  );
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
for (const r of mdFiles.filter((x) => /^docs\/plan\/[^/]+\.md$/.test(x))) {
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
