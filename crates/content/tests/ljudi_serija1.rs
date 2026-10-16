//! Banka ljudi (K2, čuvar): serija-001 = 50 radnika (R-00051–R-00100) i 50 gostiju (G-00051–G-00100). Raspon serije N je
//! 50·N+1 … 50·N+50. Testovi čitaju stvarni repo (`Banka::read`, `Okvir::read`), ne prepisuju konstante; ciljeve i dobne
//! razrede uzimaju iz `raspodjele.json`. Pragovi za 50 zapisa su ovdje (čuvarev prijedlog, vidi k2-tests.md); svi prekršaji
//! se skupljaju u jedan `panic!`. Nova pravila stroja (Azija, S6) i opća provjera banke su u `ljudi_pravila.rs` i
//! `ljudi_okvir.rs` (`repo_banka_prolazi`).
use content::ljudi::{Banka, Gost, Okvir, Podrijetlo, Radnik, Razina, Rod, Segment, Uloga};
use content::repo_dir;
use content::templates::Names;
use std::collections::{BTreeMap, BTreeSet};

const R: &str = "radnici/serija-001.json";
const G: &str = "gosti/serija-001.json";

fn raspon(k: &str) -> (u8, u8) {
    let (a, b) = k
        .split_once('-')
        .unwrap_or_else(|| panic!("dobni razred {k:?} nije oblika 'od-do'"));
    (a.parse().unwrap(), b.parse().unwrap())
}

/// Riječi malim slovima (bez interpunkcije), za traženje fraza u opisu.
fn rijeci(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

fn sva_imena() -> BTreeSet<String> {
    let text = std::fs::read_to_string(repo_dir().join("v2/names.json")).expect("v2/names.json");
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

fn radnici_001(b: &Banka) -> Option<&content::ljudi::Serija<Radnik>> {
    b.radnici.iter().find(|(p, _)| p == R).map(|(_, s)| s)
}

fn gosti_001(b: &Banka) -> Option<&content::ljudi::Serija<Gost>> {
    b.gosti.iter().find(|(p, _)| p == G).map(|(_, s)| s)
}

#[test]
fn serija_001_ima_pedeset_i_pedeset() {
    // G1 za K2: zaglavlje `serija: 1`, raspon i točno 50 zapisa po vrsti, ID-jevi bez rupa.
    let banka = Banka::read(&repo_dir()).expect("banka se čita");
    let mut e: Vec<String> = Vec::new();

    match radnici_001(&banka) {
        None => e.push(format!("nema serija-001: nema datoteke {R}")),
        Some(s) => {
            if (s.serija, s.od.as_str(), s.do_.as_str()) != (1, "R-00051", "R-00100") {
                e.push(format!(
                    "{R}: zaglavlje serija {} {}–{}, očekivano 1 R-00051–R-00100",
                    s.serija, s.od, s.do_
                ));
            }
            let ids: Vec<&str> = s.zapisi.iter().map(|r| r.id.as_str()).collect();
            let want: Vec<String> = (51..=100).map(|i| format!("R-{i:05}")).collect();
            if ids != want {
                e.push(format!(
                    "{R}: {} zapisa, ID-jevi moraju biti točno R-00051…R-00100 redom (ima {:?}…)",
                    ids.len(),
                    ids.iter().take(3).collect::<Vec<_>>()
                ));
            }
        }
    }
    match gosti_001(&banka) {
        None => e.push(format!("nema serija-001: nema datoteke {G}")),
        Some(s) => {
            if (s.serija, s.od.as_str(), s.do_.as_str()) != (1, "G-00051", "G-00100") {
                e.push(format!(
                    "{G}: zaglavlje serija {} {}–{}, očekivano 1 G-00051–G-00100",
                    s.serija, s.od, s.do_
                ));
            }
            let ids: Vec<&str> = s.zapisi.iter().map(|g| g.id.as_str()).collect();
            let want: Vec<String> = (51..=100).map(|i| format!("G-{i:05}")).collect();
            if ids != want {
                e.push(format!(
                    "{G}: {} zapisa, ID-jevi moraju biti točno G-00051…G-00100 redom (ima {:?}…)",
                    ids.len(),
                    ids.iter().take(3).collect::<Vec<_>>()
                ));
            }
        }
    }
    assert!(
        e.is_empty(),
        "serija-001 nije cjelovita ({} prekršaja):\n  - {}",
        e.len(),
        e.join("\n  - ")
    );
}

#[test]
fn serija_001_je_raznolika() {
    // G2 za K2: kao `primjeri_su_raznolici` za 000, ali pragovi za 50 + 50 (+ K1b M4/M5: razlozi i „prvi put").
    let dir = repo_dir();
    let okvir = Okvir::read(&dir).expect("okvir se čita");
    let banka = Banka::read(&dir).expect("banka se čita");
    let (Some(rs), Some(gs)) = (radnici_001(&banka), gosti_001(&banka)) else {
        panic!("nema serija-001: radnici {R} i gosti {G} moraju postojati");
    };
    let (radnici, gosti) = (&rs.zapisi, &gs.zapisi);
    let mut e: Vec<String> = Vec::new();

    // ---------------------------------------------------------------- radnici
    for u in Uloga::SVE {
        if !radnici.iter().any(|r| r.uloga == u) {
            e.push(format!("radnici: nema uloge {}", u.kod()));
        }
    }
    for k in Razina::SVE {
        if !radnici.iter().any(|r| r.razina == k) {
            e.push(format!("radnici: nema razine {}", k.kod()));
        }
    }
    let pocetnici = radnici
        .iter()
        .filter(|r| r.razina == Razina::Pocetnik)
        .count();
    let stari_pocetnici = radnici
        .iter()
        .filter(|r| r.razina == Razina::Pocetnik && r.dob > 35)
        .count();
    if stari_pocetnici < 2 || stari_pocetnici * 10 < pocetnici {
        e.push(format!(
            "radnici: početnika starijih od 35 je {stari_pocetnici} od {pocetnici} početnika (traži se najmanje 2 i 10 %, BANKA_LJUDI §2.2)"
        ));
    }
    for k in okvir.raspodjele.radnici.dob.keys() {
        let (a, z) = raspon(k);
        let n = radnici.iter().filter(|r| (a..=z).contains(&r.dob)).count();
        if n < 3 {
            e.push(format!(
                "radnici: u dobnom razredu {k} je {n}, traži se najmanje 3"
            ));
        }
    }
    for rod in Rod::SVE {
        let n = radnici.iter().filter(|r| r.rod == rod).count();
        if n < 15 {
            e.push(format!(
                "radnici: rod {} ima {n}, traži se najmanje 15 od 50",
                rod.kod()
            ));
        }
    }
    for (p, najmanje) in [
        (Podrijetlo::Lokalno, 15),
        (Podrijetlo::Regija, 6),
        (Podrijetlo::Inozemstvo, 6),
    ] {
        let n = radnici.iter().filter(|r| r.podrijetlo == p).count();
        if n < najmanje {
            e.push(format!(
                "radnici: podrijetlo {} ima {n}, traži se najmanje {najmanje}",
                p.kod()
            ));
        }
    }
    let strani: Vec<_> = radnici
        .iter()
        .filter(|r| r.podrijetlo == Podrijetlo::Inozemstvo)
        .collect();
    let zemlje_stranih: BTreeSet<&str> = strani.iter().map(|r| r.zemlja.as_str()).collect();
    if zemlje_stranih.len() < 3 {
        e.push(format!(
            "radnici: strani radnici iz {} zemalja {zemlje_stranih:?}, traže se najmanje 3",
            zemlje_stranih.len()
        ));
    }
    let azija = |z: &str| okvir.mjesta.zemlje.get(z).is_some_and(|m| m.azija);
    if !strani.iter().any(|r| azija(&r.zemlja)) {
        e.push("radnici: nema radnika iz Azije (mjesta.json azija: true)".into());
    }
    let regionalna = |z: &str| ["BA", "RS", "ME"].contains(&z);
    if !strani
        .iter()
        .any(|r| regionalna(&r.zemlja) && r.jezici.iter().any(|(j, l)| j == "hr" && *l == 4))
    {
        e.push("radnici: nema radnika iz BA/RS/ME s hrvatskim 4".into());
    }

    // K1b M4: razlozi promjene zanimanja (početnik stariji od 35) ne smiju se ponavljati doslovno.
    let okidaci = ["nakon", "zbog", "jer", "otkako", "kada", "kad", "poslije"];
    let mut razlozi: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for r in radnici
        .iter()
        .filter(|r| r.razina == Razina::Pocetnik && r.dob > 35)
    {
        let w = rijeci(r.opis.as_deref().unwrap_or(""));
        for par in w.windows(2).filter(|p| okidaci.contains(&p[0].as_str())) {
            razlozi
                .entry(par.join(" "))
                .or_default()
                .push(r.id.as_str());
        }
    }
    for (razlog, ids) in &razlozi {
        if ids.len() > 1 {
            e.push(format!(
                "radnici: razlog promjene zanimanja '{razlog}' ponavlja se u {ids:?} (K1b M4)"
            ));
        }
    }

    // ---------------------------------------------------------------- gosti
    let zemlje: BTreeSet<&str> = gosti.iter().map(|g| g.zemlja.as_str()).collect();
    if zemlje.len() < 15 {
        e.push(format!(
            "gosti: {} zemalja {zemlje:?}, traži se najmanje 15",
            zemlje.len()
        ));
    }
    for z in ["AT", "DE", "HR"] {
        if !zemlje.contains(z) {
            e.push(format!("gosti: nema gosta iz zemlje {z}"));
        }
    }
    for sg in Segment::SVE {
        let n = gosti.iter().filter(|g| g.segment == sg).count();
        if n < 3 {
            e.push(format!(
                "gosti: segment {} ima {n}, traži se najmanje 3",
                sg.kod()
            ));
        }
    }
    let zene = gosti.iter().filter(|g| g.rod == Rod::Z).count();
    if !(20..=30).contains(&zene) {
        e.push(format!(
            "gosti: {zene} ž od 50, dopušteno 20–30 (cilj 50 : 50)"
        ));
    }
    for k in okvir.raspodjele.gosti.dob.keys() {
        let (a, z) = raspon(k);
        let n = gosti.iter().filter(|g| (a..=z).contains(&g.dob)).count();
        if n < 3 {
            e.push(format!(
                "gosti: u dobnom razredu {k} je {n}, traži se najmanje 3"
            ));
        }
    }
    let sa_zanimanjem = gosti.iter().filter(|g| g.zanimanje.is_some()).count();
    if sa_zanimanjem < 25 {
        e.push(format!(
            "gosti: zanimanje ima {sa_zanimanjem} od 50, traži se najmanje 25"
        ));
    }

    // K1b M5: „prvi put" uz zemlje s mnogo posjeta (cilj ≥ 600 bp) samo kao „prvi put na Kvarneru".
    for g in gosti {
        let cilj = okvir
            .raspodjele
            .gosti
            .zemlja
            .get(&g.zemlja)
            .copied()
            .unwrap_or(0);
        if cilj < 600 {
            continue;
        }
        let w = rijeci(g.opis.as_deref().unwrap_or(""));
        for i in 0..w.len().saturating_sub(1) {
            if w[i] == "prvi" && w[i + 1] == "put" {
                let kvarner = w[i + 2..].iter().take(3).any(|x| x.starts_with("kvarner"));
                if !kvarner {
                    e.push(format!(
                        "gosti: {} ({}, {} bp) piše 'prvi put' bez 'na Kvarneru' (K1b M5)",
                        g.id, g.zemlja, cilj
                    ));
                }
            }
        }
    }

    // ---------------------------------------------------------------- opis i imena (obje vrste)
    let bez_opisa = radnici.iter().filter(|r| r.opis.is_none()).count()
        + gosti.iter().filter(|g| g.opis.is_none()).count();
    if bez_opisa > 0 {
        e.push(format!(
            "opis: {bez_opisa} od 100 zapisa je bez opisa (svi moraju imati)"
        ));
    }
    let opisi: Vec<&str> = radnici
        .iter()
        .filter_map(|r| r.opis.as_deref())
        .chain(gosti.iter().filter_map(|g| g.opis.as_deref()))
        .collect();
    let razliciti: BTreeSet<&str> = opisi.iter().copied().collect();
    if razliciti.len() != opisi.len() {
        e.push(format!(
            "opis: {} opisa, od toga {} različitih (doslovno ponavljanje)",
            opisi.len(),
            razliciti.len()
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
        "serija-001 nije raznolika ({} prekršaja):\n  - {}",
        e.len(),
        e.join("\n  - ")
    );
}
