//! Dosljednost okvira banke ljudi (R1–R3): `mjesta.json` (zemlje, jezici, mjesta bez ponavljanja) i
//! `raspodjele.json` (svaka dimenzija potpuna i zbroja 10 000 bp, dobni razredi bez rupa, izvor roda po ulozi).

use std::collections::{BTreeMap, BTreeSet};
use std::ops::RangeInclusive;

use crate::ContentError;
use crate::ljudi::{Okvir, Podrijetlo, Razina, Rod, Segment, Uloga};
use crate::ljudi_check::{BP_UKUPNO, DOB_GOSTA, DOB_RADNIKA, Popis};
use crate::ljudi_tekst::razred;

pub(crate) fn provjeri_okvir(o: &Okvir) -> Result<(), ContentError> {
    let mut p = Popis::default();
    let m = &o.mjesta;
    if !m.zemlje.contains_key("HR") {
        p.dodaj("", "", "mjesta.zemlje: nema HR".into());
    }
    let mut jezici = BTreeSet::new();
    for k in &m.jezici {
        if k.len() != 2 || !k.bytes().all(|c| c.is_ascii_lowercase()) {
            p.dodaj(
                "",
                "",
                format!("mjesta.jezici: kod '{k}' mora biti dva mala slova (ISO 639-1)"),
            );
        }
        if !jezici.insert(k) {
            p.dodaj("", "", format!("mjesta.jezici: '{k}' se ponavlja"));
        }
    }
    for (kod, zemlja) in &m.zemlje {
        if kod.len() != 2 || !kod.bytes().all(|c| c.is_ascii_uppercase()) {
            p.dodaj(
                "",
                "",
                format!("mjesta.zemlje: kod '{kod}' mora biti dva velika slova (ISO 3166)"),
            );
        }
        if zemlja.materinji.is_empty() {
            p.dodaj(
                "",
                "",
                format!("mjesta.zemlje.{kod}.materinji: prazan popis"),
            );
        }
        for j in zemlja.materinji.iter().filter(|j| !jezici.contains(j)) {
            p.dodaj(
                "",
                "",
                format!("mjesta.zemlje.{kod}.materinji: nepoznat jezik '{j}'"),
            );
        }
    }
    let mut mjesta = BTreeSet::new();
    for (popis, ime) in [(&m.lokalno, "lokalno"), (&m.regija, "regija")] {
        for x in popis {
            if !mjesta.insert(x) {
                p.dodaj(
                    "",
                    "",
                    format!("mjesta.{ime}: '{x}' se ponavlja (ili je i u lokalno i u regija)"),
                );
            }
        }
    }

    let r = &o.raspodjele.radnici;
    potpuno(
        &mut p,
        "radnici.uloga",
        &r.uloga,
        &Uloga::SVE,
        Uloga::kod,
        "uloge",
    );
    potpuno(
        &mut p,
        "radnici.razina",
        &r.razina,
        &Razina::SVE,
        Razina::kod,
        "razine",
    );
    potpuno(
        &mut p,
        "radnici.podrijetlo",
        &r.podrijetlo,
        &Podrijetlo::SVE,
        Podrijetlo::kod,
        "podrijetla",
    );
    potpuno(
        &mut p,
        "radnici.rod_po_ulozi",
        &r.rod_po_ulozi,
        &Uloga::SVE,
        Uloga::kod,
        "uloge",
    );
    zbroj(&mut p, "radnici.uloga", r.uloga.values());
    zbroj(&mut p, "radnici.razina", r.razina.values());
    zbroj(&mut p, "radnici.dob", r.dob.values());
    zbroj(&mut p, "radnici.podrijetlo", r.podrijetlo.values());
    zbroj(
        &mut p,
        "radnici.zemlje_inozemstvo",
        r.zemlje_inozemstvo.values(),
    );
    razredi(&mut p, "radnici.dob", &r.dob, &DOB_RADNIKA);
    if r.zemlje_izvor.trim().is_empty() {
        p.dodaj(
            "",
            "",
            "raspodjele.radnici.zemlje_izvor: izvor je obvezan".into(),
        );
    }
    for k in r.zemlje_inozemstvo.keys().filter(|k| *k != "ostalo") {
        if k == "HR" || !m.zemlje.contains_key(k) {
            p.dodaj(
                "",
                "",
                format!(
                    "raspodjele.radnici.zemlje_inozemstvo: '{k}' nije strana zemlja iz mjesta.json"
                ),
            );
        }
    }
    for (jezik, udjeli) in &r.jezici {
        if !jezici.contains(jezik) {
            p.dodaj(
                "",
                "",
                format!("raspodjele.radnici.jezici: nepoznat jezik '{jezik}'"),
            );
        }
        zbroj(&mut p, &format!("radnici.jezici.{jezik}"), udjeli.iter());
    }
    for (u, rod) in &r.rod_po_ulozi {
        let u = u.kod();
        if u64::from(rod.z_bp) > BP_UKUPNO {
            p.dodaj(
                "",
                "",
                format!(
                    "raspodjele.radnici.rod_po_ulozi.{u}: z_bp {} veći od 10000",
                    rod.z_bp
                ),
            );
        }
        if rod.izvor.trim().is_empty() {
            p.dodaj(
                "",
                "",
                format!("raspodjele.radnici.rod_po_ulozi.{u}: izvor je obvezan (ADR-042 t. 8)"),
            );
        }
    }

    let g = &o.raspodjele.gosti;
    potpuno(
        &mut p,
        "gosti.segment",
        &g.segment,
        &Segment::SVE,
        Segment::kod,
        "segmenta",
    );
    potpuno(&mut p, "gosti.rod", &g.rod, &Rod::SVE, Rod::kod, "roda");
    zbroj(&mut p, "gosti.zemlja", g.zemlja.values());
    zbroj(&mut p, "gosti.segment", g.segment.values());
    zbroj(&mut p, "gosti.dob", g.dob.values());
    zbroj(&mut p, "gosti.rod", g.rod.values());
    razredi(&mut p, "gosti.dob", &g.dob, &DOB_GOSTA);
    p.rezultat()
}

fn zbroj<'a>(p: &mut Popis, ime: &str, v: impl Iterator<Item = &'a u32>) {
    let s: u64 = v.map(|x| u64::from(*x)).sum();
    if s != BP_UKUPNO {
        p.dodaj(
            "",
            "",
            format!("raspodjele.{ime}: zbroj {s} bp, mora biti 10000"),
        );
    }
}

fn potpuno<K: Ord + Copy, V>(
    p: &mut Popis,
    ime: &str,
    m: &BTreeMap<K, V>,
    sve: &[K],
    kod: fn(K) -> &'static str,
    rijec: &str,
) {
    for k in sve.iter().filter(|k| !m.contains_key(k)) {
        p.dodaj(
            "",
            "",
            format!("raspodjele.{ime}: nema {rijec} {}", kod(*k)),
        );
    }
}

/// Dobni razredi pokrivaju cijeli raspon vrste, bez rupa i preklapanja.
fn razredi(p: &mut Popis, ime: &str, m: &BTreeMap<String, u32>, raspon: &RangeInclusive<u8>) {
    let mut r = Vec::new();
    for k in m.keys() {
        match razred(k) {
            Some(x) => r.push(x),
            None => p.dodaj(
                "",
                "",
                format!("raspodjele.{ime}: razred '{k}' nije oblika od-do"),
            ),
        }
    }
    r.sort_unstable();
    let mut sljedeci = u16::from(*raspon.start());
    let mut ok = !r.is_empty();
    for (a, b) in &r {
        ok &= u16::from(*a) == sljedeci;
        sljedeci = u16::from(*b) + 1;
    }
    if !ok || sljedeci != u16::from(*raspon.end()) + 1 {
        let (a, b) = (raspon.start(), raspon.end());
        p.dodaj(
            "",
            "",
            format!("raspodjele.{ime}: razredi moraju pokriti {a}–{b} bez rupa i preklapanja"),
        );
    }
}
