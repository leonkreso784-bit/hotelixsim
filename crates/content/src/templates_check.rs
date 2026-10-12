//! Provjera predložaka situacija v2 i imena (S-1). Svaka greška imenuje predložak i polje.

use std::collections::{BTreeMap, BTreeSet};

use sim_hotel::{Catalog, Month};
use sim_hotel_v2::department::UsaliClass;

use crate::ContentError;
use crate::templates::{
    Bounds, Conditions, MajorRule, NameList, Names, Param, Quantity, Scope, Template, TemplateFile,
    TemplateMechanic,
};

fn invalid<T>(msg: String) -> Result<T, ContentError> {
    Err(ContentError::Invalid(msg))
}

/// Provjeri datoteku predložaka i imena prema katalogu (cijene i plaće kategorija) i odjelima v2.
pub(crate) fn check(
    file: &TemplateFile,
    names: &Names,
    base: &Catalog,
    v2: &sim_hotel_v2::Catalog,
) -> Result<(), ContentError> {
    check_names(names)?;
    check_major(&file.major)?;
    let mut seen = BTreeSet::new();
    for t in &file.templates {
        let ctx = format!("predložak {}", t.code);
        if !well_formed_code(&t.code) {
            return invalid(format!(
                "{ctx}: šifra mora biti 'P-' pa velika slova, brojke i crtice"
            ));
        }
        if !seen.insert(t.code.as_str()) {
            return invalid(format!("{ctx}: šifra se ponavlja"));
        }
        check_template(&ctx, t, base, v2)?;
    }
    Ok(())
}

fn check_names(names: &Names) -> Result<(), ContentError> {
    let lists: [(&str, &NameList); 3] = [
        ("guest", &names.guest),
        ("staff", &names.staff),
        ("partner", &names.partner),
    ];
    for (source, list) in lists {
        for (gender, v) in [("f", &list.f), ("m", &list.m)] {
            let ctx = format!("names.{source}.{gender}");
            if v.is_empty() {
                return invalid(format!("{ctx}: popis imena je prazan"));
            }
            let mut seen = BTreeSet::new();
            for n in v {
                if n.trim().is_empty() {
                    return invalid(format!("{ctx}: prazno ime"));
                }
                if !seen.insert(n.as_str()) {
                    return invalid(format!("{ctx}: ime '{n}' se ponavlja"));
                }
            }
        }
    }
    Ok(())
}

fn check_major(m: &MajorRule) -> Result<(), ContentError> {
    if m.high_avg_min > 100 || m.low_avg_max >= m.high_avg_min || m.max_per_month == 0 {
        return invalid(format!(
            "situation_templates.major: treba low_avg_max < high_avg_min <= 100 i max_per_month >= 1 \
             (sada {}, {}, {})",
            m.low_avg_max, m.high_avg_min, m.max_per_month
        ));
    }
    Ok(())
}

fn well_formed_code(code: &str) -> bool {
    code.strip_prefix("P-").is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    })
}

fn is_key(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn check_template(
    ctx: &str,
    t: &Template,
    base: &Catalog,
    v2: &sim_hotel_v2::Catalog,
) -> Result<(), ContentError> {
    if !(1..=4).contains(&t.weight) {
        return invalid(format!("{ctx}: weight {} izvan 1..=4", t.weight));
    }
    let depts = departments(ctx, &t.department, v2)?;
    if t.major.is_some() && t.scope == Scope::All {
        return invalid(format!(
            "{ctx}: veća situacija je za jedan hotel, ne za sve (scope 'svi')"
        ));
    }
    let months = check_conditions(ctx, "when", &t.when, depts.is_some())?;
    if t.chance.base == 0 {
        return invalid(format!("{ctx}: chance.base mora biti barem 1"));
    }
    for b in &t.chance.boost {
        if b.when.is_empty() {
            return invalid(format!("{ctx}: chance.boost bez uvjeta"));
        }
        if b.times < 2 {
            return invalid(format!(
                "{ctx}: chance.boost.times {} mora biti barem 2",
                b.times
            ));
        }
        check_conditions(ctx, "chance.boost", &b.when, depts.is_some())?;
    }
    if t.repeat.max_per_season == 0 {
        return invalid(format!("{ctx}: repeat.max_per_season mora biti barem 1"));
    }
    let allowed = check_roles_and_params(ctx, t, depts.as_ref())?;
    check_texts(ctx, t, &allowed)?;
    if t.rubric.len() != 5 || t.rubric.iter().any(|r| r.trim().is_empty()) {
        return invalid(format!(
            "{ctx}: rubrika mora imati točno 5 nepraznih kriterija"
        ));
    }
    if !(2..=4).contains(&t.answers.len()) {
        return invalid(format!(
            "{ctx}: treba 2–4 ponuđena odgovora, ima ih {}",
            t.answers.len()
        ));
    }
    let band = Band::new(ctx, t)?;
    let mech = MechanicCheck {
        ctx,
        params: &t.params,
        months: &months,
        when_months: !t.when.months.is_empty(),
        raise: band.raise,
    };
    for a in &t.answers {
        let actx = format!("{ctx}: odgovor '{}'", a.label);
        check_criteria(&actx, &a.criteria)?;
        if a.mechanics.is_empty() {
            return invalid(format!("{actx}: nema mehanika"));
        }
        band.check_cost(&actx, &mech.costs(&actx, &a.mechanics)?, base, &months)?;
    }
    if t.unanswered.is_empty() {
        return invalid(format!("{ctx}: unanswered nema mehanika"));
    }
    let uctx = format!("{ctx}: unanswered");
    band.check_cost(&uctx, &mech.costs(&uctx, &t.unanswered)?, base, &months)?;
    Ok(())
}

/// Odjeli koje predložak može dobiti; `None` za cijeli hotel.
fn departments(
    ctx: &str,
    dept: &str,
    v2: &sim_hotel_v2::Catalog,
) -> Result<Option<BTreeSet<String>>, ContentError> {
    let pick = |f: &dyn Fn(&sim_hotel_v2::department::DeptSpec) -> bool| -> BTreeSet<String> {
        v2.departments
            .iter()
            .filter(|(_, s)| f(s))
            .map(|(k, _)| k.as_str().to_string())
            .collect()
    };
    let set = match dept {
        "hotel" => return Ok(None),
        "*staffed" => pick(&|s| s.wage_cents > 0),
        "*guest_facing" => pick(&|s| {
            matches!(
                s.class,
                UsaliClass::Rooms | UsaliClass::FoodBeverage | UsaliClass::OtherOperated
            )
        }),
        _ if dept.starts_with('*') => {
            return invalid(format!("{ctx}: nepoznata skupina odjela '{dept}'"));
        }
        _ if v2.departments.keys().any(|k| k.as_str() == dept) => {
            BTreeSet::from([dept.to_string()])
        }
        _ => return invalid(format!("{ctx}: nepoznat odjel '{dept}'")),
    };
    if set.is_empty() {
        return invalid(format!("{ctx}: skupina '{dept}' nema nijedan odjel"));
    }
    Ok(Some(set))
}

/// Prag jednog polja uvjeta: ime, vrijednost, dopušteni raspon i je li polje odjela.
type Threshold<'a> = (&'a str, &'a Option<Bounds>, (i64, i64), bool);

/// Provjeri uvjete; vraća mjesece u kojima se predložak može pojaviti (prazno = cijela sezona).
fn check_conditions(
    ctx: &str,
    place: &str,
    c: &Conditions,
    has_dept: bool,
) -> Result<Vec<Month>, ContentError> {
    let mut months = Vec::new();
    for code in &c.months {
        let Some(m) = Month::from_code(code) else {
            return invalid(format!("{ctx}: {place}.months: nepoznat mjesec '{code}'"));
        };
        if months.contains(&m) {
            return invalid(format!(
                "{ctx}: {place}.months: mjesec '{code}' se ponavlja"
            ));
        }
        months.push(m);
    }
    let pts = (0, 10_000);
    let count = (0, i64::from(u16::MAX));
    let all: [Threshold; 8] = [
        ("occupancy_bp", &c.occupancy_bp, pts, false),
        ("morale_pts", &c.morale_pts, pts, false),
        ("cash_cents", &c.cash_cents, (i64::MIN, i64::MAX), false),
        ("dept_condition_pts", &c.dept_condition_pts, pts, true),
        ("dept_satisfaction_pts", &c.dept_satisfaction_pts, pts, true),
        ("dept_load_hundredths", &c.dept_load_hundredths, count, true),
        ("dept_staff", &c.dept_staff, count, true),
        ("dept_departures", &c.dept_departures, count, true),
    ];
    for (name, bounds, (lo, hi), dept) in all {
        let Some(b) = bounds else { continue };
        let f = format!("{ctx}: {place}.{name}");
        if dept && !has_dept {
            return invalid(format!("{f}: uvjet odjela traži predložak s odjelom"));
        }
        if b.at_least.is_none() && b.below.is_none() {
            return invalid(format!("{f}: prag nema ni at_least ni below"));
        }
        for v in [b.at_least, b.below].into_iter().flatten() {
            if !(lo..=hi).contains(&v) {
                return invalid(format!("{f}: prag {v} izvan {lo}..={hi}"));
            }
        }
        if let (Some(a), Some(below)) = (b.at_least, b.below)
            && a >= below
        {
            return invalid(format!(
                "{f}: at_least {a} mora biti manji od below {below}"
            ));
        }
    }
    if months.is_empty() {
        months = Month::SEASON.to_vec();
    }
    Ok(months)
}

/// Uloge i mjesta; vraća dopuštena imena mjesta u tekstu.
fn check_roles_and_params(
    ctx: &str,
    t: &Template,
    depts: Option<&BTreeSet<String>>,
) -> Result<BTreeSet<String>, ContentError> {
    let mut allowed = BTreeSet::new();
    for (key, role) in &t.roles {
        if !is_key(key) || t.params.contains_key(key) {
            return invalid(format!(
                "{ctx}: uloga '{key}' (mala slova, brojke, '_'; ne smije biti i u params)"
            ));
        }
        if role.types.iter().any(|x| x.trim().is_empty()) {
            return invalid(format!("{ctx}: uloga '{key}' ima prazan tip"));
        }
        allowed.insert(format!("{key}.ime"));
        if !role.types.is_empty() {
            allowed.insert(format!("{key}.tip"));
        }
    }
    for (key, p) in &t.params {
        let f = format!("{ctx}: params.{key}");
        if !is_key(key) || key == "odjel" {
            return invalid(format!(
                "{f}: ime mjesta (mala slova, brojke, '_'; ne 'odjel')"
            ));
        }
        match p {
            Param::Range([a, b]) if a > b => {
                return invalid(format!("{f}: raspon [{a}, {b}] nije uzlazan"));
            }
            Param::Pick(v) if v.is_empty() => return invalid(format!("{f}: pick je prazan")),
            Param::PricePct(0) | Param::WagePct(0) => {
                return invalid(format!("{f}: postotak mora biti barem 1"));
            }
            Param::ByDepartment(m) => check_by_department(&f, m, depts)?,
            _ => {}
        }
        allowed.insert(key.clone());
    }
    if depts.is_some() {
        allowed.insert("odjel".to_string());
    }
    if let Some(c) = &t.character {
        if !t.roles.contains_key(&c.role) {
            return invalid(format!("{ctx}: lik: uloga '{}' nije u roles", c.role));
        }
        let fields = [&c.brief, &c.opening, &c.fallback_reply];
        if fields.iter().any(|f| f.trim().is_empty()) {
            return invalid(format!("{ctx}: lik ima prazno polje"));
        }
    }
    Ok(allowed)
}

fn check_by_department(
    f: &str,
    m: &BTreeMap<String, String>,
    depts: Option<&BTreeSet<String>>,
) -> Result<(), ContentError> {
    let Some(depts) = depts else {
        return invalid(format!("{f}: tekst po odjelu traži predložak s odjelom"));
    };
    if let Some(extra) = m.keys().find(|k| !depts.contains(*k)) {
        return invalid(format!("{f}: odjel '{extra}' nije među odjelima predloška"));
    }
    if let Some(missing) = depts.iter().find(|d| !m.contains_key(*d)) {
        return invalid(format!("{f}: nema teksta za odjel '{missing}'"));
    }
    if m.values().any(|v| v.trim().is_empty()) {
        return invalid(format!("{f}: prazan tekst"));
    }
    Ok(())
}

fn check_texts(ctx: &str, t: &Template, allowed: &BTreeSet<String>) -> Result<(), ContentError> {
    if t.title.trim().is_empty() || t.description.trim().is_empty() {
        return invalid(format!("{ctx}: naslov i opis ne smiju biti prazni"));
    }
    let mut texts: Vec<(String, &str)> = vec![
        ("title".to_string(), &t.title),
        ("description".to_string(), &t.description),
    ];
    for (i, r) in t.rubric.iter().enumerate() {
        texts.push((format!("rubric[{i}]"), r));
    }
    for (i, a) in t.answers.iter().enumerate() {
        if a.label.trim().is_empty() {
            return invalid(format!("{ctx}: odgovor {i} nema teksta"));
        }
        texts.push((format!("answers[{i}].label"), &a.label));
    }
    if let Some(c) = &t.character {
        texts.push(("character.brief".to_string(), &c.brief));
        texts.push(("character.opening".to_string(), &c.opening));
        texts.push(("character.fallback_reply".to_string(), &c.fallback_reply));
    }
    for (field, text) in texts {
        let found = placeholders(text)
            .map_err(|e| ContentError::Invalid(format!("{ctx}: {field}: {e}")))?;
        if let Some(bad) = found.iter().find(|p| !allowed.contains(*p)) {
            return invalid(format!("{ctx}: {field}: nepoznato mjesto {{{bad}}}"));
        }
    }
    Ok(())
}

/// Imena mjesta `{…}` u tekstu, redom; greška za zagradu bez para.
pub(crate) fn placeholders(text: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut open: Option<String> = None;
    for ch in text.chars() {
        match (ch, open.as_mut()) {
            ('{', None) => open = Some(String::new()),
            ('}', Some(_)) => out.extend(open.take()),
            ('{', Some(_)) | ('}', None) => {
                return Err("vitičasta zagrada bez para".to_string());
            }
            (c, Some(name)) => name.push(c),
            (_, None) => {}
        }
    }
    if open.is_some() {
        return Err("vitičasta zagrada nije zatvorena".to_string());
    }
    Ok(out)
}

fn check_criteria(ctx: &str, crit: &[u8]) -> Result<(), ContentError> {
    if let Some(bad) = crit.iter().find(|n| !(1..=5).contains(*n)) {
        return invalid(format!("{ctx}: kriterij {bad} izvan 1..=5"));
    }
    let unique: BTreeSet<&u8> = crit.iter().collect();
    if unique.len() != crit.len() {
        return invalid(format!("{ctx}: kriterij se ponavlja"));
    }
    Ok(())
}

/// Novčani iznos u mehanici: postotak cijene noćenja ili plaće.
#[derive(Clone, Copy)]
enum Money {
    PricePct(u16),
    WagePct(u16),
}

struct MechanicCheck<'a> {
    ctx: &'a str,
    params: &'a BTreeMap<String, Param>,
    months: &'a [Month],
    when_months: bool,
    raise: [u16; 2],
}

impl MechanicCheck<'_> {
    /// Provjeri mehanike jednog odgovora; vraća njegove novčane iznose.
    fn costs(&self, actx: &str, list: &[TemplateMechanic]) -> Result<Vec<Money>, ContentError> {
        let mut out = Vec::new();
        for m in list {
            match m {
                TemplateMechanic::ServiceRecovery { compensation } => {
                    out.extend(self.money(actx, compensation)?);
                }
                TemplateMechanic::EquipmentRepair { .. } => {}
                TemplateMechanic::StaffRetention {
                    raise_one_pct,
                    bonus,
                } => {
                    let [lo, hi] = self.raise;
                    for v in
                        self.number(actx, "raise_one_pct", raise_one_pct, i64::from(u16::MAX))?
                    {
                        if !(i64::from(lo)..=i64::from(hi)).contains(&v) {
                            return invalid(format!(
                                "{actx}: povišica {v} % izvan pojasa raise_pct [{lo}, {hi}]"
                            ));
                        }
                    }
                    out.extend(self.money(actx, bonus)?);
                }
                TemplateMechanic::GroupBooking {
                    rooms,
                    nights,
                    discount_bp,
                    month,
                    ..
                } => {
                    let max = i64::from(u16::MAX);
                    self.number(actx, "rooms", rooms, max)?;
                    self.number(actx, "nights", nights, max)?;
                    self.number(actx, "discount_bp", discount_bp, 10_000)?;
                    self.group_month(actx, month)?;
                }
                TemplateMechanic::CrisisCommunication { backup } => {
                    out.extend(self.money(actx, backup)?);
                }
            }
        }
        Ok(out)
    }

    fn money(&self, actx: &str, q: &Quantity) -> Result<Option<Money>, ContentError> {
        match q {
            Quantity::Fixed(0) => Ok(None),
            Quantity::Fixed(n) => invalid(format!(
                "{actx}: iznos {n} mora biti 0 ili mjesto iz params (price_pct, wage_pct)"
            )),
            Quantity::Param(name) => match self.params.get(name) {
                Some(Param::PricePct(p)) => Ok(Some(Money::PricePct(*p))),
                Some(Param::WagePct(w)) => Ok(Some(Money::WagePct(*w))),
                Some(_) => invalid(format!(
                    "{actx}: mjesto '{name}' nije iznos (price_pct ili wage_pct)"
                )),
                None => invalid(format!("{actx}: nepoznato mjesto '{name}'")),
            },
        }
    }

    /// Mogući brojevi polja; svaki mora biti u `0..=max`.
    fn number(
        &self,
        actx: &str,
        field: &str,
        q: &Quantity,
        max: i64,
    ) -> Result<Vec<i64>, ContentError> {
        let values = match q {
            Quantity::Fixed(n) => vec![*n],
            Quantity::Param(name) => match self.params.get(name).map(Param::numbers) {
                Some(Some(v)) => v,
                Some(None) => {
                    return invalid(format!(
                        "{actx}: mjesto '{name}' nije broj (range ili pick)"
                    ));
                }
                None => return invalid(format!("{actx}: nepoznato mjesto '{name}'")),
            },
        };
        if let Some(bad) = values.iter().find(|v| !(0..=max).contains(*v)) {
            return invalid(format!("{actx}: {field} {bad} izvan 0..={max}"));
        }
        Ok(values)
    }

    /// Grupa ne smije doći prije mjeseca u kojem se situacija pojavi.
    fn group_month(&self, actx: &str, code: &str) -> Result<(), ContentError> {
        let Some(g) = Month::from_code(code) else {
            return invalid(format!("{actx}: nepoznat mjesec grupe '{code}'"));
        };
        if !self.when_months {
            return invalid(format!(
                "{}: grupa traži when.months (mjeseci prije dolaska grupe)",
                self.ctx
            ));
        }
        if let Some(late) = self.months.iter().find(|m| **m > g) {
            return invalid(format!(
                "{actx}: grupa u mjesecu '{code}' dolazi prije situacije u mjesecu '{}'",
                late.code()
            ));
        }
        Ok(())
    }
}

/// Pojas učinka predloška.
struct Band {
    cost: [u16; 2],
    raise: [u16; 2],
}

impl Band {
    fn new(ctx: &str, t: &Template) -> Result<Band, ContentError> {
        let cost = t.effect_band.cost_night_pct;
        let raise = t.effect_band.raise_pct.unwrap_or([0, 0]);
        if cost[0] > cost[1] {
            return invalid(format!(
                "{ctx}: effect_band.cost_night_pct: pojas [{}, {}] nije uzlazan",
                cost[0], cost[1]
            ));
        }
        if raise[0] > raise[1] {
            return invalid(format!(
                "{ctx}: effect_band.raise_pct: pojas povišice [{}, {}] nije uzlazan",
                raise[0], raise[1]
            ));
        }
        Ok(Band { cost, raise })
    }

    /// Trošak odgovora u postotku cijene noćenja mora biti u pojasu za svaku kategoriju i mjesec.
    fn check_cost(
        &self,
        actx: &str,
        costs: &[Money],
        base: &Catalog,
        months: &[Month],
    ) -> Result<(), ContentError> {
        let [lo, hi] = self.cost.map(i128::from);
        for (cat, spec) in &base.categories {
            for &m in months {
                let price = i128::from(spec.price_ref_cents.get(m));
                let wage = i128::from(spec.wage_cents);
                // Trošak × 100 u centima: postotak × osnovica.
                let cost100: i128 = costs
                    .iter()
                    .map(|c| match c {
                        Money::PricePct(p) => i128::from(*p) * price,
                        Money::WagePct(w) => i128::from(*w) * wage,
                    })
                    .sum();
                if cost100 < lo * price || cost100 > hi * price {
                    let pct = if price > 0 { cost100 / price } else { 0 };
                    return invalid(format!(
                        "{actx}: trošak {pct} % cijene noćenja izvan pojasa [{lo}, {hi}] \
                         (kategorija {}, {})",
                        cat.code(),
                        m.code()
                    ));
                }
            }
        }
        Ok(())
    }
}
