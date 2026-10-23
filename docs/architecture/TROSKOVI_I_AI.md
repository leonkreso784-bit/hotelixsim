# Troškovi i AI — HotelixSim za fakultet

> Status: 🟨 **NACRT** · 24.10.2026. · Brojke su **[PROCJENA]** dok ih ne potvrdi **test troška po igri** (val 0 plana sesije 14,
> `crates/server/tests/ai_cost.rs`); nakon mjerenja tablice §3 i §4 zamjenjuju se izmjerenim brojkama.
> Vezano: [AI_TEMELJ](./AI_TEMELJ.md) (kako se trošak spušta), [ADR-065](../records/DECISIONS/ADR-065-leonove-odluke-24-10-tempo-profesora-lokal-generatori.md)
> t. 12–15, [ADR-066](../records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md) (pilot s profesorovim tempom),
> [ADR-063](../records/DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) (stropovi pilota).
> Podloge (sesija 14, 24.10.): `.superpowers/sdd/2026-10-14-popravak-temelja/` → `ai-trosak-istrazivanje-24-10.md`,
> `ai-cijene-provjera-24-10.md`, `besplatni-ai-fakultet-24-10.md`, `temelj-opterecenja-24-10.md`.
> Cijene su u američkim dolarima; **1 $ ≈ 0,87 €** (plan popravka temelja, odjeljak „Trošak"), bez PDV-a.

## 1. Ukratko

- Jedan student koji odigra cijelu tjednu sezonu (17 krugova, otprilike jedan semestar) danas troši **0,48–0,84 $ (0,42–0,73 €)** AI-ja;
  razred od 40 studenata **19–34 $ (17–30 €)** po semestru [PROCJENA iz izmjerenih poziva].
- S novim AI temeljem cilj je **≤ 0,25 $ po studentu** i **≤ 10 $ po razredu** ([AI_TEMELJ](./AI_TEMELJ.md) §3); najjeftiniji način
  (sve na manjem modelu) spušta razred na **2,5–4 $** po semestru.
- Poslužitelj i baza (Railway Hobby) koštaju **5–7 $ mjesečno (4,5–6 €)**, bez obzira igra li 1 ili 40 studenata.
- Trošak ne može pobjeći: aplikacija ima tvrdi strop **5 $ na dan i 30 $ na mjesec**, a pružatelj AI-ja još jedan (**40 $/mj**).
  Kad se strop dosegne, igra radi dalje po pravilima, bez AI-ja.
- U pilotu sve plaća **Leon Kreso sam**; poslije pilota trošak može preuzeti fakultet, projekt ili krediti (§6) — bez obećanja.
- Podaci: student igra pod **nadimkom**, bez e-adrese i računa. Tekst koji napiše ide AI-ju (Anthropic, obrada u SAD-u, briše se u
  30 dana, **ne koristi se za učenje modela**). Obrada u EU moguća je preko AWS-a (Frankfurt), oko 10 % skuplje.

## 2. Od čega se sastoji trošak

**Pojmovi u jednoj rečenici:**
- **API** — „utičnica" preko koje naš poslužitelj šalje tekst AI-ju i dobiva odgovor; plaća se po korištenju.
- **Token** — komadić teksta (otprilike pola do tri četvrtine riječi); AI se plaća po milijunu tokena koje primi (ulaz) i napiše (izlaz).
- **Predmemorija** — pružatelj pamti dio upita koji se ponavlja (npr. istu uputu za sve studente) i drugi put ga naplaćuje 10 % cijene.
- **Model** — „veći" AI (Sonnet) bolje sudi, ali je ~20× skuplji od „manjeg" (Haiku).

**AI ima tri uloge u igri:**

| uloga | što radi | model danas | tokena po pozivu (ulaz / izlaz) | $ po pozivu | poziva po tjednoj igri |
|---|---|---|---|---|---|
| **Lik** | gost ili zaposlenik s kojim student razgovara | Haiku 5.5 | 988 / 93 | 0,00015 | 102–306 (2–6 poruka po situaciji) |
| **Presuda** | ocjenjuje studentov pisani odgovor po rubrici | Sonnet 5.5 | 785 / 689 | 0,0085 | 17–51 (samo pisani odgovori) |
| **Mentor kruga** | sažetak i savjet nakon svakog kruga | Sonnet 5.5 (štedljivo) | 5 518 / 437 | 0,0154 | 17 |
| **Mentor sezone** | završni osvrt | Sonnet 5.5 | 6 136 / 534 | 0,0176 | 1 |

Izvor tokena: mjerenje na lokalnoj bazi ([PROVJERA_ARHITEKTURE_2026-10-17](./PROVJERA_ARHITEKTURE_2026-10-17.md) §2.4); cijene:
Anthropicov cjenik 24.10.2026. (Sonnet 5.5 2 $ ulaz / 10 $ izlaz po milijunu tokena, Haiku 5.5 0,10 / 0,50 $), isto kao tablica
`PRICES` u `crates/app/src/spend.rs`. Odgovor koji student **odabere** među ponuđenima ne zove AI (0 $).

**Ostalo:**
- **Hosting** (poslužitelj + baza, Railway Hobby, Amsterdam): 5 $/mj uključuje 5 $ potrošnje, ukupno **5–7 $/mj** (railway.com/pricing; plan pilota).
- **Kopije baze**: tjedni `pg_dump` na Leonovo računalo — **0 $** (Railway Hobby nema kopija diska, ADR-063 t. 15–16).
- Domena, nadzor, CI (GitHub): **0 $** (adresa `*.up.railway.app` je besplatna).

**Formula riječima:** trošak jedne igre = za svaku ulogu (broj poziva × (ulazni tokeni × cijena ulaza + izlazni tokeni × cijena izlaza))
+ ~10 % za ponovne pokušaje. Trošak razreda = trošak igre × broj studenata. Trošak semestra = trošak razreda + hosting × broj mjeseci.

## 3. Trošak po studentu, razredu i semestru [PROCJENA]

Tjedna sezona = 17 krugova ≈ jedan semestar; hosting za semestar = 5 mjeseci × 5–7 $ = **25–35 $ (22–30 €)** [PROCJENA].

| način | AI po studentu | AI razred (40) | AI + hosting, razred, semestar | ukupno po studentu |
|---|---|---|---|---|
| **A. Danas** — presuda i mentor Sonnet, lik Haiku | 0,48–0,84 $ (0,42–0,73 €) | 19–34 $ (17–30 €) | 44–69 $ (38–60 €) | 1,1–1,7 $ (0,95–1,5 €) |
| **B. AI temelj** — Haiku sudi, Sonnet samo granične (≤ 20 %), predmemorija za razred, mentor Sonnet štedljivo | 0,26–0,50 $ (0,23–0,44 €); **cilj ≤ 0,25 $** | 10–20 $ (9–17 €); cilj ≤ 10 $ | 35–55 $ (30–48 €) | 0,9–1,4 $ (0,8–1,2 €) |
| **C. Najjeftinije** — sve Haiku | 0,06–0,10 $ (0,05–0,09 €) | 2,5–4 $ (2,2–3,5 €) | 28–39 $ (24–34 €) | 0,7–1,0 $ (0,6–0,85 €) |

Kako je izračunato:
- **A**: mentor kruga 17 × 0,0154 = 0,26 $; presuda 17–51 × 0,0085 = 0,14–0,43 $; lik 0,02–0,05 $; mentor sezone 0,02 $; +10 %
  (`ai-trosak-istrazivanje-24-10.md` §1).
- **B**: Haiku presuda ≈ 0,00055 $ po pozivu (manji model ima ~30 % više tokena za isti tekst); Sonnet ponovo sudi do 20 % presuda;
  mentoru predmemorija upute i sažetiji podaci štede ~25–40 % → mentor 0,16–0,26 $ [PROCJENA]. **Mentor ostaje najveća stavka**, pa
  cilj ≤ 0,25 $ traži ili još štednje na mentoru ili mentora na Haikuu — to odlučuje test troška i slijepi pregled tekstova.
- **C**: `ai-trosak-istrazivanje-24-10.md` §3, kombinacija K1. Kvaliteta presude na Haikuu **nije izmjerena** (§10).
- **Mjesečna sezona** (4 kruga, kraća igra): 0,13–0,21 $ po studentu, 5–8 $ za razred od 40 u načinu A.
- Ako razred odigra cijelu sezonu u jednom mjesecu (profesorov tempo na satu, ADR-066), način A može doći do stropa 30 $/mj — upravo
  zato se gradi način B.

## 4. Razine 40 / 200 / 1 000 / 5 000 studenata [PROCJENA]

Izvor: `temelj-opterecenja-24-10.md` (dodatak); AI po semestru tamo pretpostavlja današnje modele (0,03–0,06 $ po studentu po krugu),
s AI temeljem bit će niži. Hosting za 40 je viši od pilota (5–7 $) jer uključuje rezervu za više radnika.

| studenata | AI / semestar | hosting / mj | što se mijenja u sustavu | razina kod Anthropica |
|---|---|---|---|---|
| **40** | 15–30 $ | 10–25 $ | više AI radnika po vrstama posla, rok na svakom poslu (AI_TEMELJ §6.1–6.2) | Start (1 000 zahtjeva/min, strop 500 $/mj) — velika rezerva |
| **200** | 80–160 $ | 30–60 $ | predmemorija igre, rjeđe osvježavanje stranice, predmemorija upita za razred, radnici kao zaseban proces | Start ili Build |
| **1 000** | 400–800 $ | 150–300 $ | odvojeni API i radnici, posrednik veza baze, raspodjela po fakultetu, drugi izvor AI-ja (npr. Bedrock) | Scale ili raspoređeno Build |
| **5 000** | 2 000–4 000 $ | 600–1 500 $ | upravljana baza s kopijom, više pružatelja, presuda u 5 min, skupna obrada mentora, možda drugi hosting | Custom (ugovor) |

Razine kod Anthropica (platform.claude.com/docs/en/api/rate-limits, 24.10.2026.): Evaluation (nove organizacije, niže) → Start → Build →
Scale → Custom; prelazak automatski po povijesti ili na zahtjev u konzoli. Više ključeva u istoj organizaciji **ne** diže granicu.

## 5. Kako je trošak pod nadzorom

1. **Dnevni i mjesečni strop u aplikaciji** — varijable `HOTELIXSIM_AI_STROP_DAN_USD` i `HOTELIXSIM_AI_STROP_MJESEC_USD`
   (`crates/server/src/config.rs`); u pilotu **5 $/dan i 30 $/mj** (ADR-063 t. 6). Na javnoj adresi s AI ključem poslužitelj se
   **ne pali** bez oba stropa. Svaki odgovor AI-ja upisuje se s cijenom u tablicu `ai_spend` (ADR-063 t. 9).
2. **Kad se tvrdi strop dosegne** — AI se gasi do sljedećeg dana ili mjeseca, a igra **radi dalje na pravilima**: presuda je neutralna
   po pravilu, lik odgovara kratko, mentor piše iz brojki. Student ne vidi grešku. Ako se potrošnja ne može pročitati, AI se također gasi
   (radije stati nego trošiti naslijepo, ADR-063 t. 12).
3. **Kanta po grupi** (AI temelj §6.6, gradi se) — svaka grupa dobiva unaprijed svoj dio mjesečnog stropa. Prazna kanta **nikad ne
   pretvara presudu u pravilo** (to bi mijenjalo ocjenu): gasi se samo drugi korak suca i skraćuje odgovor lika.
4. **Drugi strop kod pružatelja** — u Anthropicovoj konzoli 40 $/mj na zasebnom radnom prostoru (ADR-063 t. 5). Kad ga dosegne,
   pružatelj odbija pozive do 1. u mjesecu, a igra opet radi na pravilima.
5. **Hosting** — upozorenje potrošnje na Railwayu 7 $ i tvrdi strop 15 $ (prijedlog, čeka Leonovu potvrdu; ADR-063 D).
6. **Profesor vidi potrošnju** svoje grupe i upozorenje na 80 % kante, a uz svaku presudu tko je sudio: „AI", „AI (provjereno)" ili
   „pravilo (AI nije stigao)" (AI_TEMELJ §5).

## 6. Tko plaća

- **Pilot** (1 profesor, 20–40 studenata, nekoliko tjedana): **Leon Kreso sam** — hosting ~5 €/mj i AI do stropa 30 $/mj (~26 €).
- **Poslije pilota — mogućnosti, ništa od toga nije dogovoreno:**
  - fakultet plaća iz budžeta kolegija ili projekta (u načinu B ~30–48 € po razredu po semestru, s hostingom);
  - projekt (npr. Erasmus+, §8) u kojem je AI trošak stavka;
  - krediti pružatelja preko profesora ili istraživača (§8);
  - vlastiti model na sveučilišnim ili Srceovim računalima (nema troška po tokenu, ali netko ga održava);
  - startup krediti tek kad postoji tvrtka (Leon planira j.d.o.o.; danas nema obrta ni tvrtke).

## 7. Pružatelji i GDPR

**GDPR** je europska uredba o zaštiti osobnih podataka. **DPA** (ugovor o obradi podataka) je ugovor kojim se pružatelj obvezuje
kako smije postupati s našim podacima. Studentov napisani tekst smatramo osobnim podatkom.

| pružatelj | gdje se obrađuje | uči li na podacima | zadržavanje | DPA | cijena ($ / milijun tokena, ulaz / izlaz) | hrvatski |
|---|---|---|---|---|---|---|
| **Anthropic izravno** (pilot) | SAD / globalno; **EU opcije nema** | ne, bez izričitog dopuštenja | briše se u 30 dana | dio komercijalnih uvjeta [nije ponovo čitano 24.10.] | Haiku 0,10 / 0,50; Sonnet 2 / 10 | [neizmjereno — mjeri se našim mjerilom] |
| **AWS Bedrock EU** (isti Claude) | EU: Frankfurt, Stockholm, Milano, Španjolska, Irska, Pariz | pružatelj modela ne vidi upite | po AWS-u | AWS GDPR DPA | ~+10 %: Haiku ≈ 0,11 / 0,55 [točan cjenik nepročitan]; bez skupne obrade | kao Anthropic |
| **Google Vertex EU** (Gemini) | EU regija | plaćena razina: ne | [nepročitano] | Google Cloud DPA [nepročitano] | Gemini 2.5 Flash-Lite 0,10 / 0,40 | [neizmjereno] |
| **OpenAI** | EU (EGP + CH) samo preko prodaje i uz nulto zadržavanje | ne | zapisi nadzora do 30 dana (izvan EU dogovora) | [nepročitano] | gpt-5-nano 0,05 / 0,40; gpt-5-mini 0,25 / 2,00; EU +10 % | [neizmjereno] |
| **Azure OpenAI EU** | države EU (Data Zone EU) | ne; ne vidi ni OpenAI | ljudski pregled samo označenog, osoblje u EGP-u | Microsoft DPA | gpt-5-mini ~0,275 / 2,20 [neslužbeni izvor] | [neizmjereno] |
| **Mistral** (Francuska) | DPA ne navodi lokaciju | **da, dok se ručno ne isključi** | trajanje ugovora, brisanje 30 dana nakon raskida | da (27.07.2026.) | Small 4 ~0,15 / 0,60 [neslužbeni izvor]; Large 0,5 / 1,5 | [neizmjereno; procjena: Small slabiji] |
| **Srce / vlastiti model** | Hrvatska | ne | sami odlučujemo | sveučilište / Srce | bez troška po tokenu; održavanje | [neizmjereno; procjena: slabiji od Claudea] |

**Zašto besplatne razine otpadaju:** Gemini besplatna razina koristi unos „za poboljšanje proizvoda" i uključuje ljudske pregledavače, a
za korisnike u EGP-u uvjeti traže plaćene usluge; Mistral uči na podacima dok se ne isključi. Studentski tekst ne smije ići pružatelju
koji na njemu uči. Anthropic i OpenAI API nemaju besplatnu razinu.

**Drugi pružatelj nije besplatan za nas:** novi priključak u kodu stoji **12–20 sati** rada + 3–5 sati mjerenja hrvatskog, i novi DPA
za fakultet. Zato pilot ide samo preko Anthropica; Bedrock EU je spreman kao postavka kad fakultet zatraži obradu u EU (AI_TEMELJ §6.7).

**Što student upisuje:** nadimak (ne pravo ime), bez e-adrese i lozinke; stranica je zaštićena jednom zajedničkom lozinkom, a igra se
pamti kolačićem 120 dana (ADR-063 t. 1, 5). AI dobiva samo tekst situacije i studentov odgovor ili poruku. Obavijest o privatnosti
(nadimci, odgovori idu AI-ju u SAD) piše se prije pilota i čeka Leonov tekst i kontakt (ADR-063 D).

**Kvaliteta hrvatskog:** javnog mjerila za ono što nama treba (pisanje s ispravnim rodom i padežima, ocjena po rubrici) **nema**;
postojeća mjerila (CLASSLA, COPA-HR) mjere razumijevanje. Jedini sud je naše mjerilo od 40 odgovora + Leonov slijepi pregled.

## 8. Za prezentaciju

**Točke za slajdove:**
1. **Cijena po studentu:** AI za cijeli semestar ~0,25–0,50 € po studentu (AI temelj); danas do 0,73 €. S hostingom ~1 € po studentu
   za razred od 40 [PROCJENA, potvrđuje test troška].
2. **Usporedba:** i danas je AI za cijeli semestar **manje od jedne kave po studentu** (kava u Hrvatskoj ~1,5–2 € [PROCJENA, nije iz
   izvora]; AI ≤ 0,73 €). S hostingom razred od 40 je otprilike jedna kava po studentu, a s više studenata hosting se dijeli.
3. **Trošak ne može pobjeći:** dva stropa (aplikacija 30 $/mj, pružatelj 40 $/mj); kad se dosegne, igra radi dalje po pravilima;
   profesor vidi potrošnju svoje grupe.
4. **GDPR:** nadimci, bez računa; AI ne uči na tekstu studenata, briše ga u 30 dana; obrada u EU moguća (+10 %) kad je fakultet zatraži.
5. **Mistral:** europski, ali nije besplatan put — besplatni način uči na podacima dok se ne isključi, a od kolovoza 2026. API je samo
   za tvrtke; kao plaćeni pružatelj cijenom je blizu Haikua, uz opt-out i potpisan DPA.
6. **Putovi do besplatnog AI-ja, po izglednosti:**
   1. **Srce** — već ima vlastiti AI poslužitelj (vLLM) na svojim grafičkim karticama; besplatno za akademsku zajednicu, podaci ostaju
      u Hrvatskoj. *Prvi korak:* e-mail fakulteta na **computing@srce.hr** s upitom za pristup za nastavu. (Uz to: Sveučilište u
      Rijeci ima računalo Bura s grafičkim karticama, *cnrm@uniri.hr*.)
   2. **Krediti preko profesora** — Google Cloud teaching credits (do 50 $ po studentu, prijavljuje nastavnik za kolegij), OpenAI
      Researcher Access (do 1 000 $, tema društvenih učinaka AI-a), Gemini Academic Program. *Prvi korak:* profesor kolegija predaje
      prijavu na edu.google.com/programs/credits/teaching.
   3. **AWS Cloud Credit for Research** — isti Claude u EU (Frankfurt), profesori bez gornje granice; pregled 90–120 dana. *Prvi korak:*
      prijedlog projekta (HotelixSim kao istraživanje učenja uz AI) **u studenom**, da stigne za ljetni semestar.
   4. **Erasmus+ KA220-HED** — 120 000–400 000 € paušal, najmanje 3 ustanove iz 3 zemlje; rok ~veljača–ožujak 2027. *Prvi korak:*
      fakultet traži dva partnera.
   5. **Startup krediti** (Anthropic 1 000 $, AWS Activate 1 000 $) — tek kad Leon otvori j.d.o.o. (traže tvrtku i njezinu domenu).
7. **Hrvatska nema europsku „AI tvornicu"** (jedina članica EU bez nje); EuroHPC i slovenski SLAIF nude sate na superračunalu za
   istraživanje, ne stalnu uslugu za igru.

**Tri pitanja koja Leon postavlja fakultetu:**
1. Može li fakultet nakon pilota preuzeti trošak od ~30–50 € po razredu po semestru (AI + hosting) — iz kojeg budžeta i preko kojeg računa?
2. Je li za fakultet dovoljna obrada u SAD-u bez učenja i s brisanjem u 30 dana, ili tražite obradu u EU (+10 %)? Tko je vaš službenik
   za zaštitu podataka?
3. Hoće li fakultet ili profesor poslati upit Srcu i podnijeti prijavu za kredite (Google teaching, AWS Research) u ime kolegija?

## 9. Izvori

Svi čitani **24.10.2026.**, osim gdje piše drukčije.
- Anthropic cjenik: platform.claude.com/docs/en/about-claude/pricing
- Anthropic granice brzine i razine: platform.claude.com/docs/en/api/rate-limits
- Anthropic predmemorija (prag 512 tokena): platform.claude.com/docs/en/build-with-claude/prompt-caching
- Anthropic lokacija obrade: platform.claude.com/docs/en/manage-claude/data-residency
- Anthropic zadržavanje i učenje: platform.claude.com/docs/en/manage-claude/api-and-data-retention; privacy.claude.com/en/articles/7996866 (članak 01.07.2026.)
- Anthropic komercijalni uvjeti: anthropic.com/legal/commercial-terms (nije ponovo otvoreno 24.10.)
- AWS Bedrock zaštita podataka: docs.aws.amazon.com/bedrock/latest/userguide/data-protection.html; GDPR: aws.amazon.com/compliance/gdpr-center
- Google Gemini cjenik: ai.google.dev/gemini-api/docs/pricing (ažurirano 22.10.2026.); uvjeti: ai.google.dev/gemini-api/terms
- Google Vertex lokacija: docs.cloud.google.com/vertex-ai/generative-ai/docs/learn/data-residency (ažurirano 22.10.2026.)
- OpenAI cjenik: developers.openai.com/api/docs/pricing; podaci i EU: developers.openai.com/api/docs/guides/your-data
- Azure OpenAI privatnost: learn.microsoft.com (Azure OpenAI data privacy, ažurirano 05.06.2026.)
- Mistral cjenik: mistral.ai/pricing; DPA: legal.mistral.ai/terms/data-processing-addendum (na snazi 27.07.2026.); učenje: help.mistral.ai/en/articles/455207 (03.09.2026.)
- Mistral samo za tvrtke: conductatlas.com/change/2026-08-08-mistral-ai-mistral-ai-terms-of-service-4261 [neslužbeni izvor]
- Railway: railway.com/pricing; regije: docs.railway.com/reference/deployment-regions; DPA: railway.com/legal/dpa
- Srce: dei.srce.hr/2026/predavanja; wiki.srce.hr/x/TLx3C · UNIRI Bura: cnrm.uniri.hr/bura
- Google teaching credits: edu.google.com/programs/credits/teaching · Gemini Academic: ai.google.dev/gemini-api/docs/gemini-for-research (23.09.2026.)
- OpenAI Researcher Access: help.openai.com/en/articles/10139500
- AWS Cloud Credit for Research: pages.awscloud.com/aws-cloud-credit-for-research.html
- Anthropic AI for Science: support.claude.com/en/articles/11199177 (16.03.2026.) · Claude for Startups: claude.com/programs/startups
- EuroHPC AI Factories: eurohpc-ju.europa.eu/ai-factories/ai-factories-access-modes_en · HR bez AI tvornice: en.lider.media (22.03.2026.)
- Erasmus+ rokovi: teacheracademy.eu/funding/funding-deadlines [neslužbeni izvor]
- Mjerila hrvatskog: llm-benchmarks-classla.streamlit.app; huggingface.co/datasets/classla/copa_hr
- Izmjereni tokeni po pozivu: [PROVJERA_ARHITEKTURE_2026-10-17](./PROVJERA_ARHITEKTURE_2026-10-17.md) §2.4 (17.10.2026.)

## 10. Što još nije izmjereno

1. **Test troška po igri** (`crates/server/tests/ai_cost.rs`, val 0) — stvarni $ po tjednoj i mjesečnoj igri; zamjenjuje sve [PROCJENA] u §3–4.
2. **Kvaliteta presude na Haikuu** i dvokoraka Haiku → Sonnet: mjerilo od 40 odgovora čeka Leonove ocjene; do tada je referenca Sonnet.
3. **Kvaliteta mentora na Haikuu** (slijepi pregled Leona) — odlučuje može li se doći do cilja ≤ 0,25 $.
4. **Pogađa li se predmemorija** (stvarni broj pročitanih tokena iz predmemorije) i koliko razredni prefiks stvarno štedi.
5. Stvarni broj tokena Haikua (procjena +30 %) na našim tekstovima.
6. Stvarni mjesečni račun Railwaya pod razredom od 40 (procjena 5–7 $ za pilot, 10–25 $ s više radnika).
7. Brojke razine Evaluation kod Anthropica (ne objavljuju se) — prije pilota provjeriti u konzoli.
8. Ponovo pročitati DPA Anthropica, Googleove uvjete za EGP i točan Bedrock cjenik.
9. Hrvatski kod drugih pružatelja (Gemini, OpenAI, Mistral, otvoreni modeli na Srcu) — samo našim mjerilom, ako fakultet izabere taj put.
10. AI temelj ([AI_TEMELJ](./AI_TEMELJ.md)) još nije izgrađen; način B je cilj, ne današnje stanje.
