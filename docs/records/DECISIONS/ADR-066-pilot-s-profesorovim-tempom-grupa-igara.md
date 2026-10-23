# ADR-066 — Pilot s profesorovim tempom: grupa igara, profesorski kod
**Datum:** 2026-10-24 · **Status:** ✅ **Leonove odluke 24.10.** (t. 1–3), 🟨 t. 4–13 ČEKA LEONOV PREGLED ·
**Vezano:** [ADR-065](ADR-065-leonove-odluke-24-10-tempo-profesora-lokal-generatori.md) t. 1 i 8 (tempo, opseg pilota), [ADR-037](ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) (razred),
[ADR-063](ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) (pilot), [AI_TEMELJ](../../architecture/AI_TEMELJ.md), [TROSKOVI_I_AI](../../architecture/TROSKOVI_I_AI.md)

**Povod.** ADR-065 t. 8 ostavio je opseg pilota otvorenim. Sesija 14 je analizirala oba puta (dva read-only agenta:
`.superpowers/sdd/2026-10-14-popravak-temelja/opseg-pilota-docs-24-10.md` i `opseg-pilota-kod-24-10.md`). Nalaz: tempo **ne traži razred
u jezgri**. Zatvaranje kruga ne provjerava tko zatvara (`crates/game/src/command.rs:272`), a obračun sam otvara sljedeći krug
(`crates/game/src/resolve.rs:249-251`), pa grupa živi u `app`/`api`/`store`/web, a jezgra i `CORE_VERSION` ostaju isti. Cijena je ~25–34 h koda,
uz više AI radnika. Puni razred (ADR-037, ~159 h) ostaje za kasnije.

## A — Leonove odluke (anketa i tekst, 24.10.)
1. **Opseg pilota: (a)** — profesorov tempo ide u pilot, i to kao **grupa igara**: svaki student ima svoju igru, a profesor vodi tempo grupe
   (krug gumbom ili po vremenu; student automatski u novom krugu ili sam „dalje").
2. **Profesor dolazi na stranicu i dobije poseban kod**: sam napravi grupu i dobije **profesorski kod**, kojim kasnije ulazi i upravlja
   grupom. Studenti ulaze **kodom za ulaz** u grupu. Bez računa i e-adrese u pilotu; AAI@EduHr kasnije.
3. **Cijena na prezentaciju:** sve o trošku AI-ja (po studentu, po razredu, razine), **Mistral** i **putovi do besplatnog ili gotovo
   besplatnog AI-ja za fakultet** idu u prezentaciju fakultetu (podloga [TROSKOVI_I_AI](../../architecture/TROSKOVI_I_AI.md)).

## B — Pretpostavke voditelja 🟨
4. 🟨 Grupa je u bazi **isti pojam kao budući razred** (`classes`, bez `game_id`, s popisom igara), da se za ADR-037 ne prepisuje.
5. 🟨 Profesorski kod: 12 znakova bez sličnih slova (0/O, 1/l), u bazi samo sažetak (kao kolačić gosta). Kod za ulaz: 6 znakova.
   Izgubljen profesorski kod: Leon izdaje novi naredbom na poslužitelju. Profesor smije imati više grupa.
6. 🟨 Sve igre jedne grupe imaju **zajedničko sjeme**: isti početni hotel i isti prvi krug. Od drugog kruga situacije ovise o stanju
   svakog hotela (`crates/sim-hotel-v2/src/situations/draw.rs:202-237`), pa se razilaze — potpuno isti popis za cijelu grupu traži promjenu
   jezgre (+4–6 h; pitanje Leonu uz plan). Generator događaja (ADR-065 t. 6) mijenja početak **između grupa** i u igrama za vježbu.
7. 🟨 Igra u grupi ne broji se u tri igre za vježbu (ADR-063 t. 13); jedan gost ima najviše jednu igru po grupi.
8. 🟨 Zadano: krug **ručno** (gumb), ulazak **automatski**. Profesor smije promijeniti tempo usred igre, a promjena vrijedi od sljedećeg kruga.
9. 🟨 Kad se krug zatvori sam, neodgovorene situacije idu kao danas (kvaliteta 1, mentor upozori; ADR-037 t. 6). Web pokazuje
   **odbrojavanje** i upozorava minutu prije; zadnji spremljeni plan vrijedi.
10. 🟨 Način „dalje": nema roka za neaktivne u pilotu. Zaostali smije sustići (ADR-037 t. 2), a profesor u pregledu vidi tko kasni.
11. 🟨 Krug na satu je isti krug igre (tjedan ili mjesec), samo s kratkim razmakom (npr. 5 min). D11 ostaje otvoren za razred.
12. 🟨 Više AI radnika i rok na poslu (AI_TEMELJ §6.1–6.2) su **preduvjet** za grupu s automatskim ulaskom i idu prije pilota.
13. 🟨 Deploy samo kad nijedna grupa nema krug u obračunu. Jezgra se za trajanje pilota zamrzava (ADR-063).

## Što ovo mijenja u radu
- Plan sesije 14 (jedan plan, ADR-064) ima val za grupu igara (store → app → api → web), poslije vala 0 testova i AI temelja.
- PLATFORMA faza 1 (F1-2 … F1-17) dobiva najmanju izvedbu za pilot. Ostatak razreda (ADR-037) ide nakon pilota.
