# Hotelski pokazatelji, tipične vrijednosti i uzročne veze

> Istraživanje 2026-10-04 (agent, web). Podloga za stvarni sloj hotela u igri (DEMO.md).
> Oznake: **[P]** provjereno u izvoru · **[S]** sekundarni izvor (prikaz izvora) · **[PROC]** procjena.

## 1 · Formule (USALI/STR standard, preko glosara) [S]

| pokazatelj | formula |
|---|---|
| Popunjenost | prodane sobe / raspoložive sobe |
| ADR | prihod od soba / prodane sobe |
| RevPAR | prihod od soba / raspoložive sobe = ADR × popunjenost |
| TRevPAR | ukupni prihod / raspoložive sobe |
| GOP, GOPPAR | GOP = prihod − odjelni i neraspoređeni operativni troškovi (prije najma, amortizacije, kamata); GOPPAR = GOP / raspoložive sobe; GOP marža = GOP / ukupni prihod |
| CPOR | troškovi odjela soba / prodane sobe |
| Udio troška rada | trošak rada / ukupni prihod |
| Fluktuacija | odlasci u razdoblju / prosječan broj zaposlenih × 100 |
| MPI / ARI / RGI | vlastita popunjenost / ADR / RevPAR ÷ isto za konkurentski set × 100; RGI = MPI × ARI / 100 |
| NPS | % promotora (9–10) − % kritičara (0–6) |

U igri (sloj 0) „raspoložive sobe" su **sve sobe × dani**: sobe izvan funkcije (out of order) ubrajaju se u ponudu
(USALI), pa vrijedi RevPAR = ADR × popunjenost; njihove noći prikazuju se zasebno kao izgubljene noći.

Indeks zadovoljstva gostiju nema standardnu formulu (ReviewPro GRI je vlasnički), pa ga igra definira sama.

## 2 · Tipične vrijednosti

**Popunjenost hotela, cijela Hrvatska, 2025., neto (DZS) [P]** — https://podaci.dzs.hr (Turizam, komercijalni smještaj 2025.)

| mj | I | II | III | IV | V | VI | VII | VIII | IX | X | XI | XII |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| % | 25,4 | 34,2 | 35,1 | 48,8 | 57,0 | 72,0 | 78,4 | 83,2 | 73,4 | 54,6 | — | 31,5 |

Studeni nije dohvaćen. 2024. je gotovo isto. Kvarner je vjerojatno jači zimi i u kasnoj sezoni [PROC].

**Liburnia Riviera Hoteli (Opatija, 3–5*), 2024. [P]** — ZSE, https://eho.zse.hr/fileadmin/issuers/LRH/
- Popunjenost 67,0 %, ADR 132 EUR (uključuje pansion), RevPAR 89 EUR.
- Prihod od prodaje 60,5 mil. EUR; prilagođena EBITDA marža 23,0 % (EBITDA nije GOP; GOP je viši).
- Trošak osoblja 22,1 mil. EUR ≈ 35 % prihoda; 835 zaposlenih, u vrhuncu >1 000.

**Ostalo [S]**
- Udio troška rada, 13 listanih turističkih tvrtki: 37,3 % (H1 2025), 38,8 % (H1 2026) — total-croatia-news.com/?p=198503
- Rast troška rada u HR 11–15 % godišnje (2022–2025), a cijene smještaja +1,9 % (2024) — HUT, en.hrturizam.hr
- Sezonci čine ~70 % radne snage u dijelu tvrtki; u jednom kvarnerskom hotelu vraća se 80 % sezonaca — croatiaweek.com, total-croatia-news.com
- Europa, lipanj 2019. (HotStats): GOP marža ~44,6 %, plaće 27,5 % prihoda — hospitalitynet.org/news/4094521.html
- Prosječni hotelski NPS 44 (QuestionPro, Q1 2025).

**Nema javnih podataka:** ADR/RevPAR za 3–4* na Kvarneru po mjesecima, fluktuacija sezonaca u HR, GOP marža HR hotela, CPOR za HR.
Za igru [PROC]: ADR po mjesecima iz LRH prosjeka 132 EUR i sezonskog indeksa; fluktuacija sezonaca 30–50 %, stalnih 15–25 %.

## 3 · Uzročne veze koje se smiju modelirati

| veza | jačina | status |
|---|---|---|
| Ocjena → cijena/potražnja | +1 bod GRI (0–100) → ADR +0,89 %, popunjenost +0,54 %, RevPAR +1,42 % (Cornell, Anderson 2012; korelacija) — hospitalitynet.org/news/4058547.html | [P] |
| Ocjena → cijena (meta-analiza) | medijan elastičnosti 0,851; najjače podocjene „vrijednost" i „soba" — PolyU | [S] |
| Cijena → potražnja | elastičnost −0,14 prosjek (−0,11 do −0,31 po segmentu), urbani hoteli SAD 1989–2000 (Canina i Carvell, Cornell) | [P] |
| Cijena konkurenata → potražnja | križna elastičnost +0,32 do +0,44 (niži segmenti) | [P] |
| Zaposlenici → gosti | service-profit chain (Heskett 1994): smjer potvrđen, koeficijenti ne | [S] smjer, [PROC] jačina |
| Zadovoljstvo zaposlenika → fluktuacija | Hinkin/Holtom/Liu (Cornell, 5 270 zaposlenih) | [S] |
| Fluktuacija → trošak | ~5 900 USD do 7 500–10 000 EUR po zaposleniku; 2–4 mj. do pune produktivnosti | [S] |
| Održavanje → kvarovi → recenzije | nema kvantificirane studije | [PROC] |
| Popunjenost → opterećenje osoblja | nema kvantificirane studije; logično linearno | [PROC] |

## Napomene za dizajn

- Elastičnost −0,14 je stara, američka i urbana. Za turistički 4* hotel uz jaku konkurenciju koristiti −0,3 do −1,0 [PROC]
  i u dokumentaciji ga označiti kao **kalibracijsku varijablu**.
- Najčvršća hrvatska podloga: DZS (popunjenost po mjesecima) i LRH 2024. (jedan operater u Opatiji). Ostalo je strano,
  sekundarno ili procjena.
