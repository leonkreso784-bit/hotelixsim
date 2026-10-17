//! Banka ljudi (K3, čuvar): svaka puna serija N ≥ 1 radnika i gostiju (001 = R/G-00051…00100, 002 = 00101…00150, …).
//! Raspon serije N je 50·N+1 … 50·N+50. Testovi prolaze kroz SVAKU seriju koja postoji na disku i traže da postoje
//! serije 1 i 2 (`OBVEZNE`); kad se doda serija-003, vrijedi i za nju bez izmjene testa. Čitaju stvarni repo
//! (`Banka::read`, `Okvir::read`); ciljeve i dobne razrede uzimaju iz `raspodjele.json`. Pragovi za 50 zapisa su ovdje
//! (čuvarev prijedlog, vidi k2-tests.md); prekršaji se skupljaju u jedan `panic!`, a svaka poruka nosi put serije.
//! „Nijedan opis se ne ponavlja doslovno" gleda cijelu banku (sve serije, sve tri vrste). Serija 000 ostaje pod
//! `primjeri_su_raznolici`. Nova pravila stroja (Azija, S6) i opća provjera banke su u `ljudi_pravila.rs` i
//! `ljudi_okvir.rs` (`repo_banka_prolazi`).
use content::ljudi::{Banka, Gost, Okvir, Podrijetlo, Radnik, Razina, Rod, Segment, Uloga};
use content::repo_dir;
use content::templates::Names;
use std::collections::{BTreeMap, BTreeSet};

/// Serije koje MORAJU postojati (inače bi petlja bez serije prolazila prazno); uz njih se provjeravaju i sve ostale N ≥ 1 s diska.
const OBVEZNE: [u32; 2] = [1, 2];

/// Broj serije iz puta `<vrsta>/serija-NNN.json`.
fn broj_serije(put: &str) -> u32 {
    put.rsplit_once("serija-")
        .and_then(|(_, r)| r.strip_suffix(".json"))
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("put {put:?} nije oblika '<vrsta>/serija-NNN.json'"))
}

/// Svi N ≥ 1 koji postoje kao radnici ili gosti na disku, uz obvezne; uzlazno.
fn brojevi(b: &Banka) -> Vec<u32> {
    let mut v: BTreeSet<u32> = OBVEZNE.into_iter().collect();
    v.extend(
        b.radnici
            .iter()
            .map(|(p, _)| broj_serije(p))
            .filter(|&n| n >= 1),
    );
    v.extend(
        b.gosti
            .iter()
            .map(|(p, _)| broj_serije(p))
            .filter(|&n| n >= 1),
    );
    v.into_iter().collect()
}

fn put_r(n: u32) -> String {
    format!("radnici/serija-{n:03}.json")
}

fn put_g(n: u32) -> String {
    format!("gosti/serija-{n:03}.json")
}

fn radnici_serije(b: &Banka, n: u32) -> Option<&content::ljudi::Serija<Radnik>> {
    let put = put_r(n);
    b.radnici.iter().find(|(p, _)| *p == put).map(|(_, s)| s)
}

fn gosti_serije(b: &Banka, n: u32) -> Option<&content::ljudi::Serija<Gost>> {
    let put = put_g(n);
    b.gosti.iter().find(|(p, _)| *p == put).map(|(_, s)| s)
}

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

/// Zaglavlje (`serija: N`, od–do) i ID-jevi točno 50·N+1 … 50·N+50 redom; `prefiks` je 'R' ili 'G'.
fn provjeri_cjelovitost<T>(
    put: &str,
    n: u32,
    prefiks: char,
    s: &content::ljudi::Serija<T>,
    id: impl Fn(&T) -> &str,
    e: &mut Vec<String>,
) {
    let (prvi, zadnji) = (50 * n + 1, 50 * n + 50);
    let (od, do_) = (
        format!("{prefiks}-{prvi:05}"),
        format!("{prefiks}-{zadnji:05}"),
    );
    if (s.serija, s.od.as_str(), s.do_.as_str()) != (n, od.as_str(), do_.as_str()) {
        e.push(format!(
            "{put}: zaglavlje serija {} {}–{}, očekivano {n} {od}–{do_}",
            s.serija, s.od, s.do_
        ));
    }
    let ids: Vec<&str> = s.zapisi.iter().map(&id).collect();
    let want: Vec<String> = (prvi..=zadnji)
        .map(|i| format!("{prefiks}-{i:05}"))
        .collect();
    if ids != want {
        e.push(format!(
            "{put}: {} zapisa, ID-jevi moraju biti točno {od}…{do_} redom (ima {:?}…)",
            ids.len(),
            ids.iter().take(3).collect::<Vec<_>>()
        ));
    }
}

#[test]
fn serije_su_pune() {
    // G1 (K3): za svaku seriju N ≥ 1 zaglavlje `serija: N`, raspon 50·N+1 … 50·N+50, točno 50 zapisa, ID-jevi bez rupa.
    let banka = Banka::read(&repo_dir()).expect("banka se čita");
    let mut e: Vec<String> = Vec::new();
    for n in brojevi(&banka) {
        match radnici_serije(&banka, n) {
            None => e.push(format!("nema serija-{n:03}: nema datoteke {}", put_r(n))),
            Some(s) => provjeri_cjelovitost(&put_r(n), n, 'R', s, |r| r.id.as_str(), &mut e),
        }
        match gosti_serije(&banka, n) {
            None => e.push(format!("nema serija-{n:03}: nema datoteke {}", put_g(n))),
            Some(s) => provjeri_cjelovitost(&put_g(n), n, 'G', s, |g| g.id.as_str(), &mut e),
        }
    }
    assert!(
        e.is_empty(),
        "serije nisu cjelovite ({} prekršaja):
  - {}",
        e.len(),
        e.join(
            "
  - "
        )
    );
}

/// Pragovi raznolikosti za 50 radnika jedne serije (K2 T3, razilaženje 2 i 3).
fn raznolikost_radnika(radnici: &[Radnik], okvir: &Okvir) -> Vec<String> {
    let mut e: Vec<String> = Vec::new();
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

    e
}

/// Pragovi raznolikosti za 50 gostiju jedne serije (K2 T3, razilaženje 2 i 4).
fn raznolikost_gostiju(gosti: &[Gost], okvir: &Okvir) -> Vec<String> {
    let mut e: Vec<String> = Vec::new();
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

    e
}

#[test]
fn serije_su_raznolike() {
    // G2 (K3): kao `primjeri_su_raznolici` za 000, ali pragovi za 50 + 50 po SVAKOJ seriji N ≥ 1 (+ K1b M4/M5), a opisi i imena
    // gledaju cijelu banku (sve serije, sve tri vrste).
    let dir = repo_dir();
    let okvir = Okvir::read(&dir).expect("okvir se čita");
    let banka = Banka::read(&dir).expect("banka se čita");
    let mut e: Vec<String> = Vec::new();

    for n in brojevi(&banka) {
        match radnici_serije(&banka, n) {
            None => e.push(format!(
                "nema serija-{n:03}: radnici {} moraju postojati",
                put_r(n)
            )),
            Some(s) => e.extend(
                raznolikost_radnika(&s.zapisi, &okvir)
                    .into_iter()
                    .map(|m| format!("{}: {m}", put_r(n))),
            ),
        }
        match gosti_serije(&banka, n) {
            None => e.push(format!(
                "nema serija-{n:03}: gosti {} moraju postojati",
                put_g(n)
            )),
            Some(s) => e.extend(
                raznolikost_gostiju(&s.zapisi, &okvir)
                    .into_iter()
                    .map(|m| format!("{}: {m}", put_g(n))),
            ),
        }
    }

    // ---------------------------------------------------------------- opis i imena (cijela banka, sve tri vrste)
    let mut opisi: Vec<(String, &str)> = Vec::new(); // (id s putem, opis)
    let mut bez_opisa: Vec<String> = Vec::new();
    let mut puna: Vec<(String, &str, &str)> = Vec::new(); // (put + id, ime, prezime)
    for (put, s) in &banka.radnici {
        for r in &s.zapisi {
            match r.opis.as_deref() {
                Some(o) => opisi.push((format!("{put} {}", r.id), o)),
                None => bez_opisa.push(format!("{put} {}", r.id)),
            }
            puna.push((format!("{put} {}", r.id), &r.ime, &r.prezime));
        }
    }
    for (put, s) in &banka.gosti {
        for g in &s.zapisi {
            match g.opis.as_deref() {
                Some(o) => opisi.push((format!("{put} {}", g.id), o)),
                None => bez_opisa.push(format!("{put} {}", g.id)),
            }
            puna.push((format!("{put} {}", g.id), &g.ime, &g.prezime));
        }
    }
    for (put, s) in &banka.vanjski {
        for v in &s.zapisi {
            if let Some(o) = v.opis.as_deref() {
                opisi.push((format!("{put} {}", v.id), o));
            }
        }
    }
    // Radnici i gosti serija N ≥ 1 moraju imati opis; serija-000 i vanjski se ovdje samo broje u ponavljanje.
    for id in bez_opisa.iter().filter(|x| !x.contains("serija-000")) {
        e.push(format!("opis: {id} je bez opisa (svi moraju imati)"));
    }
    let mut vidjeni: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, o) in &opisi {
        if let Some(prvi) = vidjeni.insert(o, id) {
            e.push(format!("opis: {id} ponavlja doslovno opis iz {prvi}"));
        }
    }
    let imena = sva_imena();
    for (id, ime, prezime) in puna {
        if imena.contains(&format!("{ime} {prezime}").to_lowercase()) {
            e.push(format!(
                "ime: {id} '{ime} {prezime}' je u names.json (guest/staff/partner)"
            ));
        }
    }

    assert!(
        e.is_empty(),
        "serije nisu raznolike ({} prekršaja):
  - {}",
        e.len(),
        e.join(
            "
  - "
        )
    );
}
