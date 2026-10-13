# ADR-007 — Lokalna baza: zaseban PostgreSQL 16 klaster na priključku 5433
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Claude kao pretpostavka; Leon pregledava s [ADR-008](ADR-008-odluke-iz-gradnje-kostura-koje-je.md)) ·
**Vezano:** [ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md), [ADR-008](ADR-008-odluke-iz-gradnje-kostura-koje-je.md)

**Kontekst.** Sistemski PostgreSQL na 5432 već postoji, a Leon ne zna njegovu lozinku. Treba lokalna baza za razvoj i demo.
**Odluka.** Zaseban klaster PostgreSQL 16 u `C:\Users\leonk\AppData\Local\hotelixsim\pgdata`, priključak **5433**.
Sistemski na 5432 se ne dira. Umjesto resetiranja njegove lozinke.
**Posljedice.** Ne pali se sam nakon restarta: `bash scripts/baza.sh start`. Cijena ako je krivo: obrisati `pgdata`.
Pokretanje svega opisano je u `README.md` („Lokalni razvoj").
