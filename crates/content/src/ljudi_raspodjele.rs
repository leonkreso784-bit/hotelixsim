//! Raspodjele banke ljudi: stereotip (BANKA_LJUDI §3.3, S1–S4), izvještaj prema ciljevima iz `raspodjele.json` (§2)
//! i uvjet prekretnice (§3.6, R4). Raspodjela je cilj **cijele banke**, ne svake serije: `Banka::provjeri` gleda samo
//! stereotip; prekretnicu provjerava pečaćenje (K4).

use std::collections::BTreeMap;

use crate::ContentError;
use crate::ljudi::{
    Banka, Gost, Odstupanje, Podrijetlo, Radnik, Raspodjele, Razina, Rod, Segment, Uloga,
};
use crate::ljudi_check::{PREKRETNICA_BP, STEREOTIP_BP, STEREOTIP_MIN_SKUPINA};
use crate::ljudi_tekst::razred;

/// Udio u baznim bodovima, zaokružen na najbliži.
fn bp(c: usize, n: usize) -> u32 {
    if n == 0 {
        return 0;
    }
    let (c, n) = (c as u64, n as u64);
    u32::try_from((c * 20_000 + n) / (2 * n)).unwrap_or(u32::MAX)
}

fn radnici(b: &Banka) -> Vec<&Radnik> {
    b.radnici
        .iter()
        .flat_map(|(_, s)| s.zapisi.iter())
        .collect()
}

fn gosti(b: &Banka) -> Vec<&Gost> {
    b.gosti.iter().flat_map(|(_, s)| s.zapisi.iter()).collect()
}

/// Ključ dobnog razreda iz ciljeva (`"18-24"` …) kojem dob pripada.
fn dobni_razred(dob: u8, razredi: &BTreeMap<String, u32>) -> String {
    razredi
        .keys()
        .find(|k| razred(k).is_some_and(|(a, b)| (a..=b).contains(&dob)))
        .cloned()
        .unwrap_or_else(|| "izvan razreda".to_string())
}

// ---------------------------------------------------------------------------------------------------------------
// Stereotip
// ---------------------------------------------------------------------------------------------------------------

/// Razina, dob i rod moraju imati istu raspodjelu u svakoj skupini podrijetla **unutar uloge** (±`STEREOTIP_BP`).
/// Uloga smije ovisiti o podrijetlu (Leon 16.10., odluka 3), pa se mješavina uloga ne gleda. Ćelija (uloga × podrijetlo)
/// se uspoređuje s cijelom ulogom tek od `STEREOTIP_MIN_SKUPINA` zapisa 🟨; manje ćelije pravilo preskače.
pub(crate) fn stereotip(b: &Banka, r: &Raspodjele) -> Vec<String> {
    let svi = radnici(b);
    let mut out = Vec::new();
    for u in Uloga::SVE {
        let u_svi: Vec<&Radnik> = svi.iter().copied().filter(|x| x.uloga == u).collect();
        let celije = po_podrijetlu(&u_svi);
        let kontekst = format!("; usporedba unutar uloge {}", u.kod());
        for k in Razina::SVE {
            let oznaka = format!("razina {}", k.kod());
            let tekst = Tekst::new(&oznaka, "cijele banke", &kontekst);
            usporedi(&mut out, &tekst, &u_svi, &celije, |x| x.razina == k);
        }
        for k in r.radnici.dob.keys() {
            let Some((a, z)) = razred(k) else { continue };
            let oznaka = format!("dob {k}");
            let tekst = Tekst::new(&oznaka, "cijele banke", &kontekst);
            usporedi(&mut out, &tekst, &u_svi, &celije, |x| {
                (a..=z).contains(&x.dob)
            });
        }
        let oznaka = format!("rod z u ulozi {}", u.kod());
        let tekst = Tekst::new(&oznaka, "cijele uloge", "");
        usporedi(&mut out, &tekst, &u_svi, &celije, |x| x.rod == Rod::Z);
    }
    out
}

/// Dijelovi retka „stereotip:" (oznaka dimenzije, naziv osnovice, dodatak na kraju).
struct Tekst<'a> {
    oznaka: &'a str,
    cijele: &'a str,
    kraj: &'a str,
}

impl<'a> Tekst<'a> {
    fn new(oznaka: &'a str, cijele: &'a str, kraj: &'a str) -> Self {
        Tekst {
            oznaka,
            cijele,
            kraj,
        }
    }
}

fn po_podrijetlu<'a>(svi: &[&'a Radnik]) -> Vec<(Podrijetlo, Vec<&'a Radnik>)> {
    Podrijetlo::SVE
        .iter()
        .map(|p| {
            (
                *p,
                svi.iter().copied().filter(|x| x.podrijetlo == *p).collect(),
            )
        })
        .collect()
}

fn usporedi(
    out: &mut Vec<String>,
    t: &Tekst,
    svi: &[&Radnik],
    skupine: &[(Podrijetlo, Vec<&Radnik>)],
    uvjet: impl Fn(&Radnik) -> bool,
) {
    let cilj = bp(svi.iter().filter(|x| uvjet(x)).count(), svi.len());
    for (p, v) in skupine {
        if v.len() < STEREOTIP_MIN_SKUPINA {
            continue;
        }
        let s = bp(v.iter().filter(|x| uvjet(x)).count(), v.len());
        if s.abs_diff(cilj) > STEREOTIP_BP {
            out.push(format!(
                "stereotip: {}, skupina {}: {s} bp prema {cilj} bp {} (dopušteno ±{STEREOTIP_BP} bp){}",
                t.oznaka,
                p.kod(),
                t.cijele,
                t.kraj
            ));
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Izvještaj i prekretnica
// ---------------------------------------------------------------------------------------------------------------

/// Za svaku dimenziju iz `raspodjele.json` udio svake vrijednosti u banci prema cilju (i vrijednosti kojih nema u
/// ciljevima, s ciljem 0). Dimenzija bez zapisa (npr. nema gostiju) se preskače.
pub fn izvjestaj(b: &Banka, r: &Raspodjele) -> Vec<Odstupanje> {
    let mut out = Vec::new();
    let rad = radnici(b);
    let rr = &r.radnici;
    if !rad.is_empty() {
        let vr = |f: fn(&Radnik) -> &'static str| -> Vec<String> {
            rad.iter().map(|x| f(x).to_string()).collect()
        };
        udjeli(
            &mut out,
            "uloga",
            &kodovi(&rr.uloga, Uloga::kod),
            vr(|x| x.uloga.kod()),
        );
        udjeli(
            &mut out,
            "razina",
            &kodovi(&rr.razina, Razina::kod),
            vr(|x| x.razina.kod()),
        );
        let dob: Vec<String> = rad.iter().map(|x| dobni_razred(x.dob, &rr.dob)).collect();
        udjeli(&mut out, "dob", &rr.dob, dob);
        let pod = vr(|x| x.podrijetlo.kod());
        udjeli(
            &mut out,
            "podrijetlo",
            &kodovi(&rr.podrijetlo, Podrijetlo::kod),
            pod,
        );
        let strani: Vec<String> = rad
            .iter()
            .filter(|x| x.podrijetlo == Podrijetlo::Inozemstvo)
            .map(|x| ili_ostalo(&x.zemlja, &rr.zemlje_inozemstvo))
            .collect();
        udjeli(
            &mut out,
            "zemlja (inozemstvo)",
            &rr.zemlje_inozemstvo,
            strani,
        );
        for (jezik, cilj) in &rr.jezici {
            let cilj: BTreeMap<String, u32> = cilj
                .iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), *v))
                .collect();
            let razine: Vec<String> = rad
                .iter()
                .map(|x| razina_jezika(&x.jezici, jezik))
                .collect();
            udjeli(&mut out, &format!("jezik {jezik}, razina"), &cilj, razine);
        }
        for (u, rod) in &rr.rod_po_ulozi {
            let u_svi: Vec<&&Radnik> = rad.iter().filter(|x| x.uloga == *u).collect();
            if u_svi.is_empty() {
                continue;
            }
            let z = u_svi.iter().filter(|x| x.rod == Rod::Z).count();
            out.push(Odstupanje {
                dimenzija: "rod z u ulozi".into(),
                kljuc: u.kod().into(),
                cilj_bp: rod.z_bp,
                stvarno_bp: bp(z, u_svi.len()),
            });
        }
    }
    let gos = gosti(b);
    let rg = &r.gosti;
    if !gos.is_empty() {
        let zemlje: Vec<String> = gos
            .iter()
            .map(|x| ili_ostalo(&x.zemlja, &rg.zemlja))
            .collect();
        udjeli(&mut out, "gosti: zemlja", &rg.zemlja, zemlje);
        let segment = kodovi(&rg.segment, Segment::kod);
        let vs: Vec<String> = gos.iter().map(|x| x.segment.kod().into()).collect();
        udjeli(&mut out, "gosti: segment", &segment, vs);
        let dob: Vec<String> = gos.iter().map(|x| dobni_razred(x.dob, &rg.dob)).collect();
        udjeli(&mut out, "gosti: dob", &rg.dob, dob);
        let rod = kodovi(&rg.rod, Rod::kod);
        let vs: Vec<String> = gos.iter().map(|x| x.rod.kod().into()).collect();
        udjeli(&mut out, "gosti: rod", &rod, vs);
    }
    out
}

/// Na pečaćenju prekretnice (K4) svaka vrijednost iz izvještaja smije odstupati od cilja najviše `PREKRETNICA_BP`.
pub fn provjeri_prekretnicu(b: &Banka, r: &Raspodjele) -> Result<(), ContentError> {
    if radnici(b).is_empty() && gosti(b).is_empty() {
        return Err(ContentError::Invalid("prekretnica: banka je prazna".into()));
    }
    let retci: Vec<String> = izvjestaj(b, r)
        .into_iter()
        .filter(|o| o.stvarno_bp.abs_diff(o.cilj_bp) > PREKRETNICA_BP)
        .map(|o| {
            format!(
                "prekretnica: {} {} {} bp prema cilju {} bp (dopušteno ±{PREKRETNICA_BP} bp)",
                o.dimenzija, o.kljuc, o.stvarno_bp, o.cilj_bp
            )
        })
        .collect();
    if retci.is_empty() {
        Ok(())
    } else {
        Err(ContentError::Invalid(retci.join("\n")))
    }
}

fn kodovi<K: Copy>(m: &BTreeMap<K, u32>, kod: fn(K) -> &'static str) -> BTreeMap<String, u32> {
    m.iter().map(|(k, v)| (kod(*k).to_string(), *v)).collect()
}

fn ili_ostalo(zemlja: &str, cilj: &BTreeMap<String, u32>) -> String {
    if cilj.contains_key(zemlja) || !cilj.contains_key("ostalo") {
        zemlja.to_string()
    } else {
        "ostalo".to_string()
    }
}

fn razina_jezika(jezici: &[(String, u8)], jezik: &str) -> String {
    let r = jezici
        .iter()
        .filter(|(k, _)| k == jezik)
        .map(|(_, r)| *r)
        .max();
    r.unwrap_or(0).to_string()
}

/// Udio svake vrijednosti među `vrijednosti` prema cilju; ključevi su unija ciljeva i stvarnih vrijednosti.
fn udjeli(
    out: &mut Vec<Odstupanje>,
    dimenzija: &str,
    cilj: &BTreeMap<String, u32>,
    vrijednosti: Vec<String>,
) {
    let n = vrijednosti.len();
    if n == 0 {
        return;
    }
    let mut broj: BTreeMap<String, usize> = cilj.keys().map(|k| (k.clone(), 0)).collect();
    for v in vrijednosti {
        *broj.entry(v).or_insert(0) += 1;
    }
    for (kljuc, c) in broj {
        out.push(Odstupanje {
            dimenzija: dimenzija.to_string(),
            cilj_bp: cilj.get(&kljuc).copied().unwrap_or(0),
            stvarno_bp: bp(c, n),
            kljuc,
        });
    }
}
