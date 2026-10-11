// Pomoćnik za testove naslova kruga: aplikacija iza crtice u datumima stavlja spojnik riječi (U+2060), a nerazdvojne
// razmake testing-library pri uspoređivanju svodi na obične. Zato očekivani tekst dobiva samo spojnik.
const WORD_JOINER = "\u2060";

/** „Tjedan 1 (1. 6. – 7. 6.)“ → isti tekst sa spojnikom iza crtice, kakvog ga vide upiti po imenu i tekstu. */
export function joined(text: string): string {
  return text.replace(/–/g, `–${WORD_JOINER}`);
}
