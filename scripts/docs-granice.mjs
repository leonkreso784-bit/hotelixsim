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

export function jeIznimka(relPath) {
  return Object.hasOwn(PRIJELAZNE_IZNIMKE, relPath);
}

/**
 * Granica za datoteku (putanja s `/` od korijena repozitorija).
 * `imaMapuIstogImena` = pokraj `X.md` postoji mapa `X/` (tada je `X.md` indeks).
 * Vraća `{ vrsta, redaka }` ili `null` za sve izvan `docs/`.
 */
export function granicaZa(relPath, imaMapuIstogImena) {
  if (!relPath.startsWith("docs/")) return null;
  if (relPath.startsWith("docs/archive/")) return { vrsta: "arhiva", redaka: 1100 };
  if (relPath === "docs/SLJEDECA_SESIJA.md") return { vrsta: "predaja", redaka: 60 };
  if (relPath.startsWith("docs/records/DECISIONS/")) return { vrsta: "adr", redaka: 120 };
  if (relPath === "docs/README.md" || imaMapuIstogImena) return { vrsta: "indeks", redaka: 200 };
  return { vrsta: "zivi", redaka: 300 };
}
