//! Obnova v2 riječima za mentora i „Zašto" (obnova-v2-spec §7, ADR-023): ulaganja kruga po dijelu, dobit iz
//! poslovanja nakon ulaganja uz indeks Financije, brže propadanje uz odgođeno održavanje i najamnina umanjena zbog
//! stanja prostora. Sve iz činjenica koje računa Rust (`debrief_v2.rs`); v1 igra ih nema, pa njen tekst ostaje isti.

use serde::Serialize;
use sim_hotel_v2::FinanceBasis;

use crate::ai_input::{DebriefInput, DeferredWearFact, RenewalFact, RentCutFact};
use crate::dto::MonthCode;
use crate::mentor::{ChangeFact, Effect, euros, plural, points, round_div};
use crate::round::RoundKey;

/// Ulaganja kruga za mentora (samo v2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InvestmentFacts {
    #[serde(rename = "obnove", skip_serializing_if = "Vec::is_empty")]
    pub renewals: Vec<RenewalText>,
    /// „s 80 na 79 bodova".
    #[serde(rename = "stanje_zgrade")]
    pub building: String,
}

/// Jedna obnova riječima.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RenewalText {
    #[serde(rename = "dio")]
    pub name: String,
    /// Nova odluka ovog kruga ili zaključana iz ranijeg kruga istog mjeseca.
    #[serde(rename = "odluka")]
    pub decision: &'static str,
    #[serde(rename = "iznos")]
    pub amount: String,
    #[serde(rename = "placeno")]
    pub paid: String,
    #[serde(rename = "stanje_prostora")]
    pub gained: String,
    /// Samo kad bi raspored na dva mjeseca dao više bodova.
    #[serde(
        rename = "u_dvije_mjesecne_faze",
        skip_serializing_if = "Option::is_none"
    )]
    pub split: Option<String>,
    #[serde(rename = "amortizacija")]
    pub depreciation: String,
}

/// Što miče indeks Financije u v2 (uz indeks na ekranu „Zašto"): brojke za sučelje i rečenica za studenta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FinanceNote {
    #[serde(skip)]
    pub depreciation_cents: i64,
    #[serde(skip)]
    pub ebit_cents: i64,
    #[serde(skip)]
    pub capex_cents: i64,
    /// GOP umanjen za amortizaciju obnova („dobit iz poslovanja nakon ulaganja").
    #[serde(skip)]
    pub gop_after_investment_cents: i64,
    #[serde(rename = "objasnjenje")]
    pub text: String,
}

fn is_weekly(input: &DebriefInput) -> bool {
    matches!(input.round, RoundKey::Week(_))
}

fn this_round(input: &DebriefInput) -> &'static str {
    if is_weekly(input) {
        "ovaj tjedan"
    } else {
        "ovaj mjesec"
    }
}

fn month_locative(m: MonthCode) -> &'static str {
    match m {
        MonthCode::Jun => "u lipnju",
        MonthCode::Jul => "u srpnju",
        MonthCode::Aug => "u kolovozu",
        MonthCode::Sep => "u rujnu",
    }
}

/// Bodovi stanja (0–10 000, 100 = 1 bod) s jednom decimalom kad nisu cijeli: „−0,3 boda", „+5 bodova".
/// `None` kad je zaokruženo nula.
fn points_tenths(pts: i64) -> Option<String> {
    let tenths = round_div(pts, 10);
    if tenths == 0 {
        return None;
    }
    if tenths % 10 == 0 {
        return Some(points(tenths / 10));
    }
    let n = tenths.unsigned_abs();
    let sign = if tenths < 0 { '\u{2212}' } else { '+' };
    Some(format!("{sign}{},{} boda", n / 10, n % 10))
}

/// Faktor u baznim bodovima riječima: 12 500 → „1,25".
fn factor_text(bp: u16) -> String {
    let whole = bp / 10_000;
    let rest = format!("{:04}", bp % 10_000);
    let rest = rest.trim_end_matches('0');
    if rest.is_empty() {
        whole.to_string()
    } else {
        format!("{whole},{rest}")
    }
}

/// Plaćanje u krugu: „10.000 € ovaj tjedan, još 3 tjedna"; mjesečna igra plaća cijeli iznos odjednom.
fn paid_text(r: &RenewalFact, weekly: bool) -> String {
    if !weekly {
        return format!(
            "{} ovaj mjesec, cijeli iznos",
            euros(r.paid_this_round_cents, false)
        );
    }
    let n = u64::from(r.rounds_left);
    let rest = if n == 0 {
        "zadnja rata".to_owned()
    } else {
        format!("još {n} {}", plural(n, ["tjedan", "tjedna", "tjedana"]))
    };
    format!(
        "{} ovaj tjedan, {rest}",
        euros(r.paid_this_round_cents, false)
    )
}

/// Jedan dio u planu: „Restoran 40.000 € u srpnju (10.000 € ovaj tjedan, još 3 tjedna)".
fn part_text(r: &RenewalFact, weekly: bool, name: &str) -> String {
    let head = format!(
        "{name} {} {}",
        euros(r.total_cents, false),
        month_locative(r.month)
    );
    let locked = if r.decided_this_round {
        ""
    } else {
        "; odlučeno u ranijem tjednu, zaključano do kraja mjeseca"
    };
    if weekly {
        format!("{head} ({}{locked})", paid_text(r, true))
    } else {
        head
    }
}

/// Obnova u planu kruga riječima (v2 s obnovom): „Restoran 40.000 € u srpnju (10.000 € ovaj tjedan, još 3 tjedna);
/// zgrada ne". `None` kad se u krugu ništa ne obnavlja (ostaje stari tekst).
pub(crate) fn renewal_plan_text(input: &DebriefInput) -> Option<String> {
    if input.renewals.is_empty() {
        return None;
    }
    let weekly = is_weekly(input);
    let parts: Vec<String> = input
        .renewals
        .iter()
        .filter(|r| !r.building)
        .map(|r| part_text(r, weekly, &r.name))
        .collect();
    let departments = if parts.is_empty() {
        "nijedan odjel".to_owned()
    } else {
        parts.join(", ")
    };
    let building = match input.renewals.iter().find(|r| r.building) {
        Some(b) => part_text(b, weekly, "zgrada"),
        None => "zgrada ne".to_owned(),
    };
    Some(format!("{departments}; {building}"))
}

fn depreciation_text(r: &RenewalFact) -> String {
    if r.life_years == 0 || r.depreciation_monthly_cents == 0 {
        return "bez amortizacije".to_owned();
    }
    let y = u64::from(r.life_years);
    format!(
        "{} mjesečno kroz {y} {}, od idućeg mjeseca",
        euros(r.depreciation_monthly_cents, false),
        plural(y, ["godinu", "godine", "godina"])
    )
}

/// „Isti iznos raspoređen na dva mjeseca dao bi +27 bodova umjesto +21 bod" — samo kad bi dao više.
fn split_text(r: &RenewalFact) -> Option<String> {
    let (whole, split) = (
        round_div(i64::from(r.gain_month_pts), 100),
        round_div(i64::from(r.split_gain_pts), 100),
    );
    (split > whole).then(|| {
        format!(
            "isti iznos raspoređen na dva mjeseca dao bi {} umjesto {}",
            points(split),
            points(whole)
        )
    })
}

fn renewal_text(r: &RenewalFact, input: &DebriefInput) -> RenewalText {
    let weekly = is_weekly(input);
    let round = this_round(input);
    let gained = points_tenths(i64::from(r.gained_this_round_pts))
        .unwrap_or_else(|| "bez promjene".to_owned());
    RenewalText {
        name: r.name.clone(),
        decision: match (r.decided_this_round, weekly) {
            (true, true) => "nova odluka ovog tjedna",
            (true, false) => "nova odluka ovog mjeseca",
            (false, _) => {
                "odlučeno u ranijem tjednu istog mjeseca i zaključano do kraja mjeseca; nije nova odluka ovog tjedna"
            }
        },
        amount: format!(
            "{} {}",
            euros(r.total_cents, false),
            month_locative(r.month)
        ),
        paid: paid_text(r, weekly),
        gained: format!(
            "{gained} {round}; cijeli iznos donosi {} u mjesecu (do granice 100)",
            points(round_div(i64::from(r.gain_month_pts), 100))
        ),
        split: split_text(r),
        depreciation: depreciation_text(r),
    }
}

fn building_text(before: i32, after: i32) -> String {
    let (b, a) = (
        round_div(i64::from(before), 100),
        round_div(i64::from(after), 100),
    );
    if a == b {
        format!(
            "ostalo na {a} {}",
            plural(a.unsigned_abs(), ["bod", "boda", "bodova"])
        )
    } else {
        format!(
            "s {b} na {a} {}",
            plural(a.unsigned_abs(), ["bod", "boda", "bodova"])
        )
    }
}

/// Ulaganja kruga za mentora; `None` za v1.
pub(crate) fn investment_facts(input: &DebriefInput) -> Option<InvestmentFacts> {
    let (before, after) = (input.building_before_pts?, input.building_after_pts?);
    Some(InvestmentFacts {
        renewals: input
            .renewals
            .iter()
            .map(|r| renewal_text(r, input))
            .collect(),
        building: building_text(before, after),
    })
}

const GOP_WORDS: &str = "bruto operativna dobit (GOP: ono što od prihoda ostane kad se plate troškovi poslovanja hotela)";

/// Što miče indeks Financije u ovom krugu; `None` za v1.
pub(crate) fn finance_note(input: &DebriefInput) -> Option<FinanceNote> {
    let gop = input.gop_cents?;
    let dep = input.depreciation_round_cents?;
    let ebit = input.ebit_cents?;
    let capex = input.capex_cents?;
    let basis = input.finance_basis?;
    let after = gop.saturating_sub(dep);
    let round = this_round(input);
    let mut text = match basis {
        FinanceBasis::GopAfterInvestment => format!(
            "Indeks Financije gleda dobit iz poslovanja nakon ulaganja: {} {round}. To je {GOP_WORDS} od {}, umanjena za amortizaciju obnova od {} (cijena obnova raspoređena na godine trajanja). Najam zgrade, osiguranje i porez ne ulaze u ovaj indeks.",
            euros(after, false),
            euros(gop, false),
            euros(dep, false)
        ),
        FinanceBasis::Gop => format!(
            "Indeks Financije gleda bruto operativnu dobit (GOP: ono što od prihoda ostane kad se plate troškovi poslovanja hotela): {} {round}. Amortizacija obnova i najam zgrade, osiguranje i porez ne ulaze u ovaj indeks.",
            euros(gop, false)
        ),
        FinanceBasis::Ebit => format!(
            "Indeks Financije gleda dobit nakon najma zgrade, osiguranja, poreza i amortizacije obnova (EBIT): {} {round}.",
            euros(ebit, false)
        ),
    };
    if capex > 0 {
        text.push_str(&format!(
            " Ulaganja od {} {round} plaćena su iz blagajne, ali nisu trošak kruga: u račun dobiti ulaze kroz amortizaciju, od idućeg mjeseca.",
            euros(capex, false)
        ));
    }
    Some(FinanceNote {
        depreciation_cents: dep,
        ebit_cents: ebit,
        capex_cents: capex,
        gop_after_investment_cents: after,
        text,
    })
}

/// Redak „Zašto" za brže propadanje prostora jednog odjela (pokretač Oprema).
fn deferred_row(d: &DeferredWearFact) -> Option<ChangeFact> {
    let amount = points_tenths(i64::from(d.extra_pts))?;
    let factor = factor_text(d.factor_bp);
    Some(ChangeFact {
        cause: format!(
            "zapušten prostor propada brže: {} (održavanje odgođeno)",
            d.name
        ),
        amount: format!("{amount} stanja prostora"),
        // Bodovi stanja odjela, ne opreme sažetka: mala težina, da nikad ne nadjača retke opreme hotela.
        harm: i64::from(d.extra_pts).saturating_neg().max(1),
        effect: Effect::Down,
        because: format!(
            "odgođenog održavanja, zbog kojeg zapušten prostor odjela {} propada {factor} puta brže",
            d.name
        ),
    })
}

/// Redak „Zašto" za najamninu umanjenu zbog stanja prostora (pokretač Prihod).
fn rent_row(r: &RentCutFact) -> Option<ChangeFact> {
    if round_div(r.cut_cents, 100) == 0 {
        return None;
    }
    Some(ChangeFact {
        cause: format!("najamnina umanjena zbog stanja prostora: {}", r.name),
        amount: euros(r.cut_cents.saturating_neg(), true),
        harm: r.cut_cents.max(1),
        effect: Effect::Down,
        because: format!(
            "lošeg stanja prostora odjela {}, zbog kojeg je najamnina manja",
            r.name
        ),
    })
}

/// Dodatni retci „Zašto" iz obnove v2 za pokretač Oprema (brže propadanje).
pub(crate) fn equipment_rows(input: &DebriefInput) -> Vec<ChangeFact> {
    input
        .deferred_wear
        .iter()
        .filter_map(deferred_row)
        .collect()
}

/// Dodatni retci „Zašto" iz obnove v2 za pokretač Prihod (najamnina).
pub(crate) fn revenue_rows(input: &DebriefInput) -> Vec<ChangeFact> {
    input.rent_cuts.iter().filter_map(rent_row).collect()
}

/// Napomene mentora bez AI-ja o obnovi v2 (prije završne rečenice); prazno za v1 i krug bez ičega od ovoga.
pub(crate) fn fallback_notes(input: &DebriefInput) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(plan) = renewal_plan_text(input) {
        let mut s = format!("Obnova: {plan}.");
        for r in &input.renewals {
            if r.decided_this_round {
                s.push_str(&format!(
                    " {}: blagajna plaća cijeli iznos, a račun dobiti nosi samo amortizaciju ({}).",
                    r.name,
                    depreciation_text(r)
                ));
            }
            // Savjet o podjeli samo za obnovu koja je nova odluka; zaključanu više nitko ne može podijeliti.
            if let Some(split) = split_text(r).filter(|_| r.decided_this_round) {
                s.push_str(&format!(" {}: {split}.", r.name));
            }
        }
        out.push(s);
    }
    for d in &input.deferred_wear {
        if let Some(row) = deferred_row(d) {
            out.push(format!(
                "Održavanje odjela {} je odgođeno, pa zapušten prostor propada {} puta brže ({}).",
                d.name,
                factor_text(d.factor_bp),
                row.amount
            ));
        }
    }
    for r in input.rent_cuts.iter().filter(|r| rent_row(r).is_some()) {
        out.push(format!(
            "Najamnina odjela {} umanjena je zbog stanja prostora za {}.",
            r.name,
            euros(r.cut_cents, false)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factor_and_tenths_read_as_words() {
        assert_eq!(factor_text(12_500), "1,25");
        assert_eq!(factor_text(10_000), "1");
        assert_eq!(factor_text(15_000), "1,5");
        assert_eq!(points_tenths(-30).as_deref(), Some("\u{2212}0,3 boda"));
        assert_eq!(points_tenths(500).as_deref(), Some("+5 bodova"));
        assert_eq!(points_tenths(4), None);
    }
}
