// ===== Granice veličine dokumentacije =====
// Jedno mjesto za tablicu granica (redaka, kao `wc -l`). Koriste ga `check:docs` (brana) i
// `check:docs --popis` (ispis veličina). Modul nema nuspojava pri uvozu.

/** Broji retke kao `wc -l`: samo `\n`, `\r` se ne broji (CRLF i LF daju isti rezultat). */
export function brojRedaka(text) {
  let n = 0;
  for (let i = text.indexOf("\n"); i !== -1; i = text.indexOf("\n", i + 1)) n++;
  return n;
}

// Prijelazne iznimke: izuzete od granica (i od zabrane starih ADR poveznica) dok sesija koda
// piše Sloj 1. Prazni se u Tasku 13 plana dokumentacije.
export const PRIJELAZNE_IZNIMKE = {
  "docs/plan/2026-10-10-v2-sloj1.md": "sesija koda ga piše dok traje Sloj 1; prazni se u Tasku 13",
  "docs/architecture/MODEL_V2.md": "sesija koda ga piše dok traje Sloj 1; prazni se u Tasku 13",
};

/** Iznimka vrijedi za `X.md` i, kad je razdvojen, za njegove dijelove u mapi `X/`. */
export function jeIznimka(relPath) {
  return Object.keys(PRIJELAZNE_IZNIMKE).some(
    (k) => relPath === k || relPath.startsWith(`${k.slice(0, -3)}/`),
  );
}

// Tvrda granica (Leon, 2026-10-13; podignuta s 1000 na 1200 isti dan): nijedan .md u repozitoriju —
// ni izvan docs/ — nema više od 1200 redaka. Prijelazne iznimke NE vrijede za nju. Gitignorirano (node_modules, target,
// .superpowers …) ionako ne ulazi u popis brane.
export const TVRDA_GRANICA = 1200;

// Izuzeci od tvrde granice: SAMO generirani ili vanjski .md koje git prati (ključ = putanja,
// vrijednost = razlog). Danas nema nijednog; dodaje se izrijekom, uz datum i razlog.
export const IZUZECI_TVRDE_GRANICE = {};

export function jeIzuzetOdTvrde(relPath) {
  return Object.hasOwn(IZUZECI_TVRDE_GRANICE, relPath);
}

/**
 * Granica za datoteku (putanja s `/` od korijena repozitorija).
 * `imaMapuIstogImena` = pokraj `X.md` postoji mapa `X/` (tada je `X.md` indeks).
 * Vraća `{ vrsta, redaka }` ili `null` za sve izvan `docs/`.
 */
export function granicaZa(relPath, imaMapuIstogImena) {
  if (!relPath.startsWith("docs/")) return null;
  if (relPath.startsWith("docs/archive/")) return { vrsta: "arhiva", redaka: 1000 };
  if (relPath === "docs/SLJEDECA_SESIJA.md") return { vrsta: "predaja", redaka: 60 };
  if (relPath.startsWith("docs/records/DECISIONS/")) return { vrsta: "adr", redaka: 120 };
  if (relPath === "docs/README.md" || imaMapuIstogImena) return { vrsta: "indeks", redaka: 200 };
  return { vrsta: "zivi", redaka: 300 };
}
