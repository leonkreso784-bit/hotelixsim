# HotelixSim — arhitektura sustava

> **Status:** NACRT v2 (2026-10-04). Ugrađen krug 1 rasprave ([RASPRAVA_ARHITEKTURE.md](./RASPRAVA_ARHITEKTURE.md)):
> ispravka presude (§8), puni snimak pravila (§7.2, §17), pravednost profila i AI-ja (§7.8, §10.3), red poslova
> (§9.4), što tko vidi (§12, §7.5), slobodan potez (§8.5), obrazloženje za Rust (§5, §18), lokalno pa Railway (§14).
> Domena (§1) i jezici (Rust + TypeScript, §5) dogovoreni su 04.10.; ostalo je prijedlog dok Leon ne potvrdi.
> Vizija proizvoda: [VIZIJA.md](../product/VIZIJA.md) · plan: [ROADMAP.md](../plan/ROADMAP.md).

## Dijelovi

Kod citira ‚ARHITEKTURA §N' — § brojevi su sačuvani u naslovima dijelova.

| § | dio |
|---|---|
| 0 | sažetak na jednoj stranici (ovdje, niže) |
| §1–3 · domena, zahtjevi, rječnik | [01-domena-zahtjevi-rjecnik.md](./ARHITEKTURA/01-domena-zahtjevi-rjecnik.md) |
| §4–6 · pregled sustava, jezici, struktura koda | [04-sustav-jezici-kod.md](./ARHITEKTURA/04-sustav-jezici-kod.md) |
| §7–8 · jezgra simulacije, tok runde | [07-jezgra-i-runda.md](./ARHITEKTURA/07-jezgra-i-runda.md) |
| §9–12 · podaci, AI sloj, API, prijava i prava | [09-podaci-ai-api-prava.md](./ARHITEKTURA/09-podaci-ai-api-prava.md) |
| §13–17 · sučelje, isporuka, testiranje, sigurnost, proširivost | [13-sucelje-isporuka-sigurnost.md](./ARHITEKTURA/13-sucelje-isporuka-sigurnost.md) |
| §18–20 · odbačeno, otvoreno, redoslijed gradnje | [18-odbaceno-otvoreno-redoslijed.md](./ARHITEKTURA/18-odbaceno-otvoreno-redoslijed.md) |

---

## 0 · Sažetak na jednoj stranici

**Što gradimo:** simulaciju vođenja hotela za studente (FMTU). Student dobije hotel, rješava situacije i donosi
odluke uz AI; ocjenjuje se **napredak u odnosu na početak**, ne zarada. Danas web-aplikacija za mobitel i laptop;
sutra mobilna aplikacija, dio platforme s više simulatora i simulator koji stalno raste.

**Načelo:** srce sustava je **simulacija** — čista, deterministička knjižnica koja ne zna za ekran, bazu ni AI.
Sve ostalo su slojevi oko nje koji se mogu mijenjati bez diranja srca. Svaki dio ima jedan posao i jednu jasnu vezu.

```text
   TypeScript                         Rust
┌──────────────────┐   OpenAPI   ┌──────────────────────────────────────────┐
│ web (React)      │◄───────────►│ API (Axum)  ── prijava, prava, ulaz      │
│ kasnije: Expo    │  generirani │     │                                     │
│ (mobitel)        │  tipovi     │ APLIKACIJA ── slučajevi upotrebe          │
└──────────────────┘             │     │            │             │          │
                                 │  JEZGRA       POHRANA        AI           │
                                 │  (čista)    (PostgreSQL)  (pružatelji)    │
                                 │     ▲                                     │
                                 │  SADRŽAJ (katalog, predlošci — podaci)    │
                                 └──────────────────────────────────────────┘
```

**Četiri nosive odluke:**

1. **Runde** — na kraju runde svima se istovremeno obračunaju posljedice.
2. **Pravila računaju brojke, AI samo prevodi** — AI nikad ne piše izravno u stanje.
3. **Dnevnik događaja je izvor istine** — stanje hotela se uvijek može ponovno izračunati, korak po korak.
4. **Rust za istinu, TypeScript za ekran** — veza je jedan API s automatski generiranim tipovima.

---
