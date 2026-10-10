// ===== Razdvajanje velikog plana: pregled + jedna datoteka po tasku =====
// `node scripts/razdvoji-plan.mjs <plan.md> [--provjeri]` (ili `corepack pnpm docs:plan <plan.md>`).
// Bez --provjeri: pregled se zapisuje preko izvornika, a taskovi u mapu <ime>/ pokraj njega.
// Sa --provjeri: ništa se ne mijenja; razdvoji u privremenu mapu, pročita natrag s diska i
// dokaže da je spajanje jednako izvorniku bajt po bajt.
// Uvoz nema nuspojava (izvršni dio je pod provjerom import.meta.url).
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { prepisiRelativnePoveznice, slug, uOgradi } from "./docs-alati.mjs";

const NASLOV_TASKA = /^#+\s+Task\s+\d+/;
const NASLOV_TASKA_BROJ = /^#+\s+Task\s+(\d+)/;
const NASLOV_RAZINE_2 = /^## /;
const NASLOV_TABLICE = "## Taskovi";
const ZAGLAVLJE_TABLICE = "| N | Task |";
const RAZDJELNIK_TABLICE = "|---|---|";

// Retci s vlastitim završetkom (\n ili \r\n ostaju uz redak), pa je spajanje bajt-točno.
const uRetke = (text) => text.match(/[^\n]*\n|[^\n]+/g) ?? [];
const bezKraja = (redak) => redak.replace(/\r?\n$/, "");

function ogradeRetaka(retci) {
  return uOgradi(retci.map((r) => r.replace(/\n$/, "")));
}

function nazivDatoteke(naslov, broj) {
  const nn = String(broj).padStart(2, "0");
  const ostatak = naslov.replace(/^#+\s+/, "").replace(/^Task\s+\d+\s*:?\s*/, "");
  const s = slug(ostatak, 6);
  return s ? `task-${nn}-${s}.md` : `task-${nn}.md`;
}

/** Razdvaja plan na pregled i taskove. Vidi zaglavlje datoteke za pravila. */
export function razdvoji(text, ime) {
  const retci = uRetke(text);
  const uOgr = ogradeRetaka(retci);
  const pocetci = [];
  for (let i = 0; i < retci.length; i++) {
    if (!uOgr[i] && NASLOV_TASKA.test(bezKraja(retci[i]))) pocetci.push(i);
  }
  if (pocetci.length === 0) return { pregled: text, taskovi: [] };

  // Zadnji task završava na prvom naslovu razine 2 izvan ograde, inače na kraju datoteke.
  const zadnji = pocetci[pocetci.length - 1];
  let kraj = retci.length;
  for (let i = zadnji + 1; i < retci.length; i++) {
    if (!uOgr[i] && NASLOV_RAZINE_2.test(retci[i])) {
      kraj = i;
      break;
    }
  }

  const eol = retci[pocetci[0]].endsWith("\r\n") ? "\r\n" : "\n";
  const taskovi = [];
  const vidjena = new Set();
  const redciTablice = [];
  for (let k = 0; k < pocetci.length; k++) {
    const od = pocetci[k];
    const do_ = k + 1 < pocetci.length ? pocetci[k + 1] : kraj;
    const naslov = bezKraja(retci[od]);
    const broj = Number(NASLOV_TASKA_BROJ.exec(naslov)[1]);
    const datoteka = nazivDatoteke(naslov, broj);
    if (vidjena.has(datoteka)) {
      throw new Error(`Dva taska dobivaju isto ime datoteke: ${datoteka}`);
    }
    vidjena.add(datoteka);
    const tekst = prepisiRelativnePoveznice(retci.slice(od, do_).join(""), (c) => `../${c}`);
    taskovi.push({ datoteka, tekst });
    const prikaz = naslov.replace(/^#+\s+/, "").replace(/\|/g, "\\|");
    redciTablice.push(`| ${broj} | [${prikaz}](${ime}/${datoteka}) |${eol}`);
  }

  const blok = [
    `${NASLOV_TABLICE}${eol}`,
    eol,
    `${ZAGLAVLJE_TABLICE}${eol}`,
    `${RAZDJELNIK_TABLICE}${eol}`,
    ...redciTablice,
    eol,
  ];
  const pregled = [...retci.slice(0, pocetci[0]), ...blok, ...retci.slice(kraj)].join("");
  return { pregled, taskovi };
}

/** Spaja pregled i taskove natrag u izvornik (obrnuto od `razdvoji`). */
export function spoji(pregled, taskovi) {
  const retci = uRetke(pregled);
  const uOgr = ogradeRetaka(retci);
  const b = retci.findIndex((r, i) => !uOgr[i] && bezKraja(r) === NASLOV_TABLICE);
  if (b === -1 || bezKraja(retci[b + 2] ?? "") !== ZAGLAVLJE_TABLICE) {
    if (taskovi.length === 0) return pregled;
    throw new Error(`U pregledu nema tablice "${NASLOV_TABLICE}".`);
  }
  let k = b + 4; // naslov, prazan, zaglavlje, razdjelnik
  while (k < retci.length && retci[k].startsWith("|")) k++;
  if (k < retci.length && bezKraja(retci[k]) === "") k++; // prazan redak iza tablice
  const tijela = taskovi.map((t) =>
    prepisiRelativnePoveznice(t.tekst, (c) => (c.startsWith("../") ? c.slice(3) : c)),
  );
  return [...retci.slice(0, b), ...tijela, ...retci.slice(k)].join("");
}

function ispisi(poruka) {
  process.stdout.write(`${poruka}\n`);
}

function greska(poruka) {
  process.stderr.write(`GREŠKA: ${poruka}\n`);
  process.exitCode = 1;
}

function zapisiMapu(dir, taskovi) {
  fs.mkdirSync(dir, { recursive: true });
  for (const t of taskovi) fs.writeFileSync(path.join(dir, t.datoteka), t.tekst);
}

// Čita zapisane datoteke natrag s diska i uspoređuje spoj s izvornikom bajt po bajt.
function provjeriSDiska(izvornikBajtovi, pregledPutanja, dir, taskovi) {
  const pregled = fs.readFileSync(pregledPutanja, "utf8");
  const sDiska = taskovi.map((t) => ({
    datoteka: t.datoteka,
    tekst: fs.readFileSync(path.join(dir, t.datoteka), "utf8"),
  }));
  return Buffer.from(spoji(pregled, sDiska)).equals(izvornikBajtovi);
}

function glavna(argv) {
  const provjeri = argv.includes("--provjeri");
  const putanje = argv.filter((a) => !a.startsWith("--"));
  if (putanje.length !== 1) {
    return greska("Upotreba: node scripts/razdvoji-plan.mjs <plan.md> [--provjeri]");
  }
  const plan = path.resolve(putanje[0]);
  if (!fs.existsSync(plan)) return greska(`Datoteka ne postoji: ${plan}`);

  const bajtovi = fs.readFileSync(plan);
  const ime = path.basename(plan, ".md");
  let rez;
  try {
    rez = razdvoji(bajtovi.toString("utf8"), ime);
  } catch (e) {
    return greska(e.message);
  }
  if (rez.taskovi.length === 0) return greska("U planu nema nijednog task-naslova izvan ograde.");

  if (provjeri) {
    const privremena = fs.mkdtempSync(path.join(os.tmpdir(), "razdvoji-provjera-"));
    try {
      const pregledPut = path.join(privremena, `${ime}.md`);
      fs.writeFileSync(pregledPut, rez.pregled);
      const dir = path.join(privremena, ime);
      zapisiMapu(dir, rez.taskovi);
      if (!provjeriSDiska(bajtovi, pregledPut, dir, rez.taskovi)) {
        return greska("Spoj razdvojenih datoteka NIJE jednak izvorniku.");
      }
    } finally {
      fs.rmSync(privremena, { recursive: true, force: true });
    }
    return ispisi(`OK: ${rez.taskovi.length} taskova, spoj = izvornik bajt po bajt.`);
  }

  const dir = path.join(path.dirname(plan), ime);
  if (fs.existsSync(dir) && fs.readdirSync(dir).length > 0) {
    return greska(`Mapa već postoji i nije prazna: ${dir}. Ništa nije prepisano.`);
  }
  zapisiMapu(dir, rez.taskovi);
  fs.writeFileSync(plan, rez.pregled);
  if (!provjeriSDiska(bajtovi, plan, dir, rez.taskovi)) {
    return greska("Spoj zapisanih datoteka NIJE jednak izvorniku. Vrati izvornik iz gita.");
  }
  ispisi(`Razdvojeno: ${rez.taskovi.length} taskova u ${dir}; pregled u ${plan}; spoj = izvornik.`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  glavna(process.argv.slice(2));
}
