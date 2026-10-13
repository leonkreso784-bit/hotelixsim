# ADR-010 — Rad u Claude Code dodatku za VS Code (panel), ne u terminalu
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-011](ADR-011-dokumentacija-po-sokratovom-sustavu-mape-po.md)

**Kontekst.** Leonu je VS Code okruženje prirodnije (Explorer, preglednik uz kod, slike povlačenjem).
**Odluka.** Leon: *„nećemo mi više radit u terminalu… ostat ćemo u VS Code okruženju."* Upute za prečace i slike daju
se za panel dodatka (otvaranje: iskrica / `Ctrl + Esc`; slika: `Ctrl + V` ili povlačenje u polje za poruku), ne za terminal.
**Posljedice.** Za stvari koje treba vidjeti, zaslon se može snimiti i izravno, ali vidi se samo ono što je otvoreno.
Ne mijenja ni kod ni arhitekturu.
