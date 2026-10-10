# ARHITEKTURA §18–§20 — odbačeno, otvoreno, redoslijed gradnje

Dio [ARHITEKTURA.md](../ARHITEKTURA.md)

## 18 · Razmotreno i odbačeno

| alternativa | zašto ne |
|---|---|
| **TypeScript svugdje** | Determinizam bi se mogao izgraditi i ovdje (`bigint`, isti dizajn kao §7.2), pa to nije razlog za odbacivanje. Razlozi su drugi: slabija ograda za kod koji piše AI (tipovi se brišu pri pokretanju, `any` propušta), sporije masovne simulacije i manje alata koji provjeravaju pravila jezgre umjesto čovjeka. |
| **Python (FastAPI)** | spor za masovne simulacije, slaba jamstva tipova, jezgra samo na poslužitelju |
| **Go** | slab za modeliranje domene (nema pravih `enum` tipova); jezgra samo na poslužitelju |
| **Elixir** | izvrstan za igre koje žive u vremenu, ali dinamički tipovi i treći jezik; za našu skalu Tokio + red poslova dovoljni |
| **C++** | nedefinirano ponašanje ugrožava K1; prevoditelj ne štiti od AI-pisanog koda; backend ekosustav nišni |
| **Kotlin / C#** | teži alati; C# vrijedi samo ako dođe Unity — a i tada jezgra ostaje u Rustu iza API-ja |
| **Sve u Rustu (Dioxus/Leptos)** | Rust sučelja još su mlada; rizik za proizvod |
| **Mikroservisi** | množe kvarove i trošak za jednog autora |
| **Supabase kao backend (RLS, Edge Functions)** | logika razbijena na SQL i funkcije, teško testirati, vezanost za pružatelja |
| **tRPC / GraphQL** | tRPC ne služi mobitelu i tuđim platformama; GraphQL je nepotrebna složenost |
| **Diesel / SeaORM** | SQLx je bliži SQL-u i jednostavniji za dnevnik događaja |
| **Redis / Kafka za poslove** | dodatna infrastruktura; Postgres je dovoljan |
| **Pohrana samo trenutnog stanja (bez dnevnika)** | ruši K1 i K3 — nema ponovnog obračuna ni dokaza |

---

## 19 · Otvorene odluke i pitanja za profesora

**Odluke (Leon):**

1. Pružatelj prijave (§12).
2. Gdje živi statično sučelje (§14).
3. AI pružatelj i modeli za demo (besplatni ključevi).
4. Kad stvarno treba mobilna aplikacija (mijenja hitnost Expa).

**Pitanja za profesora:**

1. Pojedinac ili timovi? Ako timovi — koliki i s kojim ulogama?
2. Koliko traje igra: broj rundi i trajanje runde (demo je mjesec; za pilot tjedan?).
3. Koliko studenata u grupi?
4. Koje stvarne hotelske pokazatelje želi da studenti vide i kako ih vrednuje (formule za indekse)?
5. Kako se relativna ocjena pretvara u ocjenu na kolegiju (ako uopće)?
6. Želi li biti suautor situacija i pravila?
7. Prijava preko AAI@EduHr — potrebna?
8. Koliko dugo se smiju čuvati podaci studenata?
9. Odakle stvarni podaci (sezonalnost, tipične brojke kvarnerskih hotela)?

---

## 20 · Okvirni redoslijed gradnje

Detalji idu u plan izgradnje nakon odobrenja ovog dokumenta.

1. **Kostur od kraja do kraja, lokalno:** workspace, CI, prazna jezgra → API → lokalni Postgres → jedan ekran.
   Dokazuje da su sve veze žive prije nego išta raste.
2. **Jezgra + alat za balans:** stanje, indeksi, mehanike, obračun, kocka — **bez baze i bez ekrana**. Prvi dokaz
   pravednosti A/B/C masovnim simulacijama.
3. **Dnevnik, runde i red poslova.**
4. **AI sloj** (presuda prvo, zatim situacije, likovi, mentor) s rezervama.
5. **Sučelje studenta** (mobitel).
6. **Profesorov dio** → **demo za profesora**.
7. **Isporuka na Railway** (jedan servis + Postgres) — tek na kraju, uz Leonov OK.
