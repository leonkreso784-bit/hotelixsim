// ===== Zajednički alati za skripte dokumentacije =====
// Koriste ih razdvajanje planova, alat za ADR-ove i ručna razdvajanja. Jedna kopija da se
// slug i prepoznavanje poveznica ne razilaze. Bez nuspojava pri uvozu.

/** Naslov -> ASCII slug od najviše `rijeci` riječi, oblika `^[a-z0-9]+(-[a-z0-9]+)*$` (ili ""). */
export function slug(naslov, rijeci = 6) {
  return naslov
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .split(" ")
    .filter(Boolean)
    .slice(0, rijeci)
    .join("-");
}

const OTVARA_OGRADU = /^ {0,3}(`{3,}|~{3,})(.*)$/;
const ZATVARA_OGRADU = /^ {0,3}(`{3,}|~{3,})\s*$/;

/**
 * Za svaki redak: je li unutar ograđenog bloka koda (uključujući retke same ograde).
 * CommonMark: zatvara redak s istim znakom i duljinom >= otvarajuće, bez ičega iza osim razmaka.
 * Neotvorena ograda traje do kraja.
 */
export function uOgradi(retci) {
  const rez = [];
  let ograda = null; // { znak, duljina }
  for (const sirovi of retci) {
    const redak = sirovi.endsWith("\r") ? sirovi.slice(0, -1) : sirovi;
    if (ograda) {
      rez.push(true);
      const m = ZATVARA_OGRADU.exec(redak);
      if (m && m[1][0] === ograda.znak && m[1].length >= ograda.duljina) ograda = null;
      continue;
    }
    const m = OTVARA_OGRADU.exec(redak);
    // info-string ograde od ` ne smije sadržavati ` (inače je to inline kod, ne ograda)
    if (m && !(m[1][0] === "`" && m[2].includes("`"))) {
      ograda = { znak: m[1][0], duljina: m[1].length };
      rez.push(true);
    } else rez.push(false);
  }
  return rez;
}

const INLINE_KOD = /(`+)(?:(?!\1)[\s\S])+?\1/g;
const JE_RELATIVAN = (cilj) =>
  !/^[a-z][a-z0-9+.-]*:/i.test(cilj) && !cilj.startsWith("#") && !cilj.startsWith("/");

// Završetak teksta poveznice: indeks zatvarajuće `]` uz uravnotežene zagrade (ili -1).
function krajTeksta(t, od) {
  let dubina = 0;
  for (let i = od; i < t.length; i++) {
    if (t[i] === "\\") i++;
    else if (t[i] === "[") dubina++;
    else if (t[i] === "]" && --dubina === 0) return i;
  }
  return -1;
}

// Cilj nakon `(`: do razmaka ili `)` na dubini 0 (zagrade u cilju, `x(1).md`, se uravnotežuju).
function krajCilja(t, od) {
  let dubina = 0;
  let i = od;
  for (; i < t.length; i++) {
    if (t[i] === "\\") i++;
    else if (t[i] === "(") dubina++;
    else if (t[i] === ")") {
      if (dubina === 0) break;
      dubina--;
    } else if (/\s/.test(t[i])) break;
  }
  return i;
}

// Skener umjesto jednog regexa: tekst poveznice se rekurzivno obrađuje, pa se prepišu i
// slika u tekstu poveznice (`[![b](img.png)](x.md)`) i vanjski cilj.
function prepisiUTekstu(tekst, fn) {
  let izlaz = "";
  let i = 0;
  while (i < tekst.length) {
    const pocetak = tekst[i] === "!" && tekst[i + 1] === "[" ? i + 1 : tekst[i] === "[" ? i : -1;
    if (pocetak === -1) {
      izlaz += tekst[i++];
      continue;
    }
    const zatvori = krajTeksta(tekst, pocetak);
    if (zatvori === -1 || tekst[zatvori + 1] !== "(") {
      izlaz += tekst.slice(i, pocetak + 1);
      i = pocetak + 1;
      continue;
    }
    const odCilja = zatvori + 2;
    const doCilja = krajCilja(tekst, odCilja);
    const naslov = /^(?:\s+"[^"\n]*")?\)/.exec(tekst.slice(doCilja));
    const cilj = tekst.slice(odCilja, doCilja);
    if (doCilja === odCilja || !naslov) {
      izlaz += tekst.slice(i, pocetak + 1);
      i = pocetak + 1;
      continue;
    }
    const unutra = prepisiUTekstu(tekst.slice(pocetak + 1, zatvori), fn);
    const noviCilj = JE_RELATIVAN(cilj) ? fn(cilj) : cilj;
    izlaz += `${tekst.slice(i, pocetak + 1)}${unutra}](${noviCilj}${naslov[0]}`;
    i = doCilja + naslov[0].length;
  }
  return izlaz;
}

/**
 * Svaki relativni cilj inline poveznice `[t](cilj)` i slike `![a](cilj)` (uz opcionalni naslov)
 * izvan ograda i izvan inline koda prolazi kroz `fn(cilj) -> noviCilj`. Ostalo (i `\r\n`) ostaje bajt-točno.
 */
export function prepisiRelativnePoveznice(text, fn) {
  const retci = text.split("\n");
  const ograde = uOgradi(retci);
  return retci
    .map((redak, i) => {
      if (ograde[i]) return redak;
      let izlaz = "";
      let kraj = 0;
      for (const m of redak.matchAll(INLINE_KOD)) {
        izlaz += prepisiUTekstu(redak.slice(kraj, m.index), fn) + m[0];
        kraj = m.index + m[0].length;
      }
      return izlaz + prepisiUTekstu(redak.slice(kraj), fn);
    })
    .join("\n");
}
