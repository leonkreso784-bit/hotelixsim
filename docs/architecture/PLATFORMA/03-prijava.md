# PLATFORMA §3 · Prijava: mogućnosti, usporedba i redoslijed

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). **Odluka: [ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)
t. 7.** Cilj je AAI@EduHr za profesore i studente; **prije toga** vlastiti računi (korisničko ime + lozinka); identitet je
odvojen od korisnika pa se AAI kasnije veže na isti račun. Kako se profesor prijavljuje za prvu probu (D4) je otvoreno.

## 3.1 Usporedba mogućnosti

| način | sigurnost | posao za nas | trošak | iskustvo studenta | GDPR |
|---|---|---|---|---|---|
| **A. Korisničko ime + lozinka koje izdaje profesor / sustav** (odabrano za prvi val) | dobra uz argon2, ograničenje pokušaja po IP-u i računu, reset; lozinka se čuva samo kao otisak | srednji: prijava, promjena i reset lozinke, generator računa iz popisa studenata | 0 € | dobro: prva prijava prisiljava na promjenu lozinke | ime i otisak lozinke; bez e-pošte |
| **B. Kod igre samo za ulazak** (uz A) | kod je za ulazak u razred, ne identitet; ograničenje pokušaja | mali: `class_players` | 0 € | 10 sekundi, uz prijavu | ništa novo |
| **C. Jednokratna poveznica koju izdaje admin** (za profesora, preporuka za D4) | dobra: vrijedi 24 h, jednom | mali: CLI `server admin add-professor` | 0 € | profesor klikne i postavi lozinku | e-adresa profesora |
| **D. AAI@EduHr** (SAML 2.0; OIDC [PROVJERITI]) | najbolja: fakultetski identitet, fakultet gasi račun kad student ode | srednji tehnički, ali **dugo administrativno**: fakultet mora registrirati uslugu kod Srca | 0 € kod Srca [PROVJERITI] | najbolje za fakultet: „prijavi se kao na Merlin" | atribute daje fakultet; traži ugovor/suglasnost |
| **E. Google / Microsoft OIDC** | dobra (MFA kod pružatelja) | mali do srednji: jedan OIDC klijent u Rustu (`openidconnect`), callback, PKCE | 0 € | dobro, ako student ima račun [PROVJERITI za UNIRI] | adresa ide preko američkih pružatelja |
| **F. Vanjski servis** (Supabase Auth, Clerk, Auth0, Zitadel, WorkOS) | dobra, održava je netko drugi | mali za standardne tokove | Supabase SAML SSO od plana Pro; Clerk/Auth0 besplatno do kvote [PROC]; WorkOS po SSO vezi [PROC] | ovisi o ekranima | izvršitelj obrade više (DPA) |
| **G. Sve vlastito u Rustu** (A + E, a D preko SAML biblioteke) | dobra ako je pažljivo napisano; SAML u Rustu je nezreo | velik za SAML, mali za ostalo | 0 € | isto | sve kod nas |

Magična poveznica e-poštom i „kod + nadimak + ključ za povratak" iz prijašnjeg nacrta (12.10.) odbačeni su: razred
traži stalan identitet i brisanje po osobi (dio 07), a ADR-037 t. 7 uvodi račune.

**Napomena o [ARHITEKTURA §12](../ARHITEKTURA/09-podaci-ai-api-prava.md).** Ondje piše „prijavu ne pišemo sami, backend samo
provjerava JWT". Ispravak: **sesije su naše** (neproziran token u `httpOnly` kolačiću, otisak u bazi, kao danas za gosta),
a **vanjski pružatelj samo potvrdi identitet** pri prijavi. JWT u pregledniku ne treba. Razlozi: naša sesija se može
opozvati jednim `DELETE`, token nikad nije u JavaScriptu, a isti mehanizam već radi i testiran je. Ovo treba upisati kao ADR.

## 3.2 Što se gradi, redom (cigle iz ROADMAP/platforma)

1. **ID-1** · prijava korisničko ime + lozinka (argon2/bcrypt otisak, promjena lozinke, ograničenje pokušaja: 11. pogrešan
   pokušaj dobiva 429; lozinka nikad u dnevniku ni u odgovoru).
2. **ID-2** · generator računa: profesor unese popis studenata, sustav izda korisnička imena i početne lozinke
   (jednokratni prikaz ili CSV), prisilna promjena pri prvoj prijavi; profesor može resetirati lozinku (reset ruši staru sesiju).
3. **F1-4** · ulaz za profesora: admin izdaje jednokratnu poveznicu (D4 otvoreno; ovo je preporuka).
4. **F1-6** · ulaz kodom igre: prijavljeni student upisuje kod, nastaje `class_players`. Isti račun na drugom uređaju vidi
   isti hotel, pa „ključ za povratak" više nije identitet.
5. **AAI-0** · istraživanje bez koda: pitanja fakultetu i Srcu (tko je IdP, SAML ili OIDC, atributi, rok). Pokreće se **sada**.
6. **Poslije (F5)** · generički OIDC, Google/Microsoft za profesora, Supabase Auth kao posrednik za SAML, povezivanje
   na isti `user_id` (povijest ostaje), TOTP za admina.

**Sigurnosni detalji za kod igre:** 6–7 znakova bez sličnih znakova (0/O, 1/I); vrijedi samo dok je upis otvoren;
ograničenje npr. 10 pokušaja po IP-u u minuti [PROC]. Ime se prikazuje po odluci D8 (imena ili nadimci; pitanje F6).

## 3.3 AAI@EduHr: kako se dobiva pristup

AAI@EduHr je sustav elektroničkih identiteta hrvatskog obrazovanja, a vodi ga **Srce**. Usluga („resurs") se mora
registrirati, a to u pravilu pokreće ustanova (FMTU ili Sveučilište u Rijeci) uz suglasnost i kontakt osobu
[PROVJERITI točan postupak na aai.hr]. Postoji testna federacija za razvoj, AAI@EduHr Lab [PROVJERITI]. Atributi koje
tipično dobivamo [PROVJERITI]: trajni pseudonimni ID (`hrEduPersonUniqueID`), ime i prezime, e-adresa, povezanost s
ustanovom (student ili djelatnik), ustanova. Iz povezanosti se uloga može dodijeliti sama.

**Tehnički put:** ako Srce nudi OIDC, to je isti klijent kao za Google i Microsoft, samo s drugom postavkom. Ako nudi
samo SAML, posrednik prevodi SAML u OIDC: Supabase Auth (Pro), Zitadel ili Keycloak. Naš Rust tada ostaje čisti OIDC
klijent. **Rok:** administrativni dio može trajati tjednima, pa ga treba pokrenuti rano (pitanje F1), a graditi kasno.

## 3.4 Supabase Auth kao posrednik (samo prijava, kasnije)

Supabase se u ovom projektu više ne koristi za bazu (dio 06). Kao **posrednik identiteta** ostaje opcija za AAI@EduHr:
podržava PKCE tok s razmjenom koda na poslužitelju. Pružatelj vraća preglednik na naš `/v1/auth/callback?code=…`, naš Rust
razmijeni kod, provjeri korisnika i postavi **vlastiti** `httpOnly` kolačić; Supabase token nikad ne dolazi u JavaScript.
Uvjeti: koristi se samo Auth (ne Data API ni RLS iz preglednika); naša tablica `identities` drži `(provider, subject)`
pa se posrednik može zamijeniti bez gubitka računa; SAML SSO traži plan Pro, a Leon ga već plaća u postojećoj organizaciji.
Latencija prijave nije bitna za igru.
