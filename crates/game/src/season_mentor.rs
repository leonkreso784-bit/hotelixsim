//! Sezonski mentor (kraj-sezone-spec §6.3, §7): ulaz iz kraja sezone, činjenice cijele sezone u ljudskim jedinicama
//! (obrazac `mentor.rs`, ADR-023) i rezerva bez AI-ja s istim četiri dijela kao uputa `mentor-sezona.v2`.
//! I AI i rezerva pišu samo iz ovih činjenica, pa nijedna interna jedinica (stotinke, centi, kodovi krugova) ne izlazi
//! studentu. Pravilo OT-2: što je student preuzeo i nije mijenjao (plaće, cijeli plan) tako se i kaže.

use engine::calc as c;
use serde::Serialize;
use sim_hotel::{Index, Month};

use crate::dto::MonthCode;
use crate::error::{GameError, corrupt};
use crate::game::Game;
use crate::ids::HotelId;
use crate::mentor::{
    advice, decimal_hundredths, direction, euros, index_name, month_genitive, plural, points,
    round_div, round_locative, thousands, with_sign,
};
use crate::renewal::SeasonRenewal;
use crate::round::RoundKey;
use crate::season::{SeasonDecisions, SeasonFinances, SeasonHotel};
use crate::view::Seat;

// ---------------------------------------------------------------- ulaz

/// Krug sezone za mentora: ključ i prvi/zadnji dan (dan u mjesecu od 1, mjesec).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonRound {
    pub key: RoundKey,
    pub first_date: (u8, MonthCode),
    pub last_date: (u8, MonthCode),
}

/// Hotel na kraju sezone uz uobičajenu cijenu sobe (da se prosječna cijena može usporediti).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonMentorHotel {
    pub hotel: SeasonHotel,
    /// Prosjek uobičajene cijene sobe po krugovima (kategorija hotela u tom krugu), u centima.
    pub price_ref_cents: i64,
}

/// Ulaz za sezonskog mentora jednog ljudskog hotela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeasonDebriefInput {
    pub rounds: Vec<SeasonRound>,
    pub me: SeasonMentorHotel,
    /// Ostali hoteli redom poretka; tuđi ljudski hotel nema financija ni odluka (ARHITEKTURA §12).
    pub others: Vec<SeasonMentorHotel>,
    /// Plaće (% uobičajenih) s kojima je hotel preuzet (profil).
    pub wage_start_pct: u16,
    /// Krugovi u kojima je student sam spremio plan, redom; prazno = plan nikad nije mijenjao.
    pub own_plan_rounds: Vec<RoundKey>,
}

/// Treba li nakon obračuna kruga `r` napisati sezonskog mentora: samo v2 igra i samo njen zadnji krug.
pub fn wants_season_mentor(g: &Game, r: RoundKey) -> bool {
    g.model().require_v2(&g.content).is_ok() && g.rounds.last() == Some(&r)
}

impl Game {
    /// Ulaz za sezonskog mentora hotela `hotel`; iste greške kao `season` (`WrongModel`, `SeasonNotOver`).
    pub fn season_debrief_input(&self, hotel: HotelId) -> Result<SeasonDebriefInput, GameError> {
        let view = self.season(hotel)?;
        let rounds = view
            .rounds
            .iter()
            .map(|i| {
                let (fd, fm) = i.period.first_date();
                let (ld, lm) = i.period.last_date();
                SeasonRound {
                    key: i.key,
                    first_date: (fd, fm.into()),
                    last_date: (ld, lm.into()),
                }
            })
            .collect();
        let mut me = None;
        let mut others = Vec::with_capacity(view.hotels.len());
        for h in view.hotels {
            let x = SeasonMentorHotel {
                price_ref_cents: self.season_price_ref(h.hotel_id)?,
                hotel: h,
            };
            if x.hotel.mine {
                me = Some(x);
            } else {
                others.push(x);
            }
        }
        let me = me.ok_or_else(|| corrupt("kraj sezone bez hotela koji gleda"))?;
        let rec = self.hotel(hotel)?;
        Ok(SeasonDebriefInput {
            rounds,
            me,
            others,
            wage_start_pct: rec.start_state.summary().wage_pct,
            own_plan_rounds: self
                .rounds
                .iter()
                .copied()
                .filter(|r| rec.plans.contains_key(r))
                .collect(),
        })
    }

    /// Prosjek uobičajene cijene sobe po krugovima sezone (isti izvor kao mentor kruga).
    fn season_price_ref(&self, id: HotelId) -> Result<i64, GameError> {
        let calc = |e: c::CalcError| corrupt(format!("sezonski mentor: {e:?}"));
        let mut sum = 0_i128;
        for &r in &self.rounds {
            let category = self.state_at(id, r)?.category();
            let p = self
                .model()
                .price_ref_cents(category, &self.round_info(r)?, &self.content)?;
            sum = c::add(sum, i128::from(p)).map_err(calc)?;
        }
        let n =
            i128::try_from(self.rounds.len()).map_err(|_| corrupt("sezonski mentor: krugovi"))?;
        c::narrow(c::div_round(sum, n.max(1)).map_err(calc)?).map_err(calc)
    }
}

// ---------------------------------------------------------------- činjenice

/// Činjenice cijele sezone; vrijednosti su već u ljudskim jedinicama.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SeasonFacts {
    #[serde(rename = "hotel")]
    pub hotel: String,
    /// „17 tjedana, od 1. lipnja do 30. rujna".
    #[serde(rename = "sezona")]
    pub season: String,
    #[serde(rename = "napredak_od_pocetka")]
    pub progress: String,
    /// „2. mjesto od 4 hotela" (uz dijeljeno mjesto i s kim).
    #[serde(rename = "mjesto")]
    pub place: String,
    /// Pobjednik je prvi po napretku (Leon 13.10.).
    #[serde(rename = "pobjednik")]
    pub winner: String,
    #[serde(rename = "kako_se_usporeduje")]
    pub fairness: &'static str,
    #[serde(rename = "ljestvica_indeksa")]
    pub index_scale: &'static str,
    #[serde(rename = "indeksi")]
    pub indices: Vec<SeasonIndexFact>,
    #[serde(rename = "najbolji_krug", skip_serializing_if = "Option::is_none")]
    pub best_round: Option<RoundFact>,
    #[serde(rename = "najslabiji_krug", skip_serializing_if = "Option::is_none")]
    pub worst_round: Option<RoundFact>,
    #[serde(rename = "financije", skip_serializing_if = "Option::is_none")]
    pub finances: Option<FinanceFacts>,
    #[serde(rename = "odluke", skip_serializing_if = "Option::is_none")]
    pub decisions: Option<DecisionFacts>,
    /// Je li student plan uopće mijenjao (OT-2).
    #[serde(rename = "vlastiti_plan")]
    pub own_plan: String,
    #[serde(rename = "situacije", skip_serializing_if = "Option::is_none")]
    pub situations: Option<String>,
    #[serde(rename = "roboti")]
    pub robots_note: &'static str,
    #[serde(rename = "ostali_hoteli")]
    pub others: Vec<OtherHotelFact>,
    /// Neodgovorene situacije, za rezervu (ne šalje se AI-ju; ista je brojka u `situacije`).
    #[serde(skip)]
    pub unanswered: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SeasonIndexFact {
    #[serde(skip)]
    pub index: Index,
    #[serde(rename = "naziv")]
    pub name: &'static str,
    #[serde(rename = "na_pocetku")]
    pub start: i64,
    #[serde(rename = "na_kraju")]
    pub end: i64,
    /// Razlika zaokruženih vrijednosti, da se slaže s ekranom.
    #[serde(rename = "promjena")]
    pub change: i64,
    #[serde(rename = "smjer")]
    pub direction: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoundFact {
    /// „tjedan od 3. kolovoza do 9. kolovoza" / „srpanj".
    #[serde(rename = "krug")]
    pub round: String,
    #[serde(rename = "promjena_napretka")]
    pub change: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FinanceFacts {
    #[serde(rename = "prihod")]
    pub revenue: String,
    #[serde(rename = "bruto_operativna_dobit")]
    pub gop: String,
    #[serde(
        rename = "udio_dobiti_u_prihodu",
        skip_serializing_if = "Option::is_none"
    )]
    pub gop_share: Option<String>,
    #[serde(rename = "blagajna_na_pocetku")]
    pub cash_start: String,
    #[serde(rename = "blagajna_na_kraju")]
    pub cash_end: String,
    /// Obnova v2: uloženo u sezoni (blagajna), amortizacija sezone i dobit iz poslovanja nakon ulaganja; samo kad je
    /// hotel išta obnovio ili ima amortizaciju, pa sezona bez obnove ostaje kakva je bila.
    #[serde(rename = "ulaganja_u_obnovu", skip_serializing_if = "Option::is_none")]
    pub investments: Option<String>,
    #[serde(
        rename = "amortizacija_sezone",
        skip_serializing_if = "Option::is_none"
    )]
    pub depreciation: Option<String>,
    #[serde(
        rename = "dobit_iz_poslovanja_nakon_ulaganja",
        skip_serializing_if = "Option::is_none"
    )]
    pub gop_after_investment: Option<String>,
}

/// Odluke sezone riječima (prosjeci i brojevi krugova iz obračunatih planova).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DecisionFacts {
    #[serde(rename = "cijena_sobe")]
    pub room_price: String,
    #[serde(rename = "marketing")]
    pub marketing: String,
    #[serde(rename = "sezonski_radnici")]
    pub seasonal: String,
    #[serde(rename = "place")]
    pub wages: String,
    #[serde(rename = "obuka")]
    pub training: String,
    #[serde(rename = "odrzavanje")]
    pub maintenance: String,
    #[serde(rename = "obnova")]
    pub renovation: String,
    #[serde(rename = "odgovori_na_recenzije")]
    pub reply_reviews: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OtherHotelFact {
    #[serde(rename = "ime")]
    pub name: String,
    /// „robot" | „drugi student".
    #[serde(rename = "vrsta")]
    pub kind: &'static str,
    /// Stalno pravilo robota riječima; nema ga za drugog studenta.
    #[serde(rename = "kako_igra", skip_serializing_if = "Option::is_none")]
    pub strategy: Option<&'static str>,
    #[serde(rename = "napredak_od_pocetka")]
    pub progress: String,
    #[serde(rename = "mjesto")]
    pub place: String,
    #[serde(rename = "indeksi")]
    pub indices: Vec<SeasonIndexFact>,
    /// Odluke robota; za drugog studenta se ne pokazuju (ARHITEKTURA §12).
    #[serde(rename = "odluke", skip_serializing_if = "Option::is_none")]
    pub decisions: Option<DecisionFacts>,
}

const FAIRNESS: &str = "Svaki hotel je krenuo s drugačijeg mjesta, zato se uspoređuje napredak od vlastitog početka, ne zarada.";
const ROBOTS_NOTE: &str = "Roboti igraju po stalnom pravilu: odluke i odgovore na situacije biraju uvijek na isti način, ne razmišljaju o pojedinoj situaciji.";
const INDEX_SCALE_WEEK: &str = "Indeksi idu do 100 bodova; 50 bodova je prosjek usporedivog hotela u istom tjednu. Indeks pokazuje kako hotel stoji u odnosu na njega.";
const INDEX_SCALE_MONTH: &str = "Indeksi idu do 100 bodova; 50 bodova je prosjek usporedivog hotela u istom mjesecu. Indeks pokazuje kako hotel stoji u odnosu na njega.";

/// Kako robot igra (kraj-sezone-spec §4, prema `sim-hotel-v2/src/robots.rs`); isti tekst prikazuje ekran kraja sezone.
fn strategy_words(code: &str) -> Option<&'static str> {
    Some(match code {
        "stedisa" => {
            "Štediša je cijelu sezonu čuvao novac: cijena sobe uobičajena, bez marketinga, pet sezonskih radnika manje nego što takav hotel obično zapošljava, plaće 95 % uobičajenih, bez obuke i s odgođenim održavanjem. U situacijama je birao najjeftiniji odgovor."
        }
        "brizni" => {
            "Brižni je ulagao u ljude i goste: cijena sobe 5 % iznad uobičajene, 5.000 € marketinga mjesečno, sezonskih radnika koliko traži očekivana popunjenost (u tjednoj igri zapošljava prije gužve i otpušta prije pada), plaće 103 % uobičajenih, osnovna obuka i redovno održavanje. U situacijama je birao najbolji odgovor."
        }
        "lovac" => {
            "Lovac na popunjenost htio je puni hotel: cijena sobe 15 % ispod uobičajene, 25.000 € marketinga mjesečno, isti broj sezonskih radnika cijelu sezonu, uobičajene plaće, bez obuke, redovno održavanje. U situacijama je prihvaćao grupe s najvećim popustom, a inače birao srednji odgovor."
        }
        "nasumicni" => "Nasumični je svaki krug izvlačio odluke nasumce.",
        _ => return None,
    })
}

fn is_weekly(input: &SeasonDebriefInput) -> bool {
    input
        .rounds
        .first()
        .is_some_and(|r| matches!(r.key, RoundKey::Week(_)))
}

fn month_nominative(m: Month) -> &'static str {
    match m {
        Month::Jun => "lipanj",
        Month::Jul => "srpanj",
        Month::Aug => "kolovoz",
        Month::Sep => "rujan",
    }
}

fn date(d: (u8, MonthCode)) -> String {
    format!("{}. {}", d.0, month_genitive(d.1))
}

/// Krug riječima: (nominativ, lokativ) — „tjedan od 3. kolovoza do 9. kolovoza" / „u tjednu od …", „srpanj" / „u srpnju".
fn round_words(input: &SeasonDebriefInput, key: RoundKey) -> (String, String) {
    match key {
        RoundKey::Month(m) => (
            month_nominative(m).to_owned(),
            round_locative(key).to_owned(),
        ),
        RoundKey::Week(_) => match input.rounds.iter().find(|r| r.key == key) {
            Some(r) => {
                let span = format!("od {} do {}", date(r.first_date), date(r.last_date));
                (format!("tjedan {span}"), format!("u tjednu {span}"))
            }
            None => ("jedan tjedan".to_owned(), "u jednom tjednu".to_owned()),
        },
    }
}

/// Napredak u desetinkama boda („+0,6 boda", „−1,2 boda"); cijeli broj kao `points` („+3 boda").
fn points_tenths(hundredths: i64) -> String {
    let t = round_div(hundredths, 10);
    let a = t.unsigned_abs();
    if a.is_multiple_of(10) {
        return points(t / 10);
    }
    with_sign(t, format!("{},{} boda", thousands(a / 10), a % 10), true)
}

/// Napredak s `digits` decimala (0–2), bez suvišnih nula: „+8 bodova", „+7,6 boda", „+8,42 boda".
fn points_with(hundredths: i32, digits: u32) -> String {
    let digits = digits.min(2);
    let v = round_div(i64::from(hundredths), 10_i64.pow(2 - digits));
    let scale = 10_u64.pow(digits);
    let a = v.unsigned_abs();
    if a.is_multiple_of(scale) {
        let w = v / 10_i64.pow(digits);
        return if w == 0 {
            "0 bodova".to_owned()
        } else {
            points(w)
        };
    }
    let frac = format!(
        "{:0width$}",
        a % scale,
        width = usize::try_from(digits).unwrap_or(2)
    );
    let frac = frac.trim_end_matches('0');
    with_sign(v, format!("{},{frac} boda", thousands(a / scale)), true)
}

/// Koliko decimala napretka treba da hoteli s različitim napretkom (pa i mjestom) ne izgledaju jednako: cijeli bodovi
/// kao na ekranu, a desetinke ili stotinke samo kad bi se zaokruženo poklopili (+8,4 i +7,6 nisu oba „+8 bodova").
fn progress_digits(input: &SeasonDebriefInput) -> u32 {
    let all: Vec<i32> = std::iter::once(&input.me)
        .chain(&input.others)
        .map(|h| h.hotel.progress_hundredths)
        .collect();
    (0..2)
        .find(|&d| {
            all.iter().all(|&a| {
                all.iter()
                    .all(|&b| a == b || points_with(a, d) != points_with(b, d))
            })
        })
        .unwrap_or(2)
}

/// „u 6 tjedana" / „u 2 mjeseca" / „u 1 tjednu".
fn in_rounds(n: u8, weekly: bool) -> String {
    let forms = if weekly {
        ["tjednu", "tjedna", "tjedana"]
    } else {
        ["mjesecu", "mjeseca", "mjeseci"]
    };
    format!("u {n} {}", plural(u64::from(n), forms))
}

fn every_round(weekly: bool) -> &'static str {
    if weekly {
        "svaki tjedan"
    } else {
        "svaki mjesec"
    }
}

/// Koliko krugova sezone je nešto vrijedilo: nijednom / svaki krug / u N krugova.
fn rounds_text(n: u8, total: usize, weekly: bool, none: &str, prefix: &str) -> String {
    if n == 0 {
        none.to_owned()
    } else if usize::from(n) == total {
        format!("{prefix}{}", every_round(weekly))
    } else {
        format!("{prefix}{}", in_rounds(n, weekly))
    }
}

fn index_facts(h: &SeasonHotel) -> Vec<SeasonIndexFact> {
    Index::ALL
        .iter()
        .zip(h.start.iter().zip(&h.end))
        .map(|(&index, (&s, &e))| {
            let start = round_div(i64::from(s), 100);
            let end = round_div(i64::from(e), 100);
            SeasonIndexFact {
                index,
                name: index_name(index),
                start,
                end,
                change: end - start,
                direction: direction(end - start),
            }
        })
        .collect()
}

fn place_of(h: &SeasonHotel, total: usize) -> String {
    format!("{}. mjesto od {total} hotela", h.rank)
}

/// „Brižni (robot)" / „Ana (drugi student)".
fn label(h: &SeasonHotel) -> String {
    match h.seat {
        Seat::Robot { .. } => format!("{} (robot)", h.name),
        Seat::Human => format!("{} (drugi student)", h.name),
    }
}

fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} i {last}", init.join(", ")),
    }
}

fn with_hotels(names: &[String]) -> String {
    if names.len() == 1 {
        format!("s hotelom {}", join_and(names))
    } else {
        format!("s hotelima {}", join_and(names))
    }
}

fn winner_text(input: &SeasonDebriefInput, digits: u32) -> String {
    let others: Vec<&SeasonHotel> = input
        .others
        .iter()
        .map(|o| &o.hotel)
        .filter(|h| h.winner)
        .collect();
    let names: Vec<String> = others.iter().map(|h| label(h)).collect();
    match (input.me.hotel.winner, others.first()) {
        (true, None) => "Tvoj hotel je pobijedio: napredovao je najviše od svih.".to_owned(),
        (true, Some(_)) => format!(
            "Tvoj hotel je pobijedio zajedno {}: napredovali su jednako, najviše od svih.",
            with_hotels(&names)
        ),
        (false, Some(w)) => {
            let verb = if names.len() == 1 {
                "Pobijedio je"
            } else {
                "Pobijedili su"
            };
            format!(
                "{verb} {} s napretkom od {}.",
                join_and(&names),
                points_with(w.progress_hundredths, digits)
            )
        }
        // Ne događa se (netko je uvijek prvi), ali tekst ostaje istinit.
        (false, None) => "Pobjednik je hotel s najvećim napretkom od početka.".to_owned(),
    }
}

fn wage_text(d: &SeasonDecisions, start: Option<u16>) -> String {
    let (lo, hi) = (d.wage_pct_min, d.wage_pct_max);
    let level = if lo == hi {
        format!("{lo} % uobičajene plaće cijelu sezonu")
    } else {
        format!("od {lo} % do {hi} % uobičajene plaće")
    };
    match start {
        None => level,
        Some(s) if lo == hi && lo == s => {
            format!("{level} (takve su od početka, student ih nije mijenjao)")
        }
        Some(s) => format!("{level} (na početku igre bile su {s} %)"),
    }
}

fn decision_facts(
    d: &SeasonDecisions,
    price_ref: i64,
    input: &SeasonDebriefInput,
    wage_start: Option<u16>,
) -> DecisionFacts {
    let weekly = is_weekly(input);
    let total = input.rounds.len();
    let reference = price_ref.max(1);
    let off = round_div((d.avg_price_cents - reference) * 100, reference);
    let rel = match off.signum() {
        0 => "kao uobičajena cijena".to_owned(),
        1 => format!("{off} % iznad uobičajene cijene"),
        _ => format!("{} % ispod uobičajene cijene", off.unsigned_abs()),
    };
    let n = u64::from(d.seasonal_peak);
    DecisionFacts {
        room_price: format!(
            "prosječno {} po noći, {rel}",
            euros(d.avg_price_cents, false)
        ),
        marketing: if d.marketing_total_cents == 0 {
            "bez marketinga cijelu sezonu".to_owned()
        } else {
            format!("ukupno {} u sezoni", euros(d.marketing_total_cents, false))
        },
        seasonal: if n == 0 {
            "bez sezonskih radnika".to_owned()
        } else {
            format!(
                "najviše {n} {} istodobno",
                plural(
                    n,
                    ["sezonski radnik", "sezonska radnika", "sezonskih radnika"]
                )
            )
        },
        wages: wage_text(d, wage_start),
        training: rounds_text(
            d.training_rounds,
            total,
            weekly,
            "bez obuke cijelu sezonu",
            "obuka ",
        ),
        maintenance: rounds_text(
            d.maintenance_deferred_rounds,
            total,
            weekly,
            "nijednom odgođeno",
            "odgođeno ",
        ),
        renovation: match (renewals_text(&d.renewals), d.renovation_round) {
            (Some(text), _) => text,
            (None, Some(r)) => format!("pokrenuta {}", round_words(input, r).1),
            (None, None) => "ne".to_owned(),
        },
        reply_reviews: rounds_text(d.reply_reviews_rounds, total, weekly, "nikad", ""),
    }
}

/// Obnove v2 sezone po dijelu i mjesecu: „Restoran 40.000 € u srpnju, Zgrada 100.000 € u lipnju"; `None` bez obnova.
fn renewals_text(renewals: &[SeasonRenewal]) -> Option<String> {
    if renewals.is_empty() {
        return None;
    }
    let parts: Vec<String> = renewals
        .iter()
        .map(|r| {
            format!(
                "{} {} {}",
                r.asset_name,
                euros(r.amount_cents, false),
                round_locative(RoundKey::Month(r.month))
            )
        })
        .collect();
    Some(parts.join(", "))
}

fn finance_facts(f: &SeasonFinances, renewals: &[SeasonRenewal]) -> FinanceFacts {
    let invested: i64 = renewals
        .iter()
        .fold(0_i64, |s, r| s.saturating_add(r.amount_cents));
    let with_renewal = invested != 0 || f.depreciation_cents != 0;
    FinanceFacts {
        investments: with_renewal.then(|| {
            format!(
                "ukupno {} (plaćeno iz blagajne; to nije trošak sezone, nego se kroz amortizaciju raspoređuje na godine trajanja obnove)",
                euros(invested, false)
            )
        }),
        depreciation: with_renewal.then(|| {
            format!(
                "{} (dio cijene obnova koji pada na ovu sezonu, od mjeseca nakon obnove)",
                euros(f.depreciation_cents, false)
            )
        }),
        gop_after_investment: with_renewal.then(|| {
            format!(
                "{} (bruto operativna dobit umanjena za amortizaciju obnova; to gleda indeks Financije, a najam zgrade, osiguranje i porez ne ulaze)",
                euros(f.gop_cents.saturating_sub(f.depreciation_cents), false)
            )
        }),
        revenue: euros(f.total_revenue_cents, false),
        gop: format!(
            "{} (GOP: ono što od prihoda ostane kad se plate troškovi poslovanja hotela)",
            euros(f.gop_cents, false)
        ),
        gop_share: f
            .gop_margin_bp
            .map(|bp| format!("{} % prihoda", round_div(bp, 100))),
        cash_start: euros(f.cash_start_cents, false),
        cash_end: euros(f.cash_end_cents, false),
    }
}

fn situations_text(d: &SeasonDecisions) -> Option<String> {
    if d.situations == 0 {
        return None;
    }
    let forms = ["situacija", "situacije", "situacija"];
    let n = u64::from(d.situations);
    let mut out = format!("{n} {}", plural(n, forms));
    if let Some(q) = d.avg_quality_hundredths {
        out.push_str(&format!(
            ", prosječna ocjena {}/5",
            decimal_hundredths(u64::from(q))
        ));
    }
    if d.unanswered > 0 {
        let u = u64::from(d.unanswered);
        out.push_str(&format!(", bez odgovora {u} {}", plural(u, forms)));
    }
    Some(out)
}

fn own_plan_text(input: &SeasonDebriefInput) -> String {
    let weekly = is_weekly(input);
    let Some(&first) = input.own_plan_rounds.first() else {
        return "Student nijednom nije mijenjao plan: cijelu sezonu vrijedio je zadani plan s kojim je hotel preuzet, pa te odluke nisu njegove promjene.".to_owned();
    };
    let n = u8::try_from(input.own_plan_rounds.len()).unwrap_or(u8::MAX);
    let mut out = format!(
        "Student je sam postavio plan {}, prvi put {}; poslije toga njegov se plan prenosio iz kruga u krug, prilagođen granicama svakog kruga (obnova se ne ponavlja).",
        in_rounds(n, weekly),
        round_words(input, first).1
    );
    if input.rounds.first().map(|r| r.key) != Some(first) {
        out.push_str(" Prije toga vrijedio je zadani plan s kojim je hotel preuzet.");
    }
    out
}

/// Najbolji (najveći skok) ili najslabiji (najveći pad) krug po napretku; samo kad je pomak bar desetinka boda.
fn extreme_round(input: &SeasonDebriefInput, best: bool) -> Option<RoundFact> {
    let mut prev = 0_i64;
    let mut pick: Option<(RoundKey, i64)> = None;
    for p in &input.me.hotel.curve {
        let now = i64::from(p.progress_hundredths);
        let d = now.saturating_sub(prev);
        prev = now;
        let better = match pick {
            None => true,
            Some((_, x)) => (best && d > x) || (!best && d < x),
        };
        if better {
            pick = Some((p.round, d));
        }
    }
    let (round, d) = pick?;
    let tenths = round_div(d, 10);
    if (best && tenths <= 0) || (!best && tenths >= 0) {
        return None;
    }
    Some(RoundFact {
        round: round_words(input, round).0,
        change: points_tenths(d),
    })
}

fn season_text(input: &SeasonDebriefInput) -> String {
    let n = u64::try_from(input.rounds.len()).unwrap_or(0);
    let unit = if is_weekly(input) {
        plural(n, ["tjedan", "tjedna", "tjedana"])
    } else {
        plural(n, ["mjesec", "mjeseca", "mjeseci"])
    };
    match (input.rounds.first(), input.rounds.last()) {
        (Some(a), Some(b)) => format!(
            "{n} {unit}, od {} do {}",
            date(a.first_date),
            date(b.last_date)
        ),
        _ => format!("{n} {unit}"),
    }
}

pub fn season_debrief_facts(input: &SeasonDebriefInput) -> SeasonFacts {
    let me = &input.me.hotel;
    let digits = progress_digits(input);
    let total = input.others.len() + 1;
    let shared: Vec<String> = input
        .others
        .iter()
        .map(|o| &o.hotel)
        .filter(|h| h.rank == me.rank)
        .map(label)
        .collect();
    let mut place = place_of(me, total);
    if !shared.is_empty() {
        place.push_str(&format!(", dijeljeno {}", with_hotels(&shared)));
    }
    let others = input
        .others
        .iter()
        .map(|o| {
            let h = &o.hotel;
            let (kind, strategy) = match &h.seat {
                Seat::Robot { strategy } => ("robot", strategy_words(strategy)),
                Seat::Human => ("drugi student", None),
            };
            OtherHotelFact {
                name: h.name.clone(),
                kind,
                strategy,
                progress: points_with(h.progress_hundredths, digits),
                place: place_of(h, total),
                indices: index_facts(h),
                decisions: h
                    .decisions
                    .as_ref()
                    .map(|d| decision_facts(d, o.price_ref_cents, input, None)),
            }
        })
        .collect();
    SeasonFacts {
        hotel: me.name.clone(),
        season: season_text(input),
        progress: points_with(me.progress_hundredths, digits),
        place,
        winner: winner_text(input, digits),
        fairness: FAIRNESS,
        index_scale: if is_weekly(input) {
            INDEX_SCALE_WEEK
        } else {
            INDEX_SCALE_MONTH
        },
        indices: index_facts(me),
        best_round: extreme_round(input, true),
        worst_round: extreme_round(input, false),
        finances: me.finances.as_ref().map(|f| {
            finance_facts(
                f,
                me.decisions.as_ref().map_or(&[][..], |d| &d.renewals[..]),
            )
        }),
        decisions: me.decisions.as_ref().map(|d| {
            decision_facts(
                d,
                input.me.price_ref_cents,
                input,
                Some(input.wage_start_pct),
            )
        }),
        own_plan: own_plan_text(input),
        situations: me.decisions.as_ref().and_then(situations_text),
        robots_note: ROBOTS_NOTE,
        others,
        unanswered: me.decisions.as_ref().map_or(0, |d| d.unanswered),
    }
}

// ---------------------------------------------------------------- mentor bez AI-ja

/// Rečenica usporedbe s najboljim robotom ispred studenta: njegove odluke i indeks u kojem je najviše odmakao.
fn robot_comparison(input: &SeasonDebriefInput) -> Option<String> {
    let me = &input.me.hotel;
    let ahead = input.others.iter().map(|o| &o.hotel).find(|h| {
        matches!(h.seat, Seat::Robot { .. }) && h.progress_hundredths > me.progress_hundredths
    })?;
    let d = ahead.decisions.as_ref()?;
    let weekly = is_weekly(input);
    let wages = if d.wage_pct_min == d.wage_pct_max {
        format!("plaće {} % uobičajenih", d.wage_pct_min)
    } else {
        format!(
            "plaće od {} % do {} % uobičajenih",
            d.wage_pct_min, d.wage_pct_max
        )
    };
    let training = rounds_text(
        d.training_rounds,
        input.rounds.len(),
        weekly,
        "bez obuke",
        "obuku ",
    );
    let marketing = if d.marketing_total_cents == 0 {
        "bez marketinga".to_owned()
    } else {
        format!("marketing ukupno {}", euros(d.marketing_total_cents, false))
    };
    let decided = format!("{} je imao {wages}, {training} i {marketing}", label(ahead));
    let (theirs, mine) = (index_facts(ahead), index_facts(me));
    // Samo indeks u kojem je robot stvarno odmakao; ako je studentov hotel svugdje jednak ili bolji, bez indeksa.
    let gap = theirs
        .iter()
        .zip(&mine)
        .filter(|(t, m)| t.change > m.change)
        .fold(
            None,
            |best: Option<(&SeasonIndexFact, &SeasonIndexFact)>, (t, m)| match best {
                Some((bt, bm)) if bt.change - bm.change >= t.change - m.change => Some((bt, bm)),
                _ => Some((t, m)),
            },
        );
    let Some((t, m)) = gap else {
        return Some(format!("{decided}."));
    };
    let moved = match t.change.signum() {
        1 => format!("porastao s {} na {}", t.start, t.end),
        -1 => format!("pao s {} na {}", t.start, t.end),
        _ => format!("ostao na {}", t.end),
    };
    Some(format!(
        "{decided}; indeks {} mu je {moved}, a tvom hotelu s {} na {}.",
        t.name, m.start, m.end
    ))
}

/// Četiri podnaslova sezonskog mentora (uputa `mentor-sezona.v2` i rezerva).
const PARTS: [&str; 4] = [
    "Kako je prošla sezona",
    "Što je išlo dobro",
    "Što nije valjalo",
    "Što ponijeti u sljedeću igru",
];

/// Ima li tekst AI-ja sva četiri dijela (svaki podnaslov u svom retku). Radnik inače sprema rezervu, da prazan ili
/// odrezan odgovor ne ostane zauvijek zapisan kao AI-jev.
pub fn season_debrief_has_parts(text: &str) -> bool {
    PARTS.iter().all(|p| text.lines().any(|l| l.trim() == *p))
}

/// Sezonski mentor bez AI-ja (kraj-sezone-spec §7): ista četiri dijela kao uputa, iz istih činjenica.
pub fn fallback_season_debrief(input: &SeasonDebriefInput) -> String {
    let f = season_debrief_facts(input);
    let mut out = format!(
        "Kako je prošla sezona\nNapredak tvog hotela od početka sezone je {}, a to je {}. {} {}\n\n",
        f.progress, f.place, f.winner, f.fairness
    );

    let mut risen: Vec<&SeasonIndexFact> = f.indices.iter().filter(|i| i.change > 0).collect();
    risen.sort_by_key(|i| std::cmp::Reverse(i.change));
    let mut good: Vec<String> = risen
        .iter()
        .take(2)
        .map(|i| format!("Indeks {} porastao je s {} na {}.", i.name, i.start, i.end))
        .collect();
    if good.is_empty() {
        good.push("Nijedan indeks nije završio više nego na početku.".to_owned());
    }
    if let Some(b) = &f.best_round {
        good.push(format!(
            "Najbolji je bio {}: {} napretka.",
            b.round, b.change
        ));
    }
    out.push_str(&format!("Što je išlo dobro\n{}\n\n", good.join(" ")));

    let mut fallen: Vec<&SeasonIndexFact> = f.indices.iter().filter(|i| i.change < 0).collect();
    fallen.sort_by_key(|i| i.change);
    let mut bad: Vec<String> = fallen
        .iter()
        .take(2)
        .map(|i| format!("Indeks {} pao je s {} na {}.", i.name, i.start, i.end))
        .collect();
    if bad.is_empty() {
        bad.push("Nijedan indeks nije završio niže nego na početku.".to_owned());
    }
    if let Some(w) = &f.worst_round {
        bad.push(format!(
            "Najslabiji je bio {}: {} napretka.",
            w.round, w.change
        ));
    }
    if f.unanswered > 0 {
        let u = u64::from(f.unanswered);
        bad.push(format!(
            "Na {u} {} nije bilo odgovora.",
            plural(u, ["situaciju", "situacije", "situacija"])
        ));
    }
    out.push_str(&format!("Što nije valjalo\n{}\n\n", bad.join(" ")));

    // Najslabiji indeks: najveći pad, a kad ništa nije palo, najniži na kraju sezone.
    let worst = fallen.first().copied().or_else(|| {
        f.indices
            .iter()
            .fold(None, |best: Option<&SeasonIndexFact>, cur| match best {
                Some(b) if b.end <= cur.end => Some(b),
                _ => Some(cur),
            })
    });
    let mut next: Vec<String> = worst
        .map(|w| advice(w.index).to_owned())
        .into_iter()
        .collect();
    next.extend(robot_comparison(input));
    if next.is_empty() {
        next.push("Zadrži ono što je radilo i u sljedećoj igri prati svih pet indeksa.".to_owned());
    }
    out.push_str(&format!(
        "Što ponijeti u sljedeću igru\n{}\n\n",
        next.join(" ")
    ));
    out.push_str("AI objašnjenje danas nije bilo dostupno, ovo je sažetak iz brojki.");
    out
}
