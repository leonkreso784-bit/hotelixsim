# ADR-043 — Popravak temelja po fazama A–C, ne gradnja ispočetka
**Datum:** 2026-10-14 · **Status:** ✅ **Leonova odluka 14.10.** (chat: *„može super idemo napraviti popravak i samo se tome
posvetit dok nije gotovo"*) · **Vezano:** [REVIZIJA_TEMELJA.md](../../architecture/REVIZIJA_TEMELJA.md),
[ADR-038](ADR-038-nacin-rada-dva-kolosijeka-faze-i.md), [ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md),
[BUGS.md](../BUGS.md) B1–B10

**Kontekst.** Leon se 14.10. pobojao da „ne gradimo od baze nego od sagrađenog bungalova radimo ciglenu nadogradnju".
Revizija šest slojeva (samo čitanje, [REVIZIJA_TEMELJA.md](../../architecture/REVIZIJA_TEMELJA.md)) zaključila je da temelj
stoji (dnevnik događaja, determinizam, snimak pravila, baza, red poslova, AI iza sučelja, ugovor API-ja), a da puca na tri
mjesta: v2 na v1 kalupu (mehanike situacija bez odjela, B2), brane balansa koje mjere robota umjesto studenta (B6) i duge
odvojene grane (B6, B8, B9). Uz to načela igre nisu zapisana kao provjere (B1, B2, B4). Popravak je procijenjen na
~125–185 h, gradnja ispočetka na ~540–910 h s istim nedostajućim katovima.

**Odluka.**
1. **Popravak, ne gradnja ispočetka.** Jezgra, baza, red poslova, AI sloj i web se ne prepisuju.
2. **Samo popravak dok nije gotov.** Nove značajke (RAZRED, LJUDI, banka ljudi K1, voditelji) čekaju kraj popravka.
   Plan: [2026-10-14-popravak-temelja.md](../../plan/2026-10-14-popravak-temelja.md).
3. **Redoslijed A → B → C.** A učvršćuje (brane koje mjere pravu stvar, kratke grane), B mijenja v1 kalup ondje gdje već
   puca, C priprema igru za semestar i razred. **Kalibracija balansa radi se jednom, na kraju faze B** (L1-13), a ne
   odmah na grani situacija, kako se ne bi kalibriralo dvaput.
4. **Brane balansa se dijele** na brane modela (ruše provjeru) i mjere robota (ispisuju se). Nova definicija „udjela
   situacija" ide Leonu na odluku kad dođe na red (faza B).
5. **Kratke grane:** jedna integracijska grana (`model-v2/sloj1`), grana cigle živi najviše ~1 dan i spaja se isti dan,
   balans se vrti na integracijskoj grani nakon svakog spajanja ([AGENTI.md](../../workflow/AGENTI.md) §2).
6. **Sigurnosna kopija** svih lokalnih grana i ledgera izvan repozitorija (`Documents/hotelixsim-kopije/<datum>/`) na
   početku popravka i na kraju svake faze. Push na javni GitHub i dalje samo uz Leonov izričit OK.

**Posljedice.**
- Plan [Sloja 1](../../plan/2026-10-10-v2-sloj1.md) je pauziran: L1-11 postaje cigla A1, L1-13 i L1-13b idu na kraj faze B,
  L1-14 i L1-15 zatvaraju fazu B. ROADMAP faze iza ZAVRŠETKA SLOJA 1 pomiču se za trajanje popravka (~125–185 h [PROC]).
- Svaka faza završava Leonovim igranjem i kopijom; odluke o brojkama koje student osjeti idu kroz anketu.

**Izmjena 17.10. (Leonova odluka 16.10., sesija 7; [ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 8).**
t. 2 se sužava: **kolosijek PLATFORMA počinje u studenom 2026., usporedo s fazom C**, ne nakon kraja popravka (P3 ovisi o platformi više nego o C4;
nacrt: [plan platforme](../../plan/2026-11-platforma.md)). Banka ljudi već ide usporedo ([ADR-052](ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md)
t. 2), sada i kao paralelna sesija (ADR-057 t. 6). Ograničenje ostaje: najviše 2 agenta koji prevode Rust ukupno. Ostale nove značajke igre
(LJUDI, voditelji, sobe, smještaj radnika, banka hotela) i dalje čekaju kraj popravka; redoslijed nakon L1-13 je 🟨
[ADR-058](ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md) t. 22.
