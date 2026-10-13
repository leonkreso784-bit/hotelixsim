# ADR-026 — Put B: hotel od brojeva postaje skup stvari, u slojevima
**Datum:** 2026-10-10 · **Status:** 🟨 **ČEKA LEONOV PREGLED** (smjer je Leonov, pretpostavke Claudeove) ·
**Vezano:** [PROSIRENJA.md](../../product/PROSIRENJA.md), [MODEL_V2.md](../../architecture/MODEL_V2.md), [ADR-023](ADR-023-mentor-govori-ljudski-pripremljene-cinjenice-cetiri.md)

**Kontekst.** Leon 10.10.: igra treba biti *„kao BitLife — opsežno do bola"*: profili zaposlenika, sve sobe sa stanjem,
dijelovi hotela (spa, restoran, kafić, rooftop) sa zaposlenicima, zaradom i održavanjem, recenzije koje se čitaju, ADR.
Danas je hotel u jezgri skup brojeva (sobe = jedan broj, zaposleni = stalni + sezonci, „ostali prihod" = 30 % prihoda
od soba, recenzije = jedna ocjena). Ponuđena su dva puta: A — profili izvedeni iz zbroja (jeftino, student ne može
djelovati na pojedinca) i B — pravi entiteti u simulaciji. Leon je izabrao **B**.
**Odluka.**
1. **Put B u slojevima**, svaki se gradi, dokaže (testovi, balans, preglednik) i pregleda zasebno: 0 pokazatelji (ADR,
   RevPAR, GOPPAR), 1 odjeli po USALI-ju, 2 zaposlenici kao pojedinci, 3 sobe sa stanjem, 4 recenzije s tekstom.
2. **Detalj samo uz uzrok:** svaki novi podatak mijenja brojke, a objašnjenje dolazi iz jezgre (isti izvor kao mentor).
3. **Demo je siguran:** sve se gradi na granama; `main` ostaje igriv za demo 14.10. Stare igre zadržavaju svoj snimak
   i verziju jezgre.
4. **Pretpostavke (🟨):** ADR = prosječna cijena prodane sobe; odjeli ovise o hotelu (profil određuje ima li spa ili
   rooftop); zaposlenicima student upravlja (zaposli, otpusti, plaća, obuka, premještaj); recenzije iz banke rečenica
   po uzroku, AI samo povremeno; cilj je prezentacija profesoru nakon demoa.
5. **Arhitektura ([MODEL_V2.md](../../architecture/MODEL_V2.md) §11, t. 6–11, 🟨):** v1 se zamrzava, v2 je novi paket
   `sim-hotel-v2` birano poljem `model` u snimku pravila; v2 uvijek daje i v1 sažetak (indeksi, mentor, ekrani rade
   dalje); kocka raspoređuje, ne mijenja zbrojeve; nalozi su dio plana mjeseca; tekst recenzija se ne sprema.
   Procjena: 110–165 sati rada agenata za slojeve 0–4 — tjedni, ne dani; najmanji „wow" je sloj 0 + 1.
**Cijena ako krivo:** put B je najveća promjena modela od početka — tjedni rada ako se slojevi rade temeljito; ako
Leon odluči drukčiji redoslijed, mijenja se samo redoslijed slojeva, ne model.
