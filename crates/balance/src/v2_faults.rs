//! Kvarovi prostora (L1-13 dio 4, A2): koliko kvarova prostora (`TemplateSet::space_fault`) hotel dobije u sezoni uz
//! održavanje svih odjela pojačano / zadano / odgođeno (kao proba `MaintenanceAll`), na zadanom planu i uz put
//! „najbolji". Po profilu medijan po sjemenu: broj kvarova, stanje Smještaja i napredak na kraju sezone. Samo ispis
//! (tablica za Leona, „pravedno": dobro održan hotel ima malo kvarova, zapušten više); ne ruši `--check`.

use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::{
    Catalog, HotelState, Maintenance, Period, PeriodKind, PeriodPlan, Profile, RefTable,
    ResolveError, RoundItem, SeasonResult,
};

use crate::v2::pct;
use crate::v2_answers::{AnswerPath, respond};
use crate::v2_situations::Situations;
use crate::{hundredths, median};

/// Jedan profil: nizovi su redom održavanje pojačano / zadano (plan kakav jest) / odgođeno.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultRow {
    pub profile: Profile,
    /// Izvučenih kvarova prostora u sezoni, medijan po sjemenu.
    pub faults: [i32; 3],
    /// Stanje Smještaja na kraju sezone (0..=10 000), medijan.
    pub rooms_condition: [i32; 3],
    /// Napredak na kraju sezone (stotinke boda), medijan.
    pub progress: [i32; 3],
}

/// Blok A2 za vrstu kruga, profili redom A, B, C.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultReport {
    pub kind: PeriodKind,
    pub rows: Vec<FaultRow>,
}

/// Održavanje svih odjela: pojačano, zadano (`None`, plan kakav jest), odgođeno.
const LEVELS: [Option<Maintenance>; 3] = [
    Some(Maintenance::Intensive),
    None,
    Some(Maintenance::Deferred),
];

/// Sezona na zadanom planu uz održavanje svih odjela `level` (`None` = zadano) i put „najbolji".
fn play_maintenance(
    sits: &Situations,
    cat: &Catalog,
    refs: &RefTable,
    (profile, seed): (Profile, u64),
    kind: PeriodKind,
    level: Option<Maintenance>,
) -> Result<SeasonResult, ResolveError> {
    let mut planner = |s: &HotelState, p: &Period| {
        let mut plan = PeriodPlan::default_for(s, p, cat)?;
        if let Some(m) = level {
            for d in plan.departments.values_mut() {
                d.maintenance = m;
            }
        }
        Ok(plan)
    };
    let mut responder =
        |_: &Period, items: &[RoundItem]| respond(AnswerPath::Best, items, &|i| sits.writable(i));
    sits.responding(
        (profile, seed),
        kind,
        cat,
        refs,
        &mut planner,
        &mut responder,
    )
}

pub(crate) fn fault_report(
    sits: &Situations,
    kind: PeriodKind,
    cat: &Catalog,
    refs_for: &dyn Fn(Profile) -> Result<RefTable, ResolveError>,
    seeds: u64,
) -> Result<FaultReport, ResolveError> {
    let mut rows = Vec::new();
    for profile in Profile::ALL {
        let refs = refs_for(profile)?;
        let mut row = FaultRow {
            profile,
            faults: [0; 3],
            rooms_condition: [0; 3],
            progress: [0; 3],
        };
        for (k, level) in LEVELS.into_iter().enumerate() {
            let (mut f, mut c, mut p) = (Vec::new(), Vec::new(), Vec::new());
            for seed in 0..seeds {
                let r = play_maintenance(sits, cat, &refs, (profile, seed), kind, level)?;
                let n = r
                    .drawn
                    .iter()
                    .flatten()
                    .filter(|d| sits.space_fault(&d.template))
                    .count();
                f.push(i32::try_from(n).unwrap_or(i32::MAX));
                c.push(
                    r.periods
                        .last()
                        .and_then(|(rep, _)| rep.after.departments.get(&DeptCode::rooms()))
                        .map_or(0, |d| d.condition_pts),
                );
                p.push(r.progress_pts);
            }
            row.faults[k] = median(&mut f);
            row.rooms_condition[k] = median(&mut c);
            row.progress[k] = median(&mut p);
        }
        rows.push(row);
    }
    Ok(FaultReport { kind, rows })
}

pub(crate) fn push_faults(s: &mut String, r: &FaultReport) {
    s.push_str("== Kvarovi prostora (A2): zadani plan, najbolji odgovor; održavanje svih odjela pojačano / zadano / odgođeno ==\n");
    let line = |f: &dyn Fn(&FaultRow) -> [String; 3]| -> String {
        r.rows
            .iter()
            .map(|x| {
                let [a, b, c] = f(x);
                format!("{} {a} / {b} / {c}", x.profile.code())
            })
            .collect::<Vec<_>>()
            .join("   ")
    };
    let cond = |x: i32| pct(x).trim_end_matches(" %").to_string();
    s.push_str(&format!(
        "  kvarova po sezoni, medijan: {}\n",
        line(&|x| x.faults.map(|v| v.to_string()))
    ));
    s.push_str(&format!(
        "  stanje Smještaja na kraju, medijan: {}\n",
        line(&|x| x.rooms_condition.map(cond))
    ));
    s.push_str(&format!(
        "  napredak na kraju, medijan: {}\n",
        line(&|x| x.progress.map(hundredths))
    ));
}
