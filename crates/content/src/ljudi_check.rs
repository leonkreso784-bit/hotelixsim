//! Pravila banke ljudi (BANKA_LJUDI §3, kartica K1): oblik iznad serdea (O), jedinstvenost (J), dosljednost (D),
//! tekst (T) i zabranjeno (Z). Okvir (R1–R3) je u `ljudi_check_okvir`, stereotip i raspodjele u `ljudi_raspodjele`,
//! usporedba teksta u `ljudi_tekst`, jedinstvenost i učestalost imena u `ljudi_imena`.
//!
//! Svaki prekršaj je jedan redak `"{put} {id}: {polje}: {poruka}"`; provjera skuplja sve i vraća jedan
//! `ContentError::Invalid`, poredano po putu i ID-u, da autor serije vidi sve odjednom. Duljine su u znakovima.
//! Pragovi [PROC] su konstante ovdje (promjena praga je vidljiva u pregledu koda), 🟨 pretpostavke kartice K1.

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use crate::ContentError;
use crate::ljudi::{Banka, Gost, Okvir, Podrijetlo, Radnik, Razina, Serija, Uloga, Vrsta};
use crate::ljudi_tekst::{cirilica, latinsko_slovo, mnozina, rijeci, sadrzi};

/// Ime i prezime: 2–30 znakova, latinica s dijakriticima, razmak, `-`, `'` (§1.2).
pub(crate) const IME_ZNAKOVA: RangeInclusive<usize> = 2..=30;
/// Slobodno mjesto (gost, radnik iz inozemstva) i zanimanje: 2–40 znakova 🟨.
pub(crate) const MJESTO_ZNAKOVA: RangeInclusive<usize> = 2..=40;
pub(crate) const ZANIMANJE_ZNAKOVA: RangeInclusive<usize> = 2..=40;
/// Opis: 40–220 znakova, najviše dvije rečenice (ADR-042 t. 8, odgovor 4).
pub(crate) const OPIS_ZNAKOVA: RangeInclusive<usize> = 40..=220;
pub(crate) const OPIS_RECENICA: usize = 2;
/// Najviše zapisa u jednoj seriji; ujedno širina raspona ID-jeva serije (50·N+1 … 50·N+50).
pub(crate) const SERIJA_NAJVISE: usize = 50;
/// Najveći broj serije kojoj raspon stane u pet znamenki ID-a.
pub(crate) const SERIJA_NAJVECA: u32 = 1998;
/// Učestalost [PROC] §3.1: jedno ime najviše 1,5 %, jedno prezime najviše 0,6 % vrste (a uvijek barem 1).
pub(crate) const UCESTALOST_IME_BP: usize = 150;
pub(crate) const UCESTALOST_PREZIME_BP: usize = 60;
/// Stereotip [PROC] §3.3: skupina podrijetla smije odstupati od cijele banke najviše 5 p. b.; pravilo šuti dok svaka
/// skupina nema barem 40 zapisa 🟨.
pub(crate) const STEREOTIP_BP: u32 = 500;
pub(crate) const STEREOTIP_MIN_SKUPINA: usize = 40;
/// Prekretnica [PROC] §3.6: na pečaćenju prekretnice banka odstupa od cilja najviše 3 p. b.
pub(crate) const PREKRETNICA_BP: u32 = 300;
/// Iskustvo po razini [PROC] §3.2: početnik 0–2, iskusan 3–9, stručnjak 8 i više godina.
pub(crate) const ISKUSTVO_POCETNIK: RangeInclusive<u8> = 0..=2;
pub(crate) const ISKUSTVO_ISKUSAN: RangeInclusive<u8> = 3..=9;
pub(crate) const ISKUSTVO_STRUCNJAK_OD: u8 = 8;
/// Godine iskustva najviše dob − 16.
pub(crate) const ISKUSTVO_OD_DOBI: i32 = 16;
pub(crate) const DOB_RADNIKA: RangeInclusive<u8> = 18..=65;
pub(crate) const DOB_GOSTA: RangeInclusive<u8> = 18..=85;
pub(crate) const BP_UKUPNO: u64 = 10_000;

// ---------------------------------------------------------------------------------------------------------------
// Popis prekršaja
// ---------------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct Popis(Vec<(String, String, String)>);

impl Popis {
    pub(crate) fn dodaj(&mut self, put: &str, id: &str, poruka: String) {
        self.0.push((put.to_string(), id.to_string(), poruka));
    }

    pub(crate) fn rezultat(mut self) -> Result<(), ContentError> {
        if self.0.is_empty() {
            return Ok(());
        }
        self.0.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        let retci: Vec<String> = self
            .0
            .into_iter()
            .map(|(put, id, poruka)| match (put.is_empty(), id.is_empty()) {
                (true, _) => poruka,
                (false, true) => format!("{put}: {poruka}"),
                (false, false) => format!("{put} {id}: {poruka}"),
            })
            .collect();
        Err(ContentError::Invalid(retci.join("\n")))
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Banka
// ---------------------------------------------------------------------------------------------------------------

/// Zabranjeno pripremljeno za usporedbu po riječima.
pub(crate) struct Zabrane {
    rijeci: Vec<(Vec<String>, String)>,
    nazivi: Vec<(Vec<String>, String)>,
    pub(crate) osobe: Vec<(Vec<String>, String)>,
}

impl Zabrane {
    fn iz(o: &Okvir) -> Zabrane {
        let pripremi = |v: &[String]| {
            let mut vidjeno = BTreeSet::new();
            v.iter()
                .map(|s| (rijeci(s), s.clone()))
                .filter(|(r, _)| !r.is_empty() && vidjeno.insert(r.clone()))
                .collect()
        };
        let z = &o.zabranjeno;
        Zabrane {
            rijeci: pripremi(&z.rijeci_opisa),
            nazivi: pripremi(&[z.hoteli.clone(), z.marke.clone()].concat()),
            osobe: pripremi(&z.osobe),
        }
    }
}

/// Zajednički dio radnika i gosta.
struct Osoba<'a> {
    put: &'a str,
    id: &'a str,
    ime: &'a str,
    prezime: &'a str,
    dob: u8,
    zemlja: &'a str,
    jezici: &'a [(String, u8)],
    opis: Option<&'a str>,
}

pub(crate) fn provjeri_banku(b: &Banka, o: &Okvir) -> Result<(), ContentError> {
    let mut p = Popis::default();
    let z = Zabrane::iz(o);
    let mut vidjeni = BTreeSet::new();
    for (put, s) in &b.radnici {
        if let Some(raspon) = zaglavlje(&mut p, put, s, Vrsta::Radnici) {
            id_jevi(
                &mut p,
                put,
                s.zapisi.iter().map(|r| r.id.as_str()),
                'R',
                raspon,
                &mut vidjeni,
            );
        }
        for r in &s.zapisi {
            radnik(&mut p, o, &z, put, r);
        }
    }
    for (put, s) in &b.gosti {
        if let Some(raspon) = zaglavlje(&mut p, put, s, Vrsta::Gosti) {
            id_jevi(
                &mut p,
                put,
                s.zapisi.iter().map(|g| g.id.as_str()),
                'G',
                raspon,
                &mut vidjeni,
            );
        }
        for g in &s.zapisi {
            gost(&mut p, o, &z, put, g);
        }
    }
    crate::ljudi_imena::imena(&mut p, b, o, &z);
    for redak in crate::ljudi_raspodjele::stereotip(b, &o.raspodjele) {
        p.dodaj("radnici", "", redak);
    }
    p.rezultat()
}

/// Zaglavlje serije; vraća raspon ID-jeva (iz imena datoteke) ili `None` kad ga nema.
fn zaglavlje<T>(p: &mut Popis, put: &str, s: &Serija<T>, vrsta: Vrsta) -> Option<(u32, u32)> {
    if s.format != 1 {
        p.dodaj(put, "", format!("format {}, poznat je samo 1", s.format));
    }
    if s.vrsta != vrsta {
        let (ima, mapa) = (s.vrsta.kod(), vrsta.kod());
        p.dodaj(put, "", format!("vrsta '{ima}' u mapi {mapa}"));
    }
    let ime = put.rsplit('/').next().unwrap_or(put);
    let n = match broj_serije(ime) {
        Some(n) => {
            if s.serija != n {
                let stem = ime.trim_end_matches(".json");
                p.dodaj(
                    put,
                    "",
                    format!("serija {} ne odgovara imenu {stem}", s.serija),
                );
            }
            n
        }
        None => {
            p.dodaj(
                put,
                "",
                format!("ime datoteke mora biti serija-NNN.json, ima '{ime}'"),
            );
            s.serija
        }
    };
    if s.zapisi.len() > SERIJA_NAJVISE {
        let n = s.zapisi.len();
        let rijec = mnozina(n, "zapis", "zapisa", "zapisa");
        p.dodaj(
            put,
            "",
            format!("serija ima {n} {rijec}, najviše {SERIJA_NAJVISE}"),
        );
    }
    if n > SERIJA_NAJVECA {
        p.dodaj(
            put,
            "",
            format!("serija {n} veća od {SERIJA_NAJVECA} (ID ima 5 znamenki)"),
        );
        return None;
    }
    let sirina = SERIJA_NAJVISE as u32;
    let (od, do_) = (sirina * n + 1, sirina * n + sirina);
    let slovo = if vrsta == Vrsta::Radnici { 'R' } else { 'G' };
    let (od_s, do_s) = (format!("{slovo}-{od:05}"), format!("{slovo}-{do_:05}"));
    if s.od != od_s || s.do_ != do_s {
        let ima = format!("{}–{}", s.od, s.do_);
        p.dodaj(
            put,
            "",
            format!("od/do: raspon mora biti {od_s}–{do_s}, ima {ima}"),
        );
    }
    Some((od, do_))
}

fn broj_serije(ime: &str) -> Option<u32> {
    let d = ime.strip_prefix("serija-")?.strip_suffix(".json")?;
    let n: u32 = d.parse().ok()?;
    (d.len() >= 3 && d.bytes().all(|c| c.is_ascii_digit()) && format!("{n:03}") == d).then_some(n)
}

/// Oblik, raspon serije, rastući redoslijed i jedinstvenost ID-jeva u cijeloj banci.
fn id_jevi<'a>(
    p: &mut Popis,
    put: &str,
    ids: impl Iterator<Item = &'a str>,
    slovo: char,
    (od, do_): (u32, u32),
    vidjeni: &mut BTreeSet<String>,
) {
    let mut prethodni: Option<u32> = None;
    for id in ids {
        let broj = id
            .strip_prefix(slovo)
            .and_then(|r| r.strip_prefix('-'))
            .filter(|d| d.len() == 5 && d.bytes().all(|c| c.is_ascii_digit()))
            .and_then(|d| d.parse::<u32>().ok());
        let Some(broj) = broj else {
            p.dodaj(
                put,
                id,
                format!("id: '{id}' mora biti {slovo}- i 5 znamenki"),
            );
            continue;
        };
        if !(od..=do_).contains(&broj) {
            let raspon = format!("{slovo}-{od:05}–{slovo}-{do_:05}");
            p.dodaj(put, id, format!("id: {id} izvan raspona serije {raspon}"));
        }
        if !vidjeni.insert(id.to_string()) {
            p.dodaj(put, id, format!("id: {id} se ponavlja"));
        } else if let Some(pr) = prethodni.filter(|pr| broj < *pr) {
            p.dodaj(
                put,
                id,
                format!("id: {id} nije iza {slovo}-{pr:05} (ID-jevi u seriji rastu)"),
            );
        }
        prethodni = Some(prethodni.map_or(broj, |pr| pr.max(broj)));
    }
}

fn radnik(p: &mut Popis, o: &Okvir, z: &Zabrane, put: &str, r: &Radnik) {
    let id = r.id.as_str();
    osoba(
        p,
        o,
        z,
        &Osoba {
            put,
            id,
            ime: &r.ime,
            prezime: &r.prezime,
            dob: r.dob,
            zemlja: &r.zemlja,
            jezici: &r.jezici,
            opis: r.opis.as_deref(),
        },
        &DOB_RADNIKA,
    );
    let g = r.godine_iskustva;
    let najvise = i32::from(r.dob) - ISKUSTVO_OD_DOBI;
    if i32::from(g) > najvise {
        p.dodaj(
            put,
            id,
            format!("godine_iskustva: {g} veće od dob − 16 ({najvise})"),
        );
    }
    let (kod, ok, raspon) = match r.razina {
        Razina::Pocetnik => (
            "pocetnik",
            ISKUSTVO_POCETNIK.contains(&g),
            raspon_godina(&ISKUSTVO_POCETNIK),
        ),
        Razina::Iskusan => (
            "iskusan",
            ISKUSTVO_ISKUSAN.contains(&g),
            raspon_godina(&ISKUSTVO_ISKUSAN),
        ),
        Razina::Strucnjak => (
            "strucnjak",
            g >= ISKUSTVO_STRUCNJAK_OD,
            format!("{ISKUSTVO_STRUCNJAK_OD} ili više godina"),
        ),
    };
    if !ok {
        p.dodaj(
            put,
            id,
            format!("razina: razina {kod} traži {raspon} iskustva, ima {g}"),
        );
    }
    let pod = r.podrijetlo.kod();
    match r.podrijetlo {
        Podrijetlo::Lokalno | Podrijetlo::Regija if r.zemlja != "HR" => {
            let zemlja = &r.zemlja;
            p.dodaj(
                put,
                id,
                format!("podrijetlo: podrijetlo {pod} traži zemlju HR, ima {zemlja}"),
            );
        }
        Podrijetlo::Inozemstvo if r.zemlja == "HR" => {
            p.dodaj(
                put,
                id,
                "podrijetlo: podrijetlo inozemstvo ne može imati zemlju HR".into(),
            );
        }
        _ => {}
    }
    let katalog = match r.podrijetlo {
        Podrijetlo::Lokalno => Some(&o.mjesta.lokalno),
        Podrijetlo::Regija => Some(&o.mjesta.regija),
        Podrijetlo::Inozemstvo => None,
    };
    match katalog {
        Some(k) if !k.contains(&r.mjesto) => {
            let m = &r.mjesto;
            p.dodaj(put, id, format!("mjesto: '{m}' nije u mjesta.json → {pod}"));
        }
        Some(_) => {}
        None => naziv(p, put, id, "mjesto", &r.mjesto, &MJESTO_ZNAKOVA),
    }
    if r.uloga == Uloga::Recepcioner && razina_jezika(&r.jezici, "en") < 2 {
        p.dodaj(put, id, "jezici: recepcioner traži engleski barem 2".into());
    }
}

fn raspon_godina(r: &RangeInclusive<u8>) -> String {
    let rijec = mnozina(usize::from(*r.end()), "godinu", "godine", "godina");
    format!("{}–{} {rijec}", r.start(), r.end())
}

fn gost(p: &mut Popis, o: &Okvir, z: &Zabrane, put: &str, g: &Gost) {
    let id = g.id.as_str();
    osoba(
        p,
        o,
        z,
        &Osoba {
            put,
            id,
            ime: &g.ime,
            prezime: &g.prezime,
            dob: g.dob,
            zemlja: &g.zemlja,
            jezici: &g.jezici,
            opis: g.opis.as_deref(),
        },
        &DOB_GOSTA,
    );
    naziv(p, put, id, "mjesto", &g.mjesto, &MJESTO_ZNAKOVA);
    if let Some(zan) = &g.zanimanje {
        let n = zan.chars().count();
        if !ZANIMANJE_ZNAKOVA.contains(&n) {
            let (a, b) = (ZANIMANJE_ZNAKOVA.start(), ZANIMANJE_ZNAKOVA.end());
            p.dodaj(
                put,
                id,
                format!("zanimanje: {n} znakova, dopušteno {a}–{b}"),
            );
        }
        znakovi_teksta(p, put, id, "zanimanje", zan);
        nazivi_u_tekstu(p, z, put, id, "zanimanje", zan);
    }
}

/// Ime, dob, zemlja, jezici i opis — isto za radnika i gosta.
fn osoba(p: &mut Popis, o: &Okvir, z: &Zabrane, x: &Osoba, dob: &RangeInclusive<u8>) {
    let (put, id) = (x.put, x.id);
    naziv(p, put, id, "ime", x.ime, &IME_ZNAKOVA);
    naziv(p, put, id, "prezime", x.prezime, &IME_ZNAKOVA);
    if !dob.contains(&x.dob) {
        let (d, a, b) = (x.dob, dob.start(), dob.end());
        p.dodaj(put, id, format!("dob: {d} izvan {a}–{b}"));
    }
    let zemlja = o.mjesta.zemlje.get(x.zemlja);
    if zemlja.is_none() {
        p.dodaj(
            put,
            id,
            format!("zemlja: '{}' nije u mjesta.json", x.zemlja),
        );
    }
    let mut vidjeni = BTreeSet::new();
    let mut ponovljeni = BTreeSet::new();
    for (k, r) in x.jezici {
        if !o.mjesta.jezici.contains(k) {
            p.dodaj(put, id, format!("jezici: nepoznat jezik '{k}'"));
        }
        if !(1..=4).contains(r) {
            p.dodaj(
                put,
                id,
                format!("jezici: razina {r} za '{k}', dopušteno 1–4"),
            );
        }
        if !vidjeni.insert(k) && ponovljeni.insert(k) {
            p.dodaj(put, id, format!("jezici: '{k}' se ponavlja"));
        }
    }
    if let Some(zm) = zemlja {
        if !zm.materinji.iter().any(|m| razina_jezika(x.jezici, m) == 4) {
            let (kod, m) = (x.zemlja, zm.materinji.join(" ili "));
            let poruka =
                format!("jezici: materinji jezik zemlje {kod} mora biti na razini 4 ({m})");
            p.dodaj(put, id, poruka);
        }
        if x.zemlja != "HR" && zm.razumije_hr && razina_jezika(x.jezici, "hr") < 4 {
            let poruka = format!(
                "jezici: zemlja {} traži hrvatski na razini 4 (ADR-042 t. 11)",
                x.zemlja
            );
            p.dodaj(put, id, poruka);
        }
    }
    if let Some(opis) = x.opis {
        provjeri_opis(p, z, put, id, opis);
    }
}

fn razina_jezika(jezici: &[(String, u8)], kod: &str) -> u8 {
    jezici
        .iter()
        .filter(|(k, _)| k == kod)
        .map(|(_, r)| *r)
        .max()
        .unwrap_or(0)
}

/// Ime, prezime ili slobodno mjesto: duljina i latinica s dijakriticima, razmak, `-`, `'`.
fn naziv(
    p: &mut Popis,
    put: &str,
    id: &str,
    polje: &str,
    s: &str,
    duljina: &RangeInclusive<usize>,
) {
    let n = s.chars().count();
    if !duljina.contains(&n) {
        let rijec = mnozina(n, "znak", "znaka", "znakova");
        let (a, b) = (duljina.start(), duljina.end());
        p.dodaj(put, id, format!("{polje}: {n} {rijec}, dopušteno {a}–{b}"));
    }
    let dopusten = |c: char| latinsko_slovo(c) || matches!(c, ' ' | '-' | '\'' | '’');
    if let Some(c) = s.chars().find(|c| !dopusten(*c)) {
        p.dodaj(put, id, format!("{polje}: nedopušten znak '{c}'"));
    } else if s != s.trim() || s.contains("  ") {
        p.dodaj(
            put,
            id,
            format!("{polje}: razmak na početku, na kraju ili dvostruki"),
        );
    }
}

fn znakovi_teksta(p: &mut Popis, put: &str, id: &str, polje: &str, s: &str) {
    if s.contains(['{', '}']) {
        p.dodaj(put, id, format!("{polje}: vitice {{}} nisu dopuštene"));
    }
    if s.chars().any(cirilica) {
        p.dodaj(put, id, format!("{polje}: ćirilica nije dopuštena"));
    }
}

fn nazivi_u_tekstu(p: &mut Popis, z: &Zabrane, put: &str, id: &str, polje: &str, s: &str) {
    let tekst = rijeci(s);
    for (fraza, ime) in &z.nazivi {
        if sadrzi(&tekst, fraza) {
            p.dodaj(put, id, format!("{polje}: zabranjeni naziv '{ime}'"));
        }
    }
    for (fraza, ime) in &z.osobe {
        if sadrzi(&tekst, fraza) {
            p.dodaj(put, id, format!("{polje}: zabranjena osoba '{ime}'"));
        }
    }
}

/// Broj rečenica: niz `.`, `!`, `?` iza sadržaja zatvara rečenicu; ostatak bez točke je još jedna.
fn broj_recenica(s: &str) -> usize {
    let (mut n, mut sadrzaj) = (0, false);
    for c in s.chars() {
        if matches!(c, '.' | '!' | '?') {
            if sadrzaj {
                n += 1;
                sadrzaj = false;
            }
        } else if !c.is_whitespace() {
            sadrzaj = true;
        }
    }
    n + usize::from(sadrzaj)
}

fn provjeri_opis(p: &mut Popis, z: &Zabrane, put: &str, id: &str, opis: &str) {
    let n = opis.chars().count();
    if !OPIS_ZNAKOVA.contains(&n) {
        let (a, b) = (OPIS_ZNAKOVA.start(), OPIS_ZNAKOVA.end());
        p.dodaj(put, id, format!("opis: {n} znakova, dopušteno {a}–{b}"));
    }
    let r = broj_recenica(opis);
    if r > OPIS_RECENICA {
        let rijec = mnozina(r, "rečenica", "rečenice", "rečenica");
        let poruka = format!(
            "opis: {r} {rijec}, najviše {OPIS_RECENICA} (kratica s točkom broji se kao rečenica)"
        );
        p.dodaj(put, id, poruka);
    }
    if !opis.trim_end().ends_with('.') {
        p.dodaj(put, id, "opis: mora završiti točkom".into());
    }
    znakovi_teksta(p, put, id, "opis", opis);
    let tekst = rijeci(opis);
    for (fraza, rijec) in &z.rijeci {
        if sadrzi(&tekst, fraza) {
            p.dodaj(put, id, format!("opis: zabranjena riječ '{rijec}'"));
        }
    }
    nazivi_u_tekstu(p, z, put, id, "opis", opis);
}
