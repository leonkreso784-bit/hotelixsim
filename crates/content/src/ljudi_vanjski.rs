//! Pravila vrste VANJSKI (BANKA_LJUDI §1.5, §3.7): zapis (dob, zemlja i mjesto po vrsti uloge, ustanova), okvir
//! (`raspodjele.vanjski`) i izvještaj prema ciljevima. Ime, jezici i opis idu kroz isti `osoba()` kao radnik i gost;
//! jedinstvenost i učestalost imena su u `ljudi_imena`. Pragovi [PROC] su konstante ovdje, 🟨 pretpostavke kartice K2v.

use std::ops::RangeInclusive;

use crate::ljudi::{Banka, Odstupanje, Okvir, Raspodjele, Rod, UlogaVanjskog, Vanjski};
use crate::ljudi_check::{
    BP_UKUPNO, MJESTO_ZNAKOVA, Osoba, Popis, Zabrane, naziv, nazivi_u_tekstu, osoba, znakovi_teksta,
};
use crate::ljudi_check_okvir::{potpuno, razredi, zbroj};
use crate::ljudi_raspodjele::{bp, dobni_razred, kodovi, udjeli};

pub(crate) const DOB_VANJSKOG: RangeInclusive<u8> = 20..=80;
/// Ustanova: 2–60 znakova 🟨.
pub(crate) const USTANOVA_ZNAKOVA: RangeInclusive<usize> = 2..=60;
/// Službe su u mirovini najkasnije s 65 godina 🟨.
pub(crate) const SLUZBE_DOB_NAJVISE: u8 = 65;
/// Inspektori, djelatnik hitne i liječnik: studij i staž, najmanje 25 godina 🟨.
pub(crate) const STRUCNI_DOB_NAJMANJE: u8 = 25;

fn strucna(u: UlogaVanjskog) -> bool {
    use UlogaVanjskog::*;
    matches!(
        u,
        InspektorSanitarni
            | InspektorTuristicki
            | InspektorRada
            | InspektorPorezni
            | DjelatnikHitne
            | Lijecnik
    )
}

pub(crate) fn vanjski(p: &mut Popis, o: &Okvir, z: &Zabrane, put: &str, v: &Vanjski) {
    let id = v.id.as_str();
    osoba(
        p,
        o,
        z,
        &Osoba {
            put,
            id,
            ime: &v.ime,
            prezime: &v.prezime,
            dob: v.dob,
            zemlja: &v.zemlja,
            jezici: &v.jezici,
            opis: v.opis.as_deref(),
            vrsta_radnik: false,
        },
        &DOB_VANJSKOG,
    );
    let (u, kod) = (v.uloga, v.uloga.kod());
    if u.sluzba() && v.dob > SLUZBE_DOB_NAJVISE {
        let d = v.dob;
        let poruka = format!("dob: {d}, uloga {kod} najviše {SLUZBE_DOB_NAJVISE}");
        p.dodaj(put, id, poruka);
    }
    if strucna(u) && v.dob < STRUCNI_DOB_NAJMANJE {
        let d = v.dob;
        let poruka = format!("dob: {d}, uloga {kod} najmanje {STRUCNI_DOB_NAJMANJE}");
        p.dodaj(put, id, poruka);
    }
    mjesto(p, o, put, v);
    if let Some(ustanova) = &v.ustanova {
        ustanova_pravila(p, z, put, id, ustanova);
    }
}

/// Lokalne uloge: HR i mjesto iz `lokalno`; tržišne: iz HR mjesto iz `lokalno` ili `regija`, strane slobodan naziv.
fn mjesto(p: &mut Popis, o: &Okvir, put: &str, v: &Vanjski) {
    let (id, m) = (v.id.as_str(), v.mjesto.as_str());
    let mj = &o.mjesta;
    if v.uloga.lokalna() {
        if v.zemlja != "HR" {
            let (kod, zemlja) = (v.uloga.kod(), &v.zemlja);
            p.dodaj(
                put,
                id,
                format!("zemlja: uloga {kod} traži HR, ima {zemlja}"),
            );
        }
        if !mj.lokalno.iter().any(|x| x == m) {
            p.dodaj(
                put,
                id,
                format!("mjesto: '{m}' nije u mjesta.json → lokalno"),
            );
        }
    } else if v.zemlja == "HR" {
        if !mj.lokalno.iter().chain(&mj.regija).any(|x| x == m) {
            let poruka = format!("mjesto: '{m}' nije u mjesta.json → lokalno ili regija");
            p.dodaj(put, id, poruka);
        }
    } else {
        naziv(p, put, id, "mjesto", m, &MJESTO_ZNAKOVA);
    }
}

/// Generički naziv ustanove: duljina, bez vitica i ćirilice, bez zabranjenih naziva, svaka riječ osim prve malim slovom.
fn ustanova_pravila(p: &mut Popis, z: &Zabrane, put: &str, id: &str, s: &str) {
    let n = s.chars().count();
    if !USTANOVA_ZNAKOVA.contains(&n) {
        let rijec = if n == 1 { "znak" } else { "znakova" };
        let (a, b) = (USTANOVA_ZNAKOVA.start(), USTANOVA_ZNAKOVA.end());
        p.dodaj(put, id, format!("ustanova: {n} {rijec}, dopušteno {a}–{b}"));
    }
    znakovi_teksta(p, put, id, "ustanova", s);
    nazivi_u_tekstu(p, z, put, id, "ustanova", s);
    for rijec in s.split_whitespace().skip(1) {
        if rijec.chars().next().is_some_and(char::is_uppercase) {
            let poruka = format!("ustanova: veliko slovo u '{rijec}' (samo generički naziv)");
            p.dodaj(put, id, poruka);
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Okvir i izvještaj
// ---------------------------------------------------------------------------------------------------------------

/// `raspodjele.vanjski`: uloge i rod po ulozi potpuni, zbroj uloga i dobi 10 000, dob bez rupa, izvor roda obvezan.
pub(crate) fn okvir(p: &mut Popis, o: &Okvir) {
    let r = &o.raspodjele.vanjski;
    let sve = &UlogaVanjskog::SVE;
    let kod = UlogaVanjskog::kod;
    potpuno(p, "vanjski.uloga", &r.uloga, sve, kod, "uloge");
    potpuno(
        p,
        "vanjski.rod_po_ulozi",
        &r.rod_po_ulozi,
        sve,
        kod,
        "uloge",
    );
    zbroj(p, "vanjski.uloga", r.uloga.values());
    zbroj(p, "vanjski.dob", r.dob.values());
    razredi(p, "vanjski.dob", &r.dob, &DOB_VANJSKOG);
    for (u, rod) in &r.rod_po_ulozi {
        let u = u.kod();
        let pocetak = format!("raspodjele.vanjski.rod_po_ulozi.{u}");
        if u64::from(rod.z_bp) > BP_UKUPNO {
            p.dodaj(
                "",
                "",
                format!("{pocetak}: z_bp {} veći od 10000", rod.z_bp),
            );
        }
        if rod.izvor.trim().is_empty() {
            p.dodaj(
                "",
                "",
                format!("{pocetak}: izvor je obvezan (ADR-042 t. 8)"),
            );
        }
    }
}

/// Udio svake uloge, dobnog razreda i žena po ulozi prema ciljevima; bez vanjskih u banci ne dodaje ništa.
pub(crate) fn izvjestaj(out: &mut Vec<Odstupanje>, b: &Banka, r: &Raspodjele) {
    let svi: Vec<&Vanjski> = b.vanjski.iter().flat_map(|(_, s)| &s.zapisi).collect();
    if svi.is_empty() {
        return;
    }
    let rv = &r.vanjski;
    let uloge: Vec<String> = svi.iter().map(|x| x.uloga.kod().to_string()).collect();
    udjeli(
        out,
        "vanjski: uloga",
        &kodovi(&rv.uloga, UlogaVanjskog::kod),
        uloge,
    );
    let dob: Vec<String> = svi.iter().map(|x| dobni_razred(x.dob, &rv.dob)).collect();
    udjeli(out, "vanjski: dob", &rv.dob, dob);
    for (u, rod) in &rv.rod_po_ulozi {
        let u_svi: Vec<&&Vanjski> = svi.iter().filter(|x| x.uloga == *u).collect();
        if u_svi.is_empty() {
            continue;
        }
        let z = u_svi.iter().filter(|x| x.rod == Rod::Z).count();
        out.push(Odstupanje {
            dimenzija: "vanjski: rod z u ulozi".into(),
            kljuc: u.kod().into(),
            cilj_bp: rod.z_bp,
            stvarno_bp: bp(z, u_svi.len()),
        });
    }
}
