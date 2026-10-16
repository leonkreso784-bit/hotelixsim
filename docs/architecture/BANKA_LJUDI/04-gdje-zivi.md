# BANKA_LJUDI §4 · Gdje živi i kako se koristi

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md).

## 4.1 Datoteke

```text
content/hotel/v2/ljudi/
├── banka.json              popis verzija: verzija N = koje su serije zapečaćene + otisak; piše samo glavna sesija
├── raspodjele.json         ciljevi iz §2 kao podatak (stroj uspoređuje banku s njima)
├── mjesta.json             mjesta za lokalno i regiju; zemlje za inozemstvo
├── zabranjeno.json         zabranjena imena, hoteli i riječi u opisu
├── povuceni.json           ID-jevi povučeni od verzije N (zapis se nikad ne briše)
├── radnici/serija-000.json … (piše samo agent RADNICI)
├── gosti/serija-000.json   … (piše samo agent GOSTI)
└── vanjski/serija-000.json … (piše samo agent VANJSKI; od K2v)
```

Serija je **jedna datoteka od najviše 50 zapisa** (~1 000 redaka formatiranog JSON-a, ~25 KB): stane u jedan pregled i
jedan prolaz agenta. Zaglavlje datoteke: `{"format": 1, "vrsta": "radnici", "serija": 0, "od": "R-00001", "do": "R-00050",
"zapisi": [...]}`; serija N ima ID-jeve 50·N+1 … 50·N+50 (K1). Za M3 to je 60 datoteka radnika, 30 gostiju i 12 vanjskih. Kod pravila (`crates/content/src/ljudi.rs`,
`ljudi_check.rs`) poštuje granicu od 1000 redaka (`check:kod`); JSON se ne broji, ali granica od 50 zapisa vrijedi.

## 4.2 Kako igra bira — deterministički po sjemenu

- **Ključ izbora** je isti kakav igra već ima: igra za vježbu i profili A/B/C → **sjeme profila** (isti hotel = isti
  ljudi, MODEL_V2 §3.3); razred i generirani hoteli → (sjeme igre, slot) ([PLATFORMA §4](../PLATFORMA/04-generirani-hoteli.md)).
- **Postupak:** iz verzije banke zapisane u igri uzmi zapise koji odgovaraju filtru (vrsta, uloga, rod, razina; za goste
  segment i dob), poredaj po ID-u, ispremiješaj vlastitim tokom kocke (`Rng::derive(sjeme, [label("ljudi:pocetni"), slot,
  uloga])`, novi tokovi, postojeći se ne diraju — MODEL_V2 §2 t. 3) i uzimaj redom **preskačući one koje igra već ima**.
  Hoteli se pune redom slota, pa nitko nije u dva hotela iste igre.
- **Burza** se izvodi pri otvaranju razdoblja iz stanja igre u tom trenutku (tok `ljudi:burza`), jer utjecaj među
  hotelima djeluje samo na prijelazu kruga (ADR-037). Nezaposleni kandidat ostaje na burzi 2–4 razdoblja [PROC] — manje
  potrošnje banke.
- **Gosti situacija** (🟨, zamjenjuje današnji izbor iz `names.json` u `crates/sim-hotel-v2/src/situations/fill.rs`):
  isti obrazac — imena slobodna u krugu, pa izbjegavanje ranijih, kao danas (`first_nonempty`).
- **Kad bazen presuši** (rijedak filtar): popušta se redom (razina → rod → dob/segment), a tek zatim se vraća lik koji je
  **napustio** tu igru. Ime se nikad ne slaže iz dijelova. Svaki korak popuštanja ima test.

## 4.3 Verzije i pečati — dodavanje ne mijenja stare igre ni balans

- Nova igra u snimak (`GameCreated`) zapisuje **`banka_ljudi: N`** i otisak te verzije. Stara igra se uvijek čita s
  verzijom s kojom je stvorena (isto načelo kao katalog u snimku, MODEL_V2 §6.2); igre bez tog polja koriste `names.json`
  kao danas.
- **Zapečaćena serija se ne mijenja.** Kad Leon odobri seriju, njen SHA-256 ide u `crates/content/tests/ljudi_sealed.rs`
  (obrazac `fixtures_sealed.rs`: konstanta, CRLF → LF, promjena je svjestan potez vidljiv u pregledu).
- **Ispravak** se radi povlačenjem: zapis ide u `povuceni.json` od verzije N, a ispravljeni lik dobiva novi ID u novoj
  seriji. Verzije prije N ga i dalje vide. Tako izbor u starim igrama ostaje bajt-isti.
- **Format** (polja) ima svoj broj. Novi format (npr. `padezi`) čita i stare serije; stara polja se ne preimenuju.
- **Balans** (`cargo run -p balance`) radi s verzijom banke iz `banka.json`, zadano posljednjom zapečaćenom za balans.
  Jer banka daje samo identitet, razinu i jezike, a raspodjele razine i jezika čuva stroj (§3.6), nova serija balans ne
  pomiče; podizanje zadane verzije ipak traži jedan `--check` kao dokaz.

## 4.4 Vanjski u datotekama i u igri

- **Datoteke:** `content/hotel/v2/ljudi/vanjski/serija-NNN.json`, zaglavlje `"vrsta": "vanjski"`, ID-jevi `V-…` po istom
  pravilu raspona; ciljevi u `raspodjele.json` pod novim ključem **`vanjski`** (`uloga`, `dob`, `rod_po_ulozi`); `mjesta.json`
  i `zabranjeno.json` su zajednički. Brana pečata (`ljudi_sealed.rs`) gleda i mapu `vanjski/`.
- **Rust (K2v):** tip `Vanjski` (polja §1.5, `deny_unknown_fields`), enum `UlogaVanjskog` (14 kodova), `Vrsta::Vanjski`,
  `Banka.vanjski`, `RaspodjeleVanjskih`; pravila u novom modulu `ljudi_vanjski.rs` (da `ljudi_check.rs` ostane ispod 600
  redaka). Igra banku i dalje ne čita.
- **Kako igra bira (K5, 🟨):** predložak ostaje u formatu `"from": "partner"`; uloga predloška dobiva neobvezno polje s kodom
  uloge iz banke (ime polja određuje K5), npr. P-INSPEKCIJA → `inspektor_sanitarni`. Izbor je isti obrazac kao za goste
  (§4.2): filtar (uloga, rod), poredak po ID-u, vlastiti tok kocke (npr. `label("ljudi:vanjski")`, novi tok, postojeći se ne
  diraju), najprije slobodni u krugu, zatim izbjegavanje ranijih primjeraka predloška (`first_nonempty` u `fill.rs`).
  Ponavljanje među hotelima je dopušteno 🟨 (§2.4). **Rod se ne popušta** (tekst je u rodu uloge); kad bazen (uloga, rod)
  presuši, vraća se lik iz ranijeg kruga istog hotela.
- **Dok K5 ne dođe**, igra uzima partnerska imena iz `names.json` kao danas; stare igre bez `banka_ljudi` i dalje (§4.3).
