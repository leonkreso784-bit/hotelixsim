//! Načela igre kao provjere sadržaja pri učitavanju s diska (`Snapshot::read_v2_checked`).
//!
//! Sadržaj koji krši načelo ne prolazi: server ne krene i kaže koji predložak krši koje načelo. Današnja kršenja
//! su dug u `templates_principles_debt.rs` koji smije samo padati. Snimak iz dnevnika (`from_value`) se ne
//! provjerava — stara igra se uvijek čita.

use std::collections::{BTreeMap, BTreeSet};

use sim_hotel::{Effects, Month, SituationOutcome};
use sim_hotel_v2::calendar::month_period;
use sim_hotel_v2::situations::{
    DrawnSituation, NumberParam, RoundSituation, Scope, TemplateSet as CoreSet, reward_only,
};
use sim_hotel_v2::{Period, PeriodKind, season_periods, situation_effect};

use crate::templates::{Kind, NameList, Template, TemplateMechanic};
use crate::{Content, ContentError};

/// Najveći dopušteni omjer najduljeg i najkraćeg odgovora, u postocima (🟨 k = 1,5).
pub const LABEL_RATIO_PCT: usize = 150;
/// Najmanji razmak ponavljanja predloška u krugovima.
pub const MIN_GAP_ROUNDS: u8 = 4;
/// Najmanje imena po popisu (izvor × rod).
pub const MIN_NAMES_PER_LIST: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Principle {
    BestAnswer,
    MiddleAnswer,
    RepeatGap,
    NamesPerList,
    LabelLengths,
    BreakdownCondition,
    DeptPlaceholder,
    NamesUnique,
    SilenceNotBetter,
}

impl Principle {
    pub fn name(self) -> &'static str {
        match self {
            Principle::BestAnswer => "najbolji odgovor (kvaliteta 5)",
            Principle::MiddleAnswer => "srednji odgovor (kvaliteta 3 ili 4)",
            Principle::RepeatGap => "razmak ponavljanja barem 4 kruga",
            Principle::NamesPerList => "barem 20 imena po popisu",
            Principle::LabelLengths => "duljine odgovora unutar 1,5 puta",
            Principle::BreakdownCondition => "kvar samo kad je stanje loše ili je uzrok izvana",
            Principle::DeptPlaceholder => "{odjel} samo u zagradi ili iza riječi „odjel”",
            Principle::NamesUnique => "isto ime nije u dva popisa",
            Principle::SilenceNotBetter => "šutnja nikad nije bolja od najlošijeg odgovora",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    pub principle: Principle,
    /// Šifra predloška; za imena `names.<izvor>.<rod>` (NamesPerList) ili `names` (NamesUnique).
    pub code: String,
    pub detail: String,
}

/// Kvar ili popravak: vrsta predloška je kvar ili neka mehanika (odgovor ili neodgovoreno) popravlja opremu.
pub(crate) fn is_breakdown(t: &Template) -> bool {
    t.kind == Kind::Breakdown
        || t.answers
            .iter()
            .flat_map(|a| a.mechanics.iter())
            .chain(t.unanswered.iter())
            .any(|m| matches!(m, TemplateMechanic::EquipmentRepair { .. }))
}

/// Troši li se prostor predloška: `condition_of`, inače odjel predloška ili svi članovi skupine; nijedan ne smije biti
/// neraspoređen, a predložak cijelog hotela bez `condition_of` nema prostor. Vraća `true` (staro ponašanje) kad nema v2
/// ili predloška u jezgri; to je danas nedostižno jer se jezgra gradi iz istih predložaka.
fn wears(c: &Content, core: &CoreSet, t: &Template) -> bool {
    let Some(v2) = &c.v2 else {
        return true;
    };
    let Some(spec) = core.templates.iter().find(|s| s.code == t.code) else {
        return true;
    };
    let spaces: Vec<_> = match (&spec.condition_of, spec.department.members(v2)) {
        (Some(c), _) => vec![c.clone()],
        (None, Some(m)) => m,
        (None, None) => return false,
    };
    !spaces.is_empty()
        && spaces.iter().all(|k| {
            v2.departments.get(k).is_some_and(|d| {
                !matches!(
                    d.class,
                    sim_hotel_v2::department::UsaliClass::Undistributed(_)
                )
            })
        })
}

/// Sva kršenja načela u sadržaju: imena, pa predlošci redom datoteke; rezultat je sortiran.
/// Sadržaj bez predložaka nema kršenja.
pub fn principle_violations(c: &Content) -> Result<Vec<Violation>, ContentError> {
    let Some(tpl) = &c.situation_templates else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    let n = &tpl.names;
    let lists: [(&str, &str, &Vec<String>); 6] = [
        ("guest", "f", &n.guest.f),
        ("guest", "m", &n.guest.m),
        ("staff", "f", &n.staff.f),
        ("staff", "m", &n.staff.m),
        ("partner", "f", &n.partner.f),
        ("partner", "m", &n.partner.m),
    ];
    for (src, gender, v) in lists {
        if v.len() < MIN_NAMES_PER_LIST {
            out.push(Violation {
                principle: Principle::NamesPerList,
                code: format!("names.{src}.{gender}"),
                detail: format!("{} imena, treba barem {MIN_NAMES_PER_LIST}", v.len()),
            });
        }
    }
    names_unique(&[&n.guest, &n.staff, &n.partner], &mut out);
    let core = tpl.to_core();
    for t in &tpl.templates {
        template_violations(c, &core, t, &mut out)?;
    }
    out.sort();
    Ok(out)
}

/// Isto ime u dva od šest popisa (ponavljanje unutar istog izvora, drugi rod, računa se kao drugi popis).
fn names_unique(lists: &[&NameList; 3], out: &mut Vec<Violation>) {
    let labels = ["guest", "staff", "partner"];
    let mut seen: BTreeMap<&str, String> = BTreeMap::new();
    for (l, src) in lists.iter().zip(labels) {
        for (gender, v) in [("f", &l.f), ("m", &l.m)] {
            let place = format!("{src}.{gender}");
            let mut here = BTreeSet::new();
            for name in v {
                if !here.insert(name.as_str()) {
                    continue;
                }
                match seen.get(name.as_str()) {
                    Some(prev) => out.push(Violation {
                        principle: Principle::NamesUnique,
                        code: "names".into(),
                        detail: format!("'{name}' je u {prev} i u {place}"),
                    }),
                    None => {
                        seen.insert(name.as_str(), place.clone());
                    }
                }
            }
        }
    }
}

fn template_violations(
    c: &Content,
    core: &CoreSet,
    t: &Template,
    out: &mut Vec<Violation>,
) -> Result<(), ContentError> {
    let mut push = |principle, detail: String| {
        out.push(Violation {
            principle,
            code: t.code.clone(),
            detail,
        })
    };
    let q: Vec<u8> = t.answers.iter().map(|a| a.quality()).collect();
    if !q.contains(&5) {
        push(Principle::BestAnswer, format!("kvalitete {q:?}"));
    }
    if !q.iter().any(|v| (3..=4).contains(v)) {
        push(Principle::MiddleAnswer, format!("kvalitete {q:?}"));
    }
    if t.repeat.gap_rounds < MIN_GAP_ROUNDS {
        push(
            Principle::RepeatGap,
            format!("razmak {} < {MIN_GAP_ROUNDS}", t.repeat.gap_rounds),
        );
    }
    let lens: Vec<usize> = t.answers.iter().map(|a| a.label.chars().count()).collect();
    if let (Some(&min), Some(&max)) = (lens.iter().min(), lens.iter().max())
        && max * 100 > min * LABEL_RATIO_PCT
    {
        push(
            Principle::LabelLengths,
            format!("duljine odgovora {lens:?}: najdulji {max}, najkraći {min}"),
        );
    }
    if is_breakdown(t)
        && !t.external_cause
        && !(wears(c, core, t) && t.when.dept_condition_pts.is_some_and(|b| b.below.is_some()))
    {
        push(
            Principle::BreakdownCondition,
            "kvar bez uvjeta lošeg stanja odjela (dept_condition_pts.below) i bez external_cause, ili s uvjetom na prostoru koji se ne troši (condition_of)"
                .into(),
        );
    }
    if let Some(field) = dept_placeholder_misuse(t) {
        push(
            Principle::DeptPlaceholder,
            format!("{field}: {{odjel}} nije u zagradi ni iza riječi „odjel”"),
        );
    }
    if let Some(detail) = silence_dominates(c, core, t)? {
        push(Principle::SilenceNotBetter, detail);
    }
    Ok(())
}

/// Polje prvog teksta u kojem je `{odjel}` krivo upotrijebljen.
fn dept_placeholder_misuse(t: &Template) -> Option<String> {
    let mut texts: Vec<(String, &str)> = vec![
        ("title".into(), &t.title),
        ("description".into(), &t.description),
    ];
    for (i, r) in t.rubric.iter().enumerate() {
        texts.push((format!("rubric[{i}]"), r));
    }
    for (i, a) in t.answers.iter().enumerate() {
        texts.push((format!("answers[{i}].label"), &a.label));
    }
    if let Some(ch) = &t.character {
        texts.push(("character.brief".into(), &ch.brief));
        texts.push(("character.opening".into(), &ch.opening));
        texts.push(("character.fallback_reply".into(), &ch.fallback_reply));
    }
    const P: &str = "{odjel}";
    for (field, text) in texts {
        for (at, _) in text.match_indices(P) {
            let (before, after) = (&text[..at], &text[at + P.len()..]);
            if before.ends_with('(') && after.starts_with(')') {
                continue;
            }
            let word = before
                .trim_end()
                .rsplit(char::is_whitespace)
                .next()
                .unwrap_or("");
            if !word.to_lowercase().starts_with("odjel") {
                return Some(field);
            }
        }
    }
    None
}

/// Krugovi u kojima se situacija mjeseca `m` obračunava: mjesečni krug i prvi tjedni krug s tim mjesecom situacija.
fn rounds_of(m: Month) -> Vec<Period> {
    let mut out = vec![month_period(m)];
    out.extend(
        season_periods(PeriodKind::Week)
            .into_iter()
            .find(|p| p.situation_month() == m),
    );
    out
}

/// Opis prve kombinacije kategorije, mjeseca i vrste kruga u kojoj šutnja nije lošija od najlošijeg odgovora.
/// Učinak je obračun v2 (`sim_hotel_v2::situation_effect`: faktor kruga, popravak u prostoru odjela predloška).
fn silence_dominates(
    c: &Content,
    core: &CoreSet,
    t: &Template,
) -> Result<Option<String>, ContentError> {
    let (Some(tpl), Some(v2)) = (&c.situation_templates, &c.v2) else {
        return Ok(None);
    };
    let Some(spec) = core.templates.iter().find(|s| s.code == t.code) else {
        return Ok(None);
    };
    let months: Vec<Month> = if t.when.months.is_empty() {
        Month::SEASON.to_vec()
    } else {
        t.when
            .months
            .iter()
            .filter_map(|m| Month::from_code(m))
            .collect()
    };
    let numbers = spec
        .numbers
        .iter()
        .map(|(k, n)| {
            let v = match n {
                NumberParam::Range(a, _) => *a,
                NumberParam::Pick(v) => v.iter().copied().min().unwrap_or(0),
            };
            (k.clone(), v)
        })
        .collect::<BTreeMap<_, _>>();
    let department = spec.department.members(v2).and_then(|d| d.first().cloned());
    let Some(min_q) = t.answers.iter().map(|a| a.quality()).min() else {
        return Ok(None);
    };
    let bad = |e: String| ContentError::Invalid(format!("predložak {}: {e}", t.code));
    for &category in v2.base.categories.keys() {
        for &m in &months {
            let d = DrawnSituation {
                code: format!("{}.{}", t.code, m.code()),
                template: t.code.clone(),
                period: month_period(m),
                day: 1,
                occurrence: 1,
                slot: 1,
                kind: spec.kind,
                weight: spec.weight,
                scope: Scope::Hotel,
                major: None,
                department: department.clone(),
                roles: BTreeMap::new(),
                numbers: numbers.clone(),
            };
            let inst = tpl.instance(&d, &v2.base, category)?;
            for p in rounds_of(m) {
                let fx = |o: SituationOutcome| -> Result<Effects, ContentError> {
                    let o = if t.no_penalty { reward_only(o) } else { o };
                    let s = RoundSituation {
                        outcome: o,
                        space: department.clone(),
                    };
                    Ok(situation_effect(&s, &p, v2, category)
                        .map_err(|e| bad(format!("{e:?}")))?
                        .effects)
                };
                let silent = fx(SituationOutcome::unanswered(&inst))?;
                for (a, ta) in inst.answers.iter().zip(&t.answers) {
                    if ta.quality() != min_q {
                        continue;
                    }
                    let w = fx(SituationOutcome::from_answer(&inst.code, a))?;
                    let dominates = silent.groups == w.groups
                        && silent.satisfaction_pts >= w.satisfaction_pts
                        && silent.morale_pts >= w.morale_pts
                        && silent.rating >= w.rating
                        && silent.equipment_pts >= w.equipment_pts
                        && silent.cost_cents <= w.cost_cents
                        && silent.extra_wage_cents <= w.extra_wage_cents
                        && silent != w;
                    if dominates {
                        return Ok(Some(format!(
                            "šutnja ({}, {}, {}) nije lošija od odgovora '{}'",
                            category.code(),
                            m.code(),
                            p.kind.code(),
                            ta.label
                        )));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Provjera načela uz popis poznatog duga: novo kršenje ruši; šifra s popisa koja više ne krši ruši
/// („makni ga"); šifre kojih nema u sadržaju (druga mapa) se preskaču. Dug smije samo padati.
pub fn check_principles_with(
    c: &Content,
    known: &[(Principle, &[&str])],
) -> Result<(), ContentError> {
    let found = principle_violations(c)?;
    let mut lines = Vec::new();
    for v in &found {
        let listed = known
            .iter()
            .any(|(p, codes)| *p == v.principle && codes.contains(&v.code.as_str()));
        if !listed {
            lines.push(format!(
                "načelo '{}': {}: {} — novo kršenje; popravi sadržaj (popis duga smije samo padati)",
                v.principle.name(),
                v.code,
                v.detail
            ));
        }
    }
    let existing: BTreeSet<&str> = c
        .situation_templates
        .iter()
        .flat_map(|t| t.templates.iter().map(|x| x.code.as_str()))
        .collect();
    for (p, codes) in known {
        for code in *codes {
            if existing.contains(code)
                && !found.iter().any(|v| v.principle == *p && v.code == *code)
            {
                lines.push(format!(
                    "načelo '{}': {code} više ne krši — makni ga s popisa KNOWN_VIOLATIONS \
                     (crates/content/src/templates_principles_debt.rs)",
                    p.name()
                ));
            }
        }
    }
    if lines.is_empty() {
        return Ok(());
    }
    lines.sort();
    Err(ContentError::Invalid(format!(
        "načela igre:\n{}",
        lines.join("\n")
    )))
}
