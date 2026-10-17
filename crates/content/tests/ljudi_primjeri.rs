//! Banka ljudi (K1b), G2: 20 primjera (10 radnika, 10 gostiju) pokrivaju raznolikost iz kartice K1b i nacrta §2.2/§2.3.
//! Dobne razrede čita iz `raspodjele.json` (ne prepisuje ih). Svi prekršaji se skupljaju u jedan `panic!`.
use content::ljudi::{Banka, Okvir, Razina, Rod, Segment, Uloga, UlogaVanjskog};
use content::repo_dir;
use content::templates::Names;
use std::collections::BTreeSet;

/// Dobni razred `"18-24"` → (18, 24).
fn raspon(k: &str) -> (u8, u8) {
    let (a, b) = k
        .split_once('-')
        .unwrap_or_else(|| panic!("dobni razred {k:?} nije oblika 'od-do'"));
    (a.parse().unwrap(), b.parse().unwrap())
}

fn sva_imena() -> BTreeSet<String> {
    let dir = repo_dir();
    let text = std::fs::read_to_string(dir.join("v2/names.json")).expect("v2/names.json se čita");
    let n: Names = serde_json::from_str(&text).expect("v2/names.json se parsira");
    [n.guest, n.staff, n.partner]
        .into_iter()
        .flat_map(|l| l.f.into_iter().chain(l.m))
        .map(|x| {
            x.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        })
        .collect()
}

#[test]
fn primjeri_su_raznolici() {
    let dir = repo_dir();
    let okvir = Okvir::read(&dir).expect("okvir se čita");
    let banka = Banka::read(&dir).expect("banka se čita");
    // K2: od serije-001 banka ima više serija; ovaj test i dalje čuva samo prvih 20 primjera (serija-000).
    let radnici: Vec<_> = banka
        .radnici
        .iter()
        .filter(|(put, _)| put.ends_with("/serija-000.json"))
        .flat_map(|(_, s)| &s.zapisi)
        .collect();
    let gosti: Vec<_> = banka
        .gosti
        .iter()
        .filter(|(put, _)| put.ends_with("/serija-000.json"))
        .flat_map(|(_, s)| &s.zapisi)
        .collect();
    let mut e: Vec<String> = Vec::new();

    if banka.radnici.is_empty() || banka.gosti.is_empty() {
        e.push(format!(
            "nema serije: radnici {} datoteka, gosti {} datoteka (očekivano radnici/serija-000.json i gosti/serija-000.json)",
            banka.radnici.len(),
            banka.gosti.len()
        ));
    }
    if radnici.len() != 10 {
        e.push(format!("radnika je {}, mora biti 10", radnici.len()));
    }
    if gosti.len() != 10 {
        e.push(format!("gostiju je {}, mora biti 10", gosti.len()));
    }

    // ID-jevi točno R-00001…R-00010 / G-00001…G-00010.
    let mut rid: Vec<&str> = radnici.iter().map(|r| r.id.as_str()).collect();
    let mut gid: Vec<&str> = gosti.iter().map(|g| g.id.as_str()).collect();
    rid.sort();
    gid.sort();
    let want_r: Vec<String> = (1..=10).map(|i| format!("R-{i:05}")).collect();
    let want_g: Vec<String> = (1..=10).map(|i| format!("G-{i:05}")).collect();
    if rid != want_r {
        e.push(format!(
            "ID-jevi radnika {rid:?}, očekivano R-00001…R-00010"
        ));
    }
    if gid != want_g {
        e.push(format!(
            "ID-jevi gostiju {gid:?}, očekivano G-00001…G-00010"
        ));
    }

    // Radnici.
    let uloge: BTreeSet<Uloga> = radnici.iter().map(|r| r.uloga).collect();
    if uloge.len() != Uloga::SVE.len() {
        let fale: Vec<&str> = Uloga::SVE
            .iter()
            .filter(|u| !uloge.contains(u))
            .map(|u| u.kod())
            .collect();
        e.push(format!(
            "radnici: uloge koje nedostaju: {fale:?} (traže se svih 9)"
        ));
    }
    let razine: BTreeSet<Razina> = radnici.iter().map(|r| r.razina).collect();
    if razine.len() != Razina::SVE.len() {
        e.push(format!("radnici: razina ima {} od 3", razine.len()));
    }
    if !radnici
        .iter()
        .any(|r| r.razina == Razina::Pocetnik && r.dob > 35)
    {
        e.push("radnici: nema početnika starijeg od 35".into());
    }
    for k in okvir.raspodjele.radnici.dob.keys() {
        let (a, z) = raspon(k);
        if !radnici.iter().any(|r| (a..=z).contains(&r.dob)) {
            e.push(format!("radnici: nema nikoga u dobnom razredu {k}"));
        }
    }
    for rod in Rod::SVE {
        if !radnici.iter().any(|r| r.rod == rod) {
            e.push(format!("radnici: nema roda {}", rod.kod()));
        }
    }
    let po = |p: &str| radnici.iter().filter(|r| r.podrijetlo.kod() == p).count();
    let (lok, reg, ino) = (po("lokalno"), po("regija"), po("inozemstvo"));
    if (lok, reg, ino) != (5, 2, 3) {
        e.push(format!(
            "radnici: podrijetlo lokalno/regija/inozemstvo = {lok}/{reg}/{ino}, mora biti 5/2/3"
        ));
    }
    let strani: Vec<_> = radnici
        .iter()
        .filter(|r| r.podrijetlo.kod() == "inozemstvo")
        .collect();
    let regionalna = |z: &str| ["BA", "RS", "ME"].contains(&z);
    if !strani
        .iter()
        .any(|r| regionalna(&r.zemlja) && r.jezici.iter().any(|(j, l)| j == "hr" && *l == 4))
    {
        e.push("radnici: nema stranca iz BA/RS/ME s jezikom [\"hr\",4]".into());
    }
    if !strani.iter().any(|r| !regionalna(&r.zemlja)) {
        e.push("radnici: nema stranca izvan regije (zemlja izvan BA, RS, ME)".into());
    }

    // Gosti.
    let zemlje: BTreeSet<&str> = gosti.iter().map(|g| g.zemlja.as_str()).collect();
    if zemlje.len() < 6 {
        e.push(format!(
            "gosti: {} zemalja {zemlje:?}, traži se najmanje 6",
            zemlje.len()
        ));
    }
    for z in ["AT", "DE", "HR"] {
        if !zemlje.contains(z) {
            e.push(format!("gosti: nema gosta iz zemlje {z}"));
        }
    }
    let segmenti: BTreeSet<Segment> = gosti.iter().map(|g| g.segment).collect();
    if segmenti.len() != Segment::SVE.len() {
        e.push(format!("gosti: segmenata ima {} od 5", segmenti.len()));
    }
    let zene = gosti.iter().filter(|g| g.rod == Rod::Z).count();
    if (zene, gosti.len() - zene) != (5, 5) && gosti.len() == 10 {
        e.push(format!(
            "gosti: {} ž / {} m, mora biti 5 / 5",
            zene,
            gosti.len() - zene
        ));
    }
    for k in okvir.raspodjele.gosti.dob.keys() {
        let (a, z) = raspon(k);
        if !gosti.iter().any(|g| (a..=z).contains(&g.dob)) {
            e.push(format!("gosti: nema nikoga u dobnom razredu {k}"));
        }
    }
    if !gosti.iter().any(|g| g.dob >= 65) {
        e.push("gosti: nema gosta od 65 godina naviše".into());
    }
    if gosti.iter().filter(|g| g.zanimanje.is_some()).count() < 5 {
        e.push("gosti: manje od 5 gostiju ima zanimanje".into());
    }

    // Opis i imena.
    let bez_opisa = radnici
        .iter()
        .map(|r| (&r.id, r.opis.is_some()))
        .chain(gosti.iter().map(|g| (&g.id, g.opis.is_some())))
        .filter(|(_, ima)| !ima)
        .count();
    if bez_opisa > 0 || radnici.len() + gosti.len() != 20 {
        e.push(format!(
            "opis: {bez_opisa} zapisa bez opisa, ukupno {} zapisa (traži se 20, svi s opisom)",
            radnici.len() + gosti.len()
        ));
    }
    let imena = sva_imena();
    let puna = radnici
        .iter()
        .map(|r| (&r.id, &r.ime, &r.prezime))
        .chain(gosti.iter().map(|g| (&g.id, &g.ime, &g.prezime)));
    for (id, ime, prezime) in puna {
        if imena.contains(&format!("{ime} {prezime}").to_lowercase()) {
            e.push(format!(
                "ime: {id} '{ime} {prezime}' je u names.json (guest/staff/partner)"
            ));
        }
    }

    assert!(
        e.is_empty(),
        "primjeri nisu raznoliki ({} prekršaja):\n  - {}",
        e.len(),
        e.join("\n  - ")
    );
}

/// K2v-b, VG2: 10 vanjskih u `vanjski/serija-000.json` pokrivaju raznolikost iz kartice K2v i nacrta §1.5/§2.4.
/// Dobne razrede čita iz `raspodjele.json` → `vanjski.dob`. Svi prekršaji se skupljaju u jedan `panic!`.
#[test]
fn vanjski_primjeri_raznoliki() {
    let dir = repo_dir();
    let okvir = Okvir::read(&dir).expect("okvir se čita");
    let banka = Banka::read(&dir).expect("banka se čita");
    let vanjski: Vec<_> = banka
        .vanjski
        .iter()
        .filter(|(put, _)| put.ends_with("/serija-000.json"))
        .flat_map(|(_, s)| &s.zapisi)
        .collect();
    assert!(
        !vanjski.is_empty(),
        "nema serije: vanjski {} datoteka, nijedna serija-000.json (očekivano content/hotel/v2/ljudi/vanjski/serija-000.json s 10 zapisa)",
        banka.vanjski.len()
    );
    let mut e: Vec<String> = Vec::new();

    if vanjski.len() != 10 {
        e.push(format!("vanjskih je {}, mora biti 10", vanjski.len()));
    }
    let mut vid: Vec<&str> = vanjski.iter().map(|v| v.id.as_str()).collect();
    vid.sort();
    let want: Vec<String> = (1..=10).map(|i| format!("V-{i:05}")).collect();
    if vid != want {
        e.push(format!(
            "ID-jevi vanjskih {vid:?}, očekivano V-00001…V-00010"
        ));
    }

    // Uloge.
    let uloge: BTreeSet<&str> = vanjski.iter().map(|v| v.uloga.kod()).collect();
    if uloge.len() < 8 {
        e.push(format!(
            "vanjski: {} različitih uloga {uloge:?}, traži se najmanje 8",
            uloge.len()
        ));
    }
    for obvezna in ["policajac", "vatrogasac", "inspektor_sanitarni"] {
        if !uloge.contains(obvezna) {
            e.push(format!("vanjski: nema uloge {obvezna} (Leonova obvezna)"));
        }
    }

    // Parovi (uloga, rod) koje predlošci danas traže.
    for (uloga, rod) in [
        (UlogaVanjskog::InspektorSanitarni, Rod::M),
        (UlogaVanjskog::Novinar, Rod::Z),
        (UlogaVanjskog::Dobavljac, Rod::M),
        (UlogaVanjskog::OrganizatorDogadanja, Rod::Z),
        (UlogaVanjskog::ZastupnikAgencije, Rod::Z),
    ] {
        if !vanjski.iter().any(|v| v.uloga == uloga && v.rod == rod) {
            e.push(format!(
                "vanjski: nema para uloga {} + rod {} (traže ga predlošci)",
                uloga.kod(),
                rod.kod()
            ));
        }
    }
    for rod in Rod::SVE {
        if !vanjski.iter().any(|v| v.rod == rod) {
            e.push(format!("vanjski: nema roda {}", rod.kod()));
        }
    }

    // Strani zapisi: barem jedan, i to samo u tržišnoj ulozi.
    let strani: Vec<_> = vanjski.iter().filter(|v| v.zemlja != "HR").collect();
    if strani.is_empty() {
        e.push("vanjski: nema stranog zapisa (zemlja različita od HR)".into());
    }
    for v in &strani {
        if !matches!(
            v.uloga,
            UlogaVanjskog::Novinar
                | UlogaVanjskog::OrganizatorDogadanja
                | UlogaVanjskog::ZastupnikAgencije
                | UlogaVanjskog::PoslovniKlijent
        ) {
            e.push(format!(
                "vanjski: {} je strani ({}), a uloga {} nije tržišna (novinar, organizator_dogadanja, zastupnik_agencije, poslovni_klijent)",
                v.id,
                v.zemlja,
                v.uloga.kod()
            ));
        }
    }

    // Dob u najmanje 3 razreda iz raspodjele.
    let popunjeni = okvir
        .raspodjele
        .vanjski
        .dob
        .keys()
        .filter(|k| {
            let (a, z) = raspon(k);
            vanjski.iter().any(|v| (a..=z).contains(&v.dob))
        })
        .count();
    if popunjeni < 3 {
        e.push(format!(
            "vanjski: dob pokriva {popunjeni} dobnih razreda iz raspodjele, traži se najmanje 3"
        ));
    }

    // Opis i ustanova.
    let bez_opisa = vanjski.iter().filter(|v| v.opis.is_none()).count();
    if bez_opisa > 0 {
        e.push(format!(
            "opis: {bez_opisa} od {} vanjskih bez opisa (traže se svi)",
            vanjski.len()
        ));
    }
    let s_ustanovom = vanjski.iter().filter(|v| v.ustanova.is_some()).count();
    if s_ustanovom < 6 {
        e.push(format!(
            "ustanova: {s_ustanovom} vanjskih ima ustanovu, traži se najmanje 6"
        ));
    }

    assert!(
        e.is_empty(),
        "vanjski primjeri nisu raznoliki ({} prekršaja):\n  - {}",
        e.len(),
        e.join("\n  - ")
    );
}
