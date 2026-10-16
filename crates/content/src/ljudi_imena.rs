//! Imena u banci ljudi (BANKA_LJUDI §3.1, §3.5): puno ime jedinstveno među radnicima, gostima i partnerima iz
//! `v2/names.json` (J2), nije na popisu zabranjenih osoba (Z1), a jedno ime ili prezime ne ponavlja se češće od
//! praga [PROC] (J3, J4). Usporedba je bez velikih slova i dijakritika.

use std::collections::{BTreeMap, BTreeSet};

use crate::ljudi::{Banka, Okvir};
use crate::ljudi_check::{BP_UKUPNO, Popis, UCESTALOST_IME_BP, UCESTALOST_PREZIME_BP, Zabrane};
use crate::ljudi_tekst::{bez_dijakritika, kljuc_imena, rijeci};

/// Jedinstvenost punog imena (radnici, gosti i partneri), zabranjene osobe i učestalost imena i prezimena.
pub(crate) fn imena(p: &mut Popis, b: &Banka, o: &Okvir, z: &Zabrane) {
    let radnici: Vec<(&str, &str, &str, &str)> = b
        .radnici
        .iter()
        .flat_map(|(put, s)| {
            s.zapisi.iter().map(move |r| {
                (
                    put.as_str(),
                    r.id.as_str(),
                    r.ime.as_str(),
                    r.prezime.as_str(),
                )
            })
        })
        .collect();
    let gosti: Vec<(&str, &str, &str, &str)> = b
        .gosti
        .iter()
        .flat_map(|(put, s)| {
            s.zapisi.iter().map(move |g| {
                (
                    put.as_str(),
                    g.id.as_str(),
                    g.ime.as_str(),
                    g.prezime.as_str(),
                )
            })
        })
        .collect();

    let mut zauzeto: BTreeMap<String, String> = o
        .partneri
        .iter()
        .map(|ime| (rijeci(ime).join(" "), "partner".to_string()))
        .collect();
    let zabranjene: BTreeSet<String> = z.osobe.iter().map(|(r, _)| r.join(" ")).collect();
    for &(put, id, ime, prezime) in radnici.iter().chain(&gosti) {
        let kljuc = kljuc_imena(ime, prezime);
        let prikaz = bez_dijakritika(&format!("{ime} {prezime}"));
        if zabranjene.contains(&kljuc) {
            p.dodaj(
                put,
                id,
                format!("puno ime '{prikaz}' je na popisu zabranjenih osoba"),
            );
        }
        match zauzeto.get(&kljuc) {
            Some(v) if v == "partner" => {
                p.dodaj(put, id, format!("puno ime '{prikaz}' već je partner"));
            }
            Some(v) => p.dodaj(put, id, format!("puno ime '{prikaz}' već ima {v}")),
            None => {
                zauzeto.insert(kljuc, id.to_string());
            }
        }
    }

    for (zapisi, rijec) in [(&radnici, "radnika"), (&gosti, "gostiju")] {
        let n = zapisi.len();
        let imena: Vec<_> = zapisi
            .iter()
            .map(|&(put, id, ime, _)| (put, id, ime))
            .collect();
        let prezimena: Vec<_> = zapisi
            .iter()
            .map(|&(put, id, _, pr)| (put, id, pr))
            .collect();
        ucestalost(p, &imena, "ime", n, UCESTALOST_IME_BP, rijec);
        ucestalost(p, &prezimena, "prezime", n, UCESTALOST_PREZIME_BP, rijec);
    }
}

/// Jedno ime (ili prezime) najviše `bp` vrste, a uvijek barem jednom; redak ide zapisu koji prvi prijeđe granicu.
fn ucestalost(
    p: &mut Popis,
    zapisi: &[(&str, &str, &str)],
    polje: &str,
    n: usize,
    bp: usize,
    rijec: &str,
) {
    let najvise = (n * bp / BP_UKUPNO as usize).max(1);
    let mut ukupno: BTreeMap<String, (usize, &str)> = BTreeMap::new();
    for &(_, _, s) in zapisi {
        ukupno.entry(rijeci(s).join(" ")).or_insert((0, s)).0 += 1;
    }
    let mut dosad: BTreeMap<String, usize> = BTreeMap::new();
    for &(put, id, s) in zapisi {
        let k = rijeci(s).join(" ");
        let c = dosad.entry(k.clone()).or_insert(0);
        *c += 1;
        if *c == najvise + 1 {
            let (svega, prvo) = ukupno[&k];
            let poruka = format!("{polje} '{prvo}': {svega} od {n} {rijec}, najviše {najvise}");
            p.dodaj(put, id, poruka);
        }
    }
}
