# ADR-027 — Raspored za računalo prije demoa
**Datum:** 2026-10-10 · **Status:** 🟨 **ČEKA LEONOV PREGLED** · **Vezano:** [ADR-021](ADR-021-koncept-ekrana-v2-i-odluke-o.md), [PROSIRENJA.md](../../product/PROSIRENJA.md) P1

**Kontekst.** Leon 10.10.: *„prilagodio bi kompjuteru da paše UI isto, ne samo mobitelu"*. Ekrani su bili zaključani
na širinu mobitela (`max-w-md`); profesor će demo najvjerojatnije gledati na laptopu ili projektoru.
**Odluka (pretpostavka).** Mobitel (< 768 px) ostaje kakav jest. Od 768 px izbornik prelazi u bočnu traku, od 1024 px
Moj hotel, Mjesec završen i Situacija dobivaju dva stupca; ostali ekrani jedan stupac ograničene širine čitanja. Samo
web, bez promjene igre, prije demoa (14.10.).
**Cijena ako krivo:** pola dana rada; Leon možda želi drukčiji raspored za računalo — mijenja se samo `ui/Layout.tsx`
i četiri ekrana.
