//! B3a: putovi odgovora na zadanom planu. Svaki profil i sjeme vrte isti zadani plan (`PeriodPlan::default_for`), a
//! mijenja se samo kako se rješavaju situacije: najbolji, srednji, prepušteno voditelju, šutnja, bez situacija.
//! Pravila prepuštanja zrcale igru (`game`): tri prve situacije kruga su obvezne, ostale ide voditelju uz kvalitetu 2.

use std::cmp::Reverse;

use sim_hotel_v2::{
    Catalog, Period, PeriodKind, PeriodPlan, Profile, RefTable, ResolveError, RoundItem,
    SeasonResult, SeasonSituations, SituationOutcome, SituationTemplate, progress_pts,
    run_season_responding,
};

use crate::v2::pct;
use crate::v2_situations::{Situations, share_bp};
use crate::{hundredths, median};

/// Zrcalo `game::MANDATORY_SITUATIONS` (`command.rs`): prvih toliko situacija kruga student rješava sam.
pub const MANDATORY_SITUATIONS: usize = 3;
/// Zrcalo kvalitete prepuštene situacije u igri (`game` `resolve.rs`; nije izvezena, čuva ju test pariteta).
pub const DELEGATED_QUALITY: u8 = 2;

/// Kako student rješava situacije.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerPath {
    Best,
    Middle,
    Delegated,
    Silent,
    NoSituations,
}

impl AnswerPath {
    pub const ALL: [AnswerPath; 5] = [
        AnswerPath::Best,
        AnswerPath::Middle,
        AnswerPath::Delegated,
        AnswerPath::Silent,
        AnswerPath::NoSituations,
    ];

    pub fn name(self) -> &'static str {
        match self {
            AnswerPath::Best => "najbolji",
            AnswerPath::Middle => "srednji",
            AnswerPath::Delegated => "prepušteno",
            AnswerPath::Silent => "šutnja",
            AnswerPath::NoSituations => "bez situacija",
        }
    }
}

/// Koliko krugova je „prvi dio sezone" za ispis (mjerni izbor 🟨, ne broj igre): tjedno 4 (≈ mjesec), mjesečno 2.
pub fn horizon(kind: PeriodKind) -> usize {
    match kind {
        PeriodKind::Week => 4,
        PeriodKind::Month => 2,
    }
}

/// Odgovor koji put nudi na predlošku: najbolji (najviša kvaliteta, kod jednakih manji indeks) ili srednji.
pub fn offered_choice(path: AnswerPath, t: &SituationTemplate) -> Option<usize> {
    match path {
        AnswerPath::Best => {
            let mut best: Option<(usize, u8)> = None;
            for (i, a) in t.answers.iter().enumerate() {
                if best.is_none_or(|(_, q)| a.quality > q) {
                    best = Some((i, a.quality));
                }
            }
            best.map(|(i, _)| i)
        }
        AnswerPath::Middle => crate::middle_index(t),
        _ => None,
    }
}

/// Ishodi kruga za put. Prepušteno: poredak kao u igri (sidra redom, pa primjerci po važnosti silazno, stabilno);
/// prvih `MANDATORY_SITUATIONS` je obvezno (šutnja), ostale idu voditelju.
fn respond(path: AnswerPath, items: &[RoundItem]) -> Result<Vec<SituationOutcome>, ResolveError> {
    match path {
        AnswerPath::Best | AnswerPath::Middle => items
            .iter()
            .map(|item| {
                let t = item.template;
                let i = offered_choice(path, t).ok_or(ResolveError::BadChoice)?;
                let a = t.answers.get(i).ok_or(ResolveError::BadChoice)?;
                Ok(SituationOutcome::from_answer(&t.code, a))
            })
            .collect(),
        AnswerPath::Silent | AnswerPath::NoSituations => Ok(items
            .iter()
            .map(|i| SituationOutcome::unanswered(i.template))
            .collect()),
        AnswerPath::Delegated => {
            let mut order: Vec<usize> = (0..items.len()).collect();
            order.sort_by_key(|&k| {
                items.get(k).map_or((true, Reverse(0)), |i| match i.drawn {
                    None => (false, Reverse(0)),
                    Some(d) => (true, Reverse(d.weight)),
                })
            });
            let mut out: Vec<Option<SituationOutcome>> = vec![None; items.len()];
            for (rank, &k) in order.iter().enumerate() {
                let t = items.get(k).ok_or(ResolveError::BadChoice)?.template;
                let o = if rank < MANDATORY_SITUATIONS {
                    SituationOutcome::unanswered(t)
                } else {
                    SituationOutcome {
                        code: t.code.clone(),
                        mechanics: t.unanswered.clone(),
                        quality: DELEGATED_QUALITY,
                    }
                };
                *out.get_mut(k).ok_or(ResolveError::BadChoice)? = Some(o);
            }
            out.into_iter()
                .map(|o| o.ok_or(ResolveError::BadChoice))
                .collect()
        }
    }
}

/// Sezona jednog profila na zadanom planu kroz put `path`; isto sjeme `seed` za sve putove.
pub(crate) fn play(
    sits: &Situations,
    cat: &Catalog,
    refs: &RefTable,
    (profile, path, seed): (Profile, AnswerPath, u64),
    kind: PeriodKind,
) -> Result<SeasonResult, ResolveError> {
    let mut planner =
        |s: &sim_hotel_v2::HotelState, p: &Period| Ok(PeriodPlan::default_for(s, p, cat)?);
    let mut responder = |_: &Period, items: &[RoundItem]| respond(path, items);
    if path == AnswerPath::NoSituations {
        return run_season_responding(
            profile,
            kind,
            cat,
            SeasonSituations::Fixed(&[]),
            refs,
            &mut planner,
            &mut responder,
        );
    }
    sits.responding(
        (profile, seed),
        kind,
        cat,
        refs,
        &mut planner,
        &mut responder,
    )
}

/// Jedan profil: medijani po putovima i razlike po sjemenu (stotinke).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathRow {
    pub profile: Profile,
    /// Medijan napretka na kraju sezone, redom `AnswerPath::ALL`.
    pub end: [i32; 5],
    /// Medijan razlike po sjemenu najbolji − šutnja: nakon 1 kruga, nakon `horizon(kind)`, kraj sezone.
    pub best_vs_silent: [i32; 3],
    /// Medijan razlike po sjemenu na kraju: najbolji − srednji, srednji − prepušteno, prepušteno − šutnja.
    pub steps: [i32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathReport {
    pub kind: PeriodKind,
    pub horizon: usize,
    pub rows: Vec<PathRow>,
}

/// Medijan razlike po sjemenu: `a[i] − b[i]` za isto sjeme `i`, tek onda medijan (ulazi se ne sortiraju).
fn median_of_diffs(a: &[i32], b: &[i32]) -> i32 {
    let mut d: Vec<i32> = a.iter().zip(b).map(|(p, q)| p.saturating_sub(*q)).collect();
    median(&mut d)
}

/// Napredak od početka nakon `rounds` krugova (najviše koliko sezona ima).
fn progress_after(r: &SeasonResult, rounds: usize) -> Result<i32, ResolveError> {
    let k = rounds.min(r.periods.len()).saturating_sub(1);
    match r.periods.get(k) {
        Some((_, idx)) => Ok(progress_pts(&r.start, idx)?),
        None => Ok(0),
    }
}

/// Putovi za sve profile i sjemena `0..seeds` na zadanom planu.
pub(crate) fn path_report(
    sits: &Situations,
    kind: PeriodKind,
    cat: &Catalog,
    refs_for: &dyn Fn(Profile) -> Result<RefTable, ResolveError>,
    seeds: u64,
) -> Result<PathReport, ResolveError> {
    let h = horizon(kind);
    let mut rows = Vec::new();
    for profile in Profile::ALL {
        let refs = refs_for(profile)?;
        let mut end: [Vec<i32>; 5] = Default::default();
        let mut at_one: [Vec<i32>; 5] = Default::default();
        let mut at_h: [Vec<i32>; 5] = Default::default();
        for seed in 0..seeds {
            for (k, path) in AnswerPath::ALL.into_iter().enumerate() {
                let r = play(sits, cat, &refs, (profile, path, seed), kind)?;
                if let Some(v) = end.get_mut(k) {
                    v.push(r.progress_pts);
                }
                if let Some(v) = at_one.get_mut(k) {
                    v.push(progress_after(&r, 1)?);
                }
                if let Some(v) = at_h.get_mut(k) {
                    v.push(progress_after(&r, h)?);
                }
            }
        }
        let diff = |a: &[Vec<i32>; 5], x: usize, y: usize| -> i32 {
            median_of_diffs(
                a.get(x).map_or(&[][..], Vec::as_slice),
                a.get(y).map_or(&[][..], Vec::as_slice),
            )
        };
        let mut medians = [0_i32; 5];
        for (m, v) in medians.iter_mut().zip(end.iter()) {
            *m = median(&mut v.clone());
        }
        rows.push(PathRow {
            profile,
            end: medians,
            best_vs_silent: [diff(&at_one, 0, 3), diff(&at_h, 0, 3), diff(&end, 0, 3)],
            steps: [diff(&end, 0, 1), diff(&end, 1, 2), diff(&end, 2, 3)],
        });
    }
    Ok(PathReport {
        kind,
        horizon: h,
        rows,
    })
}

/// Granice težine situacija (🟨 do kalibracije L1-13, ADR-042 t. 15 ih sužava na 15–25 %), bp.
pub const WEIGHT_BAND_BP: (i32, i32) = (500, 5_000);

/// Strop duga (stotinke boda): dugovani korak smije biti nula ili negativan do ove vrijednosti. Stanje 15.10.2026:
/// najgore −14 (mjesečno srednji − prepušteno −14/−11/−10), Leonova odluka. Smije samo rasti prema 0; L1-13 briše dug.
pub const ORDER_DEBT_FLOOR: i32 = -25;

/// Zapisani dug redoslijeda (kao `templates_principles_debt.rs`): koraci lanca koji smiju biti 0 ili negativni
/// do `ORDER_DEBT_FLOOR` u danoj vrsti kruga. Korak 0 = najbolji − srednji, 1 = srednji − prepušteno,
/// 2 = prepušteno − šutnja. Popis smije samo padati; kad korak postane pozitivan na svim profilima, brana
/// „B3 dug" traži da ga se makne. Prazan od B1a (15.10.2026): uz bonus hitnog popravka mjesečni korak srednji −
/// prepušteno je pozitivan na svim profilima (odluka voditelja).
pub const ORDER_DEBT: &[(PeriodKind, usize)] = &[];

/// Ravnalo težine, stotinke boda: Brižni − Štediša bez situacija (planovi robota; medijan
/// `Situations.shares.without` Caring − Saver po profilu), 1000 sjemena, 15.10.2026, commit 76271d1.
/// Leonova odluka (anketa B3, opcija 1); ne prati igru ni robote, mijenja se samo u kalibraciji L1-13.
/// Ponovno mjerenje: `run_v2` (`OptionsV2 { seeds: 1000, kind, situations: None }`) za Week i Month, pa za
/// svaki profil `shares.without(Caring) − shares.without(Saver)` (privremeni test, ne commitati).
pub fn plan_ruler(kind: PeriodKind, profile: Profile) -> i32 {
    match (kind, profile) {
        (PeriodKind::Week, Profile::A) => 845,
        (PeriodKind::Week, Profile::B) => 678,
        (PeriodKind::Week, Profile::C) => 1_033,
        (PeriodKind::Month, Profile::A) => 812,
        (PeriodKind::Month, Profile::B) => 679,
        (PeriodKind::Month, Profile::C) => 1_090,
    }
}

const STEP_NAMES: [(&str, &str); 3] = [
    ("najbolji", "srednji"),
    ("srednji", "prepušteno"),
    ("prepušteno", "šutnja"),
];

/// Brane modela za situacije na zadanom planu (B3b). Ne ovise o robotima: samo putovi odgovora i ravnalo.
pub(crate) fn answer_path_failures(f: &mut Vec<String>, r: &PathReport) {
    let (lo, hi) = WEIGHT_BAND_BP;
    for row in &r.rows {
        let p = row.profile.code();
        let [_, at_h, end] = row.best_vs_silent;
        if at_h <= 0 {
            f.push(format!(
                "B3 odgovori: {p} najbolji odgovori prema šutnji {} nakon {}. kruga (cilj više od 0)",
                hundredths(at_h),
                r.horizon
            ));
        }
        if end <= 0 {
            f.push(format!(
                "B3 odgovori: {p} najbolji odgovori prema šutnji {} na kraju sezone (cilj više od 0)",
                hundredths(end)
            ));
        } else {
            // Δ ≤ 0 je već pala gore; `share_bp` uzima apsolutnu vrijednost pa obrnuta težina ne smije do pojasa.
            let bp = share_bp(end, plan_ruler(r.kind, row.profile));
            if !(lo..=hi).contains(&bp) {
                f.push(format!(
                    "B3 težina: {p} situacije nose {} (cilj {}–{})",
                    pct(bp),
                    pct(lo),
                    pct(hi)
                ));
            }
        }
        for (i, (step, (a, b))) in row.steps.iter().zip(STEP_NAMES).enumerate() {
            let forgiven = *step >= ORDER_DEBT_FLOOR && ORDER_DEBT.contains(&(r.kind, i));
            if *step <= 0 && !forgiven {
                f.push(format!(
                    "B3 redoslijed: {p} {a} prema {b} {} na kraju sezone (cilj više od 0)",
                    hundredths(*step)
                ));
            }
        }
    }
    for &(kind, i) in ORDER_DEBT {
        let paid = !r.rows.is_empty()
            && r.rows
                .iter()
                .all(|row| row.steps.get(i).is_some_and(|s| *s > 0));
        if kind == r.kind && paid {
            let (a, b) = STEP_NAMES.get(i).copied().unwrap_or(("?", "?"));
            f.push(format!(
                "B3 dug: korak {a} prema {b} je pozitivan na svim profilima, makni ga iz ORDER_DEBT (L1-13)"
            ));
        }
    }
}

/// Ispis bloka B3: isti plan i sjeme, mijenja se samo odgovor.
pub(crate) fn push_answer_paths(s: &mut String, paths: &PathReport) {
    s.push_str(
        "== Situacije na zadanom planu (B3): isti plan i sjeme, mijenja se samo odgovor ==\n",
    );
    s.push_str("  napredak na kraju sezone, medijan (najbolji / srednji / prepušteno / šutnja / bez situacija):\n");
    for r in &paths.rows {
        let v: Vec<String> = r.end.iter().map(|x| hundredths(*x)).collect();
        s.push_str(&format!("    {} {}\n", r.profile.code(), v.join(" / ")));
    }
    s.push_str(&format!(
        "  najbolji − šutnja, medijan razlike po sjemenu (1 krug / {} kruga / kraj sezone):\n",
        paths.horizon
    ));
    for r in &paths.rows {
        let v: Vec<String> = r.best_vs_silent.iter().map(|x| hundredths(*x)).collect();
        s.push_str(&format!("    {} {}\n", r.profile.code(), v.join(" / ")));
    }
    s.push_str("  koraci na kraju sezone, medijan razlike po sjemenu (najbolji − srednji / srednji − prepušteno / prepušteno − šutnja):\n");
    for r in &paths.rows {
        let v: Vec<String> = r.steps.iter().map(|x| hundredths(*x)).collect();
        s.push_str(&format!("    {} {}\n", r.profile.code(), v.join(" / ")));
    }
    s.push_str("  težina za anketu (najbolji − šutnja na kraju sezone kao dio raspona plana, ravnalo: Brižni − Štediša bez situacija):\n");
    for r in &paths.rows {
        s.push_str(&format!(
            "    {} prema rasponu plana (Brižni − Štediša bez situacija) {}\n",
            r.profile.code(),
            pct(share_bp(
                r.best_vs_silent[2],
                plan_ruler(paths.kind, r.profile)
            ))
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::median_of_diffs;

    /// Razlika po sjemenu, ne razlika medijana: [1,2,3] − [3,1,2] = [−2,1,1], medijan 1; sortirano bi bilo 0.
    #[test]
    fn median_of_diffs_pairs_by_seed() {
        assert_eq!(median_of_diffs(&[1, 2, 3], &[3, 1, 2]), 1);
    }
}
