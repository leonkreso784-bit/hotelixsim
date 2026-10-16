# ADR-052 — Leonove odluke 16.10.: uvjet kvara klime i praonice, banka ljudi usporedo, tri odgovora za K1
**Datum:** 2026-10-16 · **Status:** ✅ **Leonove odluke 16.10.** · **Vezano:** [ADR-043](ADR-043-popravak-temelja-ne-gradnja-ispocetka.md),
[ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 8, [BANKA_LJUDI](../../architecture/BANKA_LJUDI.md) §5.1,
[plan popravka](../../plan/2026-10-14-popravak-temelja.md) (B4a)

**1. Uvjet kvara P-KLIMA i P-PRAONICA** (anketa B4a): kvar se može dogoditi samo kad je stanje Smještaja **ispod 65**
(`dept_condition_pts` below 6500), kao kod bojlera. P-KLIMA zadržava pojačanje ispod 50. Mjereno: B3 težina tjedno A 42,8 → 41,9 %,
B 48,9 → 48,9 %, C 41,5 → 41,2 %; mjesečno A 25,9 → 25,7 %; oba `--check` prolaze. Spojeno s B4a (6e8ff2e).
Isti prag za **P-BRAVE, P-CURENJE-VODE i P-DIZALO** (B4c) **nije odlučen**: s njim tjedni krug profila A pri kraju sezone nema dovoljno
dopuštenih situacija (brana J-3 pada), pa Leon odlučuje uz brojke prije spajanja ([BACKLOG](../BACKLOG.md)). **Riješeno 16.10. (sesija 6): prag 80, [ADR-054](ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md) t. 1.**

**2. Banka ljudi ide usporedo s popravkom temelja** (Leon: *„možemo li poslat još jednog agenta da gradi likove u igrici po planu"*).
Dopuna ADR-043 („samo popravak"): K1 (format, pravila, 20 primjera) je zaseban kolosijek u svom ledgeru, grana `sadrzaj/banka-ljudi`,
ne dira igru ni balans (bajt-isto). Ograničenje: najviše 2 agenta koji prevode Rust ukupno.

**3. Tri odgovora za K1** (anketa):
- omjer roda po ulozi = **Eurostat LFS Hrvatska**, ISCO-08 × spol, zadnja godina (svaki unos s izvorom);
- strani radnici (zemlje) = **MUP dozvole za boravak i rad**, turizam i ugostiteljstvo, zadnja godina; dotad popis iz §2.2 🟨;
- **uloga smije ovisiti o podrijetlu** (stvarnost); razina, dob i rod unutar uloge ne smiju.

**Cijena ako krivo:** (1) jedna konstanta u sadržaju; (2) dva kolosijeka troše isti stroj, pa je puna granica Rust procesa; (3) `raspodjele.json`.
