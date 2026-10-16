//! Mentor govori ljudski (ADR-023): iz `DebriefInput` se grade činjenice samo s ljudskim vrijednostima
//! (imena indeksa, cijeli bodovi, euri, naslovi situacija, odluke iz plana). I AI mentor i mentor bez AI-ja
//! pišu iz njih, pa nijedna interna jedinica (stotinke, centi, baze, kodovi) ne izlazi studentu.
//! Polja su engleska, a JSON ključevi hrvatski (`serde(rename)`), da se čitaju kao rečenice.

use std::collections::BTreeMap;

use serde::Serialize;
use sim_hotel::{
    Driver, Index, LOAD_LIMIT_HUNDREDTHS, Month, index_drivers, load_penalty, retention,
};

use crate::ai_input::{DebriefInput, DepartmentDepartures};
use crate::dto::{CauseDto, LineDto, MaintenanceCode, MonthCode, TrainingCode, UnitCode, VarCode};
use crate::error::GameError;
use crate::game::Game;
use crate::mentor_v2::{
    FinanceNote, InvestmentFacts, equipment_rows, fallback_notes, finance_note, investment_facts,
    renewal_plan_text, revenue_rows,
};
use crate::own_words::RuleFacts;
use crate::round::RoundKey;

const MINUS: char = '\u{2212}';

/// Činjenice za mentora; vrijednosti su već u ljudskim jedinicama.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DebriefFacts {
    #[serde(rename = "hotel")]
    pub hotel: String,
    /// Mjesec u lokativu s prijedlogom („u srpnju").
    #[serde(rename = "mjesec")]
    /// Za tjedan: „u tjednu od 29. lipnja do 5. srpnja".
    pub month: String,
    /// `None` = bio je zadnji krug igre; za tjedan „sljedeći tjedan".
    #[serde(rename = "iduci_mjesec")]
    pub next_month: Option<String>,
    #[serde(rename = "napredak_od_pocetka")]
    pub progress: String,
    #[serde(rename = "blagajna_na_kraju_mjeseca")]
    pub cash_end: String,
    #[serde(rename = "ljestvica_indeksa")]
    pub index_scale: &'static str,
    #[serde(rename = "odluke_u_planu")]
    pub plan: PlanFacts,
    #[serde(rename = "indeksi")]
    pub indices: Vec<IndexFact>,
    #[serde(rename = "situacije")]
    pub situations: Vec<SituationFact>,
    /// Obnove kruga i stanje zgrade (samo v2).
    #[serde(rename = "ulaganja", skip_serializing_if = "Option::is_none")]
    pub investments: Option<InvestmentFacts>,
    /// Što miče indeks Financije: dobit iz poslovanja nakon ulaganja, amortizacija, ulaganja (samo v2).
    #[serde(rename = "financije_kruga", skip_serializing_if = "Option::is_none")]
    pub finance_note: Option<FinanceNote>,
}

/// Odluke iz plana mjeseca riječima, da „Zašto" može pad povezati s konkretnom odlukom.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlanFacts {
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
pub struct IndexFact {
    #[serde(skip)]
    pub index: Index,
    #[serde(rename = "naziv")]
    pub name: &'static str,
    #[serde(rename = "prije")]
    pub before: i64,
    #[serde(rename = "poslije")]
    pub after: i64,
    /// Razlika zaokruženih vrijednosti, da se slaže s ekranom.
    #[serde(rename = "promjena")]
    pub change: i64,
    #[serde(rename = "smjer")]
    pub direction: &'static str,
    #[serde(rename = "sto_je_pokrenulo")]
    pub drivers: Vec<DriverFact>,
}

/// Odakle je pokretač: varijabla iz objašnjenja mjeseca ili opterećenje osoblja (nije varijabla).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverSource {
    Var(VarCode),
    StaffLoad,
}

/// Jedan pokretač indeksa, s promjenama ovog mjeseca.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DriverFact {
    #[serde(skip)]
    pub source: DriverSource,
    #[serde(rename = "sto")]
    pub what: &'static str,
    #[serde(rename = "znacenje")]
    pub meaning: &'static str,
    #[serde(rename = "promjene")]
    pub changes: Vec<ChangeFact>,
}

/// Je li redak gurnuo svoj indeks gore ili dolje; razina bez smjera (prihod, troškovi, osnovna
/// potražnja, broj ljudi) je `Neutral`. Isto pravilo kao `harm`: dolje je točno kad je `harm > 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Up,
    Down,
    Neutral,
}

impl Effect {
    fn of(push: i64) -> Self {
        match push.signum() {
            1 => Effect::Up,
            -1 => Effect::Down,
            _ => Effect::Neutral,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChangeFact {
    #[serde(rename = "uzrok")]
    pub cause: String,
    #[serde(rename = "iznos")]
    pub amount: String,
    /// Koliko je uzrok gurnuo indeks dolje; uspoređuje se samo unutar istog indeksa. 0 = nije šteta.
    #[serde(skip)]
    pub harm: i64,
    /// Smjer retka za ekran („Zašto se indeksi pomaknuli"); iz istog guranja kao `harm`.
    #[serde(skip)]
    pub effect: Effect,
    /// Uzrok u genitivu („zbog nižih plaća") za mentora bez AI-ja.
    #[serde(skip)]
    pub because: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SituationFact {
    #[serde(rename = "naslov")]
    pub title: String,
    #[serde(rename = "ocjena")]
    pub grade: String,
    #[serde(rename = "ocjena_rijecima")]
    pub grade_words: &'static str,
    #[serde(rename = "slobodan_odgovor")]
    pub free_text: bool,
    /// Što je student odabrao: oznaka ponuđenog odgovora ili da je napisao vlastiti.
    #[serde(rename = "odabrani_odgovor", skip_serializing_if = "Option::is_none")]
    pub chosen: Option<String>,
    #[serde(rename = "bez_odgovora", skip_serializing_if = "is_false")]
    pub unanswered: bool,
    /// Student je situaciju prepustio voditelju odjela (Leon P4); nije „bez odgovora".
    #[serde(rename = "prepustena_voditelju", skip_serializing_if = "is_false")]
    pub delegated: bool,
    #[serde(rename = "obrazlozenje", skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// Činjenice o neutralnoj, ograničenoj i zalijepljenoj ocjeni (`own_words`).
    #[serde(flatten)]
    pub rule: RuleFacts,
    #[serde(skip)]
    pub quality: u8,
    /// Oznaka ponuđenog odgovora (bez slobodnog), za mentora bez AI-ja.
    #[serde(skip)]
    pub chosen_label: Option<String>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Tekst za mjesec ili za tjedan: tjedni krug govori o tjednu (ne o „mjesecu"), a v1 i mjesečna v2 igra dobivaju
/// točno stari tekst.
fn per<'a>(weekly: bool, month: &'a str, week: &'a str) -> &'a str {
    if weekly { week } else { month }
}

fn is_weekly(input: &DebriefInput) -> bool {
    matches!(input.round, RoundKey::Week(_))
}

// ---------------------------------------------------------------- nazivi

pub(crate) fn index_name(i: Index) -> &'static str {
    match i {
        Index::Guests => "Zadovoljstvo gostiju",
        Index::Reviews => "Recenzije",
        Index::Staff => "Zaposlenici",
        Index::Operations => "Operativa",
        Index::Finance => "Financije",
    }
}

pub(crate) fn month_genitive(m: MonthCode) -> &'static str {
    match m {
        MonthCode::Jun => "lipnja",
        MonthCode::Jul => "srpnja",
        MonthCode::Aug => "kolovoza",
        MonthCode::Sep => "rujna",
    }
}

/// Gdje u vremenu: mjesec „u lipnju", tjedan „u tjednu od 29. lipnja do 5. srpnja".
fn place_text(input: &DebriefInput) -> String {
    match input.round {
        RoundKey::Month(_) => round_locative(input.round).to_owned(),
        RoundKey::Week(_) => format!(
            "u tjednu od {}. {} do {}. {}",
            input.first_date.0,
            month_genitive(input.first_date.1),
            input.last_date.0,
            month_genitive(input.last_date.1)
        ),
    }
}

/// Idući krug riječima: mjesec u lokativu, tjedan „sljedeći tjedan".
fn next_text(next: RoundKey) -> String {
    match next {
        RoundKey::Month(_) => round_locative(next).to_owned(),
        RoundKey::Week(_) => "sljedeći tjedan".to_owned(),
    }
}

/// Mjesec igre u lokativu („u lipnju"); tjedan se slaže u `place_text`.
pub(crate) fn round_locative(r: RoundKey) -> &'static str {
    match r {
        RoundKey::Month(Month::Jun) => "u lipnju",
        RoundKey::Month(Month::Jul) => "u srpnju",
        RoundKey::Month(Month::Aug) => "u kolovozu",
        RoundKey::Month(Month::Sep) => "u rujnu",
        RoundKey::Week(_) => "u ovom tjednu",
    }
}

/// Nazivi varijabli za studenta (ekran „Zašto se indeksi pomaknuli" ih prikazuje kakvi jesu).
fn var_name(v: VarCode) -> &'static str {
    match v {
        VarCode::Sold => "Prodane noći",
        VarCode::Revenue => "Prihod",
        VarCode::Costs => "Troškovi",
        VarCode::Cash => "Blagajna",
        VarCode::Staff => "Zaposleni",
        VarCode::Morale => "Moral",
        VarCode::Experience => "Iskustvo",
        VarCode::Equipment => "Oprema",
        VarCode::Satisfaction => "Zadovoljstvo gostiju",
        VarCode::Rating => "Ocjena na portalima",
    }
}

const LOAD_NAME: &str = "Opterećenje osoblja";
const LOAD_MEANING: &str = "Koliko posla pada na jednog zaposlenog: prodana noćenja po zaposlenom na dan. Do 1,8 noćenja je u redu; iznad toga osoblje je preopterećeno, pa trpe usluga, oprema i moral.";

/// Kako čitati iznose uz varijablu (osobito moral, gdje su uzroci pritisci, a ne cijela promjena).
fn var_meaning(v: VarCode, weekly: bool) -> &'static str {
    match v {
        VarCode::Sold => {
            "Koliko je noćenja hotel prodao. Postoci kažu koliko je zbog nečega potražnja bila veća ili manja."
        }
        VarCode::Revenue => per(
            weekly,
            "Novac koji je hotel dobio od gostiju ovaj mjesec, po izvoru.",
            "Novac koji je hotel dobio od gostiju ovaj tjedan, po izvoru.",
        ),
        VarCode::Costs => per(
            weekly,
            "Što je hotel platio ovaj mjesec, po vrsti troška.",
            "Što je hotel platio ovaj tjedan, po vrsti troška.",
        ),
        VarCode::Cash => "Kako se mijenjao novac na računu hotela.",
        VarCode::Staff => per(
            weekly,
            "Tko je došao i tko je otišao; oni koji su dali otkaz zamijenjeni su novima, pa broj ljudi ostaje isti. Indeks Zaposlenici ne prati broj otišlih nego udio osoblja koje u mjesecu da otkaz: veći udio nego prije ruši indeks, a manji ga diže; od 8 % naviše taj je dio indeksa već na najnižem i više se ne miče. Udio raste kad moral padne ispod 60 bodova.",
            "Tko je došao i tko je otišao; oni koji su dali otkaz zamijenjeni su novima, pa broj ljudi ostaje isti. Indeks Zaposlenici ne prati broj otišlih nego udio osoblja koje u tjednu da otkaz: veći udio nego prije ruši indeks, a manji ga diže; od 8 % naviše taj je dio indeksa već na najnižem i više se ne miče. Udio raste kad moral padne ispod 60 bodova.",
        ),
        VarCode::Morale => per(
            weekly,
            "Raspoloženje i volja tima, na ljestvici do 100 bodova. Moral se svaki mjesec pomakne samo dio puta prema razini koju guraju plaće, opterećenje, obuka i situacije; zato je ukupna promjena manja od zbroja tih pritisaka.",
            "Raspoloženje i volja tima, na ljestvici do 100 bodova. Moral se svaki tjedan pomakne samo dio puta prema razini koju guraju plaće, opterećenje, obuka i situacije; zato je ukupna promjena manja od zbroja tih pritisaka.",
        ),
        VarCode::Experience => {
            "Koliko tim zna i može, na ljestvici do 100 bodova. Raste radom i obukom, pada kad iskusni ljudi odu."
        }
        VarCode::Equipment => per(
            weekly,
            "Stanje soba i opreme, na ljestvici do 100 bodova. Svaki mjesec se troši; održavanje i obnova je vraćaju.",
            "Stanje soba i opreme, na ljestvici do 100 bodova. Svaki tjedan se troši; održavanje i obnova je vraćaju.",
        ),
        VarCode::Satisfaction => "Koliko su gosti zadovoljni boravkom, na ljestvici do 100 bodova.",
        VarCode::Rating => {
            "Prosječna ocjena hotela na portalima za rezervacije, na ljestvici do deset."
        }
    }
}

/// Posljedica pada pokretača, za rečenicu „zašto" mentora bez AI-ja.
fn consequence(s: DriverSource) -> &'static str {
    match s {
        DriverSource::StaffLoad => "pa osoblje ne stiže sve napraviti kako treba",
        DriverSource::Var(v) => match v {
            VarCode::Morale => "pa je tim slabije raspoložen i lakše odlazi",
            VarCode::Experience => "pa tim zna i može manje nego prije",
            VarCode::Equipment => "pa su sobe i oprema u lošijem stanju",
            VarCode::Costs => "pa je hotel potrošio više",
            VarCode::Cash => "pa je na računu ostalo manje novca",
            VarCode::Sold => "pa je prodano manje noćenja",
            // Kod gostiju i ocjene posljedica je sam indeks; mehanizam već stoji u uzroku.
            VarCode::Satisfaction | VarCode::Rating | VarCode::Staff | VarCode::Revenue => "",
        },
    }
}

/// Uzrok u nominativu; „osnova" dobiva značenje po varijabli.
fn cause_label(v: VarCode, c: &CauseDto, titles: &BTreeMap<&str, &str>, weekly: bool) -> String {
    let s = match (v, c) {
        (_, CauseDto::Situation(code)) => {
            return format!("situacija „{}“", title(titles, code, weekly));
        }
        (VarCode::Sold, CauseDto::Baseline) => per(
            weekly,
            "osnovna potražnja za ovaj mjesec",
            "osnovna potražnja za ovaj tjedan",
        ),
        (VarCode::Revenue, CauseDto::Baseline) => "noćenja pojedinačnih gostiju",
        (VarCode::Costs, CauseDto::Baseline) => "troškovi po prodanom noćenju",
        (VarCode::Cash, CauseDto::Baseline) => "zarada iz poslovanja (prihod minus troškovi)",
        (VarCode::Morale, CauseDto::Baseline) => per(
            weekly,
            "ukupna promjena morala u mjesecu",
            "ukupna promjena morala u tjednu",
        ),
        (VarCode::Experience, CauseDto::Baseline) => "iskustvo stečeno radom",
        (VarCode::Equipment, CauseDto::Baseline) => "redovito trošenje opreme",
        (VarCode::Satisfaction, CauseDto::Baseline) => "usluga, oprema i cijena zajedno",
        (VarCode::Rating, CauseDto::Baseline) => "ocjene gostiju prema njihovu zadovoljstvu",
        (VarCode::Staff, CauseDto::Turnover) => "odlasci zaposlenih (zamijenjeni novima)",
        (VarCode::Experience, CauseDto::Turnover) => "novi ljudi umjesto iskusnih koji su otišli",
        (VarCode::Costs, CauseDto::Hires) => "zapošljavanje i zamjena onih koji su otišli",
        (VarCode::Sold, CauseDto::Reputation) => "ugled (ocjena na portalima)",
        (VarCode::Cash, CauseDto::Interest) => "kamate na minus na računu",
        (_, CauseDto::Baseline) => "osnova",
        (_, CauseDto::Price) => "cijena",
        (_, CauseDto::Reputation) => "ugled",
        (_, CauseDto::Marketing) => "marketing",
        (_, CauseDto::Groups) => "grupe",
        (_, CauseDto::Capacity) => "nema slobodnih soba",
        (_, CauseDto::OtherRevenue) => "ostali prihodi",
        (_, CauseDto::Wage) => "plaće",
        (_, CauseDto::Load) => "opterećenje osoblja",
        (_, CauseDto::Training) => "obuka",
        (_, CauseDto::Hires) => "zapošljavanje",
        (_, CauseDto::Release) => "otpuštanje",
        (_, CauseDto::Turnover) => "odlasci zaposlenih",
        (_, CauseDto::Maintenance) => "održavanje",
        (_, CauseDto::Renovation) => "obnova",
        (_, CauseDto::ReplyReviews) => "odgovori na recenzije",
        // Samo v2: fiksne naknade ispod GOP-a (ime kao u Računu dobiti), ne stalni dio službi iz retka troškova.
        (VarCode::Cash, CauseDto::FixedCosts) => {
            "fiksni troškovi zgrade (najam, osiguranje, porez)"
        }
        (_, CauseDto::FixedCosts) => "fiksni troškovi",
        (_, CauseDto::Interest) => "kamate",
    };
    s.to_owned()
}

/// Uzrok štete u genitivu, za „pao je ponajviše zbog …"; svaki par ima vlastitu frazu.
fn cause_because(v: VarCode, c: &CauseDto, titles: &BTreeMap<&str, &str>, weekly: bool) -> String {
    let s = match (v, c) {
        (_, CauseDto::Situation(code)) => {
            return format!("rješenja situacije „{}“", title(titles, code, weekly));
        }
        (VarCode::Morale, CauseDto::Wage) => "nižih plaća",
        (_, CauseDto::Wage) => "plaća",
        (_, CauseDto::Load) => "prevelikog opterećenja osoblja",
        (VarCode::Morale, CauseDto::Release) => "otpuštanja dijela osoblja",
        (_, CauseDto::Release) => "otpuštanja",
        (VarCode::Experience, CauseDto::Turnover) => "odlaska iskusnih ljudi",
        (_, CauseDto::Turnover) => "odlazaka zaposlenih",
        (VarCode::Equipment, CauseDto::Baseline) => {
            "trošenja opreme koje održavanje nije nadoknadilo"
        }
        (VarCode::Satisfaction, CauseDto::Baseline) => {
            "slabije usluge, opreme ili cijene: gost to osjeti kad je tim umoran ili neiskusan, sobe istrošene ili cijena previsoka za ono što dobije"
        }
        (VarCode::Rating, CauseDto::Baseline) => {
            "nižeg zadovoljstva gostiju: nezadovoljni gosti pišu slabije recenzije"
        }
        (VarCode::Cash, CauseDto::Baseline) => "gubitka u poslovanju (troškovi veći od prihoda)",
        (VarCode::Morale, CauseDto::Baseline) => "pada morala",
        (VarCode::Sold, CauseDto::Baseline) => per(
            weekly,
            "slabije potražnje u ovom mjesecu",
            "slabije potražnje u ovom tjednu",
        ),
        (_, CauseDto::Baseline) => per(weekly, "redovnog tijeka mjeseca", "redovnog tijeka tjedna"),
        (VarCode::Equipment, CauseDto::Maintenance) => "slabijeg održavanja",
        (_, CauseDto::Maintenance) => "održavanja",
        (VarCode::Sold, CauseDto::Price) => "previsoke cijene",
        (_, CauseDto::Price) => "cijene",
        (VarCode::Sold, CauseDto::Reputation) => "slabijeg ugleda na portalima",
        (_, CauseDto::Reputation) => "ugleda",
        (VarCode::Sold, CauseDto::Marketing) => "premalo marketinga",
        (_, CauseDto::Marketing) => "marketinga",
        (_, CauseDto::Groups) => "grupa",
        (_, CauseDto::Capacity) => "nedostatka slobodnih soba",
        (_, CauseDto::OtherRevenue) => "ostalih prihoda",
        (_, CauseDto::Training) => "obuke",
        (_, CauseDto::Hires) => "zapošljavanja",
        (VarCode::Cash, CauseDto::Renovation) => "ulaganja u obnovu",
        (_, CauseDto::Renovation) => "obnove",
        (_, CauseDto::ReplyReviews) => "odgovora na recenzije",
        (VarCode::Cash, CauseDto::FixedCosts) => {
            "fiksnih troškova zgrade (najam, osiguranje, porez)"
        }
        (_, CauseDto::FixedCosts) => "fiksnih troškova",
        (VarCode::Cash, CauseDto::Interest) => "kamata na minus na računu",
        (_, CauseDto::Interest) => "kamata",
    };
    s.to_owned()
}

fn title<'a>(titles: &BTreeMap<&str, &'a str>, code: &str, weekly: bool) -> &'a str {
    titles.get(code).copied().unwrap_or_else(|| {
        per(
            weekly,
            "situacija iz ovog mjeseca",
            "situacija iz ovog tjedna",
        )
    })
}

// ---------------------------------------------------------------- brojke

/// Cjelobrojno dijeljenje zaokruženo pola od nule (4850/100 → 49, 4849/100 → 48, −453/100 → −5).
pub(crate) fn round_div(v: i64, d: i64) -> i64 {
    let half = d / 2;
    if v >= 0 {
        (v + half) / d
    } else {
        -((-v + half) / d)
    }
}

/// Točka za tisuće: 1234567 → „1.234.567".
pub(crate) fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(ch);
    }
    out
}

/// Stotinke s decimalnim zarezom bez suvišnih nula: 230 → „2,3", 185 → „1,85", 200 → „2".
pub(crate) fn decimal_hundredths(n: u64) -> String {
    match n % 100 {
        0 => format!("{}", n / 100),
        r if r.is_multiple_of(10) => format!("{},{}", n / 100, r / 10),
        r => format!("{},{r:02}", n / 100),
    }
}

/// „+" ili „−" ispred (nula bez predznaka) kad je `signed`; inače samo iznos.
pub(crate) fn with_sign(n: i64, body: String, signed: bool) -> String {
    match (signed, n.signum()) {
        (true, 1) => format!("+{body}"),
        (_, -1) => format!("{MINUS}{body}"),
        _ => body,
    }
}

/// Indeks hrvatskog oblika uz broj: 0 = jednina (1, 21), 1 = 2–4 (osim 12–14), 2 = ostalo.
fn plural_class(n: u64) -> usize {
    let (d, h) = (n % 10, n % 100);
    if d == 1 && h != 11 {
        0
    } else if (2..=4).contains(&d) && !(12..=14).contains(&h) {
        1
    } else {
        2
    }
}

/// Hrvatski oblik uz broj: 1 bod, 2 boda, 5 bodova (21 bod, 12 bodova).
pub(crate) fn plural(n: u64, forms: [&'static str; 3]) -> &'static str {
    forms[plural_class(n)]
}

pub(crate) fn euros(cents: i64, signed: bool) -> String {
    let e = round_div(cents, 100);
    with_sign(e, format!("{} €", thousands(e.unsigned_abs())), signed)
}

pub(crate) fn points(n: i64) -> String {
    let a = n.unsigned_abs();
    with_sign(
        n,
        format!("{} {}", thousands(a), plural(a, ["bod", "boda", "bodova"])),
        true,
    )
}

/// Ljudi riječima, bez predznaka: „zaposleno 6 osoba", „otišle 3 osobe", „otpuštena 1 osoba".
fn people_text(c: &CauseDto, amount: i64) -> String {
    let n = amount.unsigned_abs();
    let k = plural_class(n);
    let verb = match c {
        CauseDto::Hires => ["zaposlena", "zaposlene", "zaposleno"][k],
        CauseDto::Release => ["otpuštena", "otpuštene", "otpušteno"][k],
        CauseDto::Turnover => ["otišla", "otišle", "otišlo"][k],
        _ => {
            return with_sign(
                amount,
                format!("{n} {}", ["osoba", "osobe", "osoba"][k]),
                true,
            );
        }
    };
    format!("{verb} {n} {}", ["osoba", "osobe", "osoba"][k])
}

/// Najviše toliko odjela se imenuje uz odlaske; ostali idu u „i još N".
const DEPARTURE_NAMED_MAX: usize = 3;

/// Odlasci iz hotelskog retka s odjelima: „otišle 3 osobe (Smještaj 2, Restoran 1)", a uz više od tri odjela
/// „… (Smještaj 3, Restoran 2, Uprava 1 i još 2)". Bez raščlambe, ili kad se ne slaže s brojem u retku (krug
/// zapisan prije raščlambe), tekst ostaje kakav je bio.
fn with_departments(
    text: String,
    hotel_total: i64,
    by_department: &[DepartmentDepartures],
) -> String {
    let sum: i64 = by_department.iter().map(|d| i64::from(d.count)).sum();
    if by_department.is_empty() || sum != hotel_total {
        return text;
    }
    let named: Vec<String> = by_department
        .iter()
        .take(DEPARTURE_NAMED_MAX)
        .map(|d| format!("{} {}", d.name, d.count))
        .collect();
    let rest = by_department.len().saturating_sub(DEPARTURE_NAMED_MAX);
    let list = if rest == 0 {
        named.join(", ")
    } else {
        format!("{} i još {rest}", named.join(", "))
    };
    // Jedan odjel: brojka je već u rečenici („otišla 1 osoba (Restoran)").
    match by_department {
        [only] => format!("{text} ({})", only.name),
        _ => format!("{text} ({list})"),
    }
}

/// Iznos retka u ljudskoj jedinici; `None` kad je (zaokruženo) nula pa se redak izostavlja.
fn amount_text(l: &LineDto, signed: bool) -> Option<String> {
    let a = l.amount;
    if a == 0 {
        return None;
    }
    let text = match l.unit {
        UnitCode::Cents => {
            if round_div(a, 100) == 0 {
                return None;
            }
            euros(a, signed)
        }
        UnitCode::RoomNights => {
            let n = a.unsigned_abs();
            with_sign(
                a,
                format!(
                    "{} {}",
                    thousands(n),
                    plural(n, ["noćenje", "noćenja", "noćenja"])
                ),
                signed,
            )
        }
        UnitCode::People => people_text(&l.cause, a),
        // Stotinke boda na ljestvici 0–100 → cijeli bodovi.
        UnitCode::Points => {
            let p = round_div(a, 100);
            if p == 0 {
                return None;
            }
            points(p)
        }
        // Ocjena portala u stotinkama na ljestvici do 10 (840 = 8,40) → dvije decimale.
        UnitCode::RatingHundredths => {
            let n = a.unsigned_abs();
            with_sign(a, format!("{},{:02} ocjene", n / 100, n % 100), true)
        }
        // Faktor potražnje u bazama (10 000 = 100 %): −200 → potražnja manja za 2 %.
        UnitCode::Bp => {
            let tenths = round_div(a, 10);
            if tenths == 0 {
                return None;
            }
            let n = tenths.unsigned_abs();
            let pct = if n.is_multiple_of(10) {
                format!("{}", n / 10)
            } else {
                format!("{},{}", n / 10, n % 10)
            };
            with_sign(tenths, format!("{pct} % potražnje"), true)
        }
    };
    Some(text)
}

/// Iznosi su razine (prihod, troškovi, osnovna potražnja), ne promjene: bez predznaka.
fn shows_sign(v: VarCode, c: &CauseDto) -> bool {
    !matches!(
        (v, c),
        (VarCode::Revenue | VarCode::Costs, _) | (VarCode::Sold, CauseDto::Baseline)
    )
}

/// Koliko je redak gurnuo svoj indeks: + gore, − dolje, 0 razina bez smjera. Gruba težina za usporedbu
/// unutar istog indeksa (mentor bez AI-ja bira najveću štetu), u omjeru težina iz `raw_indices`:
/// udio člana u indeksu (u postocima) × promjena člana u stotinkama boda, pa je guranje / 100 promjena
/// sirovog indeksa u stotinkama. Zaposlenici: moral 60, zadržavanje 40 (`turnover_changes`);
/// Operativa: oprema 45, iskustvo 30, opterećenje 25 (`load_driver`).
/// `room_revenue` (prihod od noćenja) pretvara postotak potražnje u prihod; `maintained` je koliko su
/// održavanje i obnova vratili opremu, pa trošenje gura dolje samo u dijelu koji nisu pokrili.
/// `morale_alone`: ukupna promjena morala je jedini prikazani redak morala.
fn push(l: &LineDto, room_revenue: i64, maintained: i64, morale_alone: bool) -> i64 {
    let a = l.amount;
    match (l.var, &l.cause) {
        // Trošak situacije ruši Financije; ostali troškovi i prihod su razine.
        (VarCode::Costs, CauseDto::Situation(_)) => a.saturating_neg(),
        // Broj ljudi je razina; Zaposlenike miče stopa odlazaka (`turnover_changes`).
        (VarCode::Revenue | VarCode::Costs | VarCode::Staff, _) => 0,
        // Ukupna promjena morala je zbroj pritisaka, ne uzrok (inače bi se brojala dvaput). Kad je jedini
        // redak morala, ona je stvarni uzrok i nosi smjer.
        (VarCode::Morale, CauseDto::Baseline) if !morale_alone => 0,
        (VarCode::Morale, _) => a.saturating_mul(60),
        // Najmanje 1, da se smjer ne izgubi zaokruživanjem kad je prihod od noćenja malen.
        (VarCode::Sold, CauseDto::Price | CauseDto::Reputation | CauseDto::Marketing) => {
            a.signum() * (room_revenue.max(1).saturating_mul(a.saturating_abs()) / 10_000).max(1)
        }
        (VarCode::Sold, _) => 0,
        (VarCode::Equipment, CauseDto::Baseline) if a < 0 => {
            -(a.saturating_neg() - maintained).max(0) * 45
        }
        (VarCode::Equipment, _) => a.saturating_mul(45),
        (VarCode::Experience, _) => a.saturating_mul(30),
        _ => a,
    }
}

/// Šteta = koliko je redak gurnuo indeks dolje (0 kad nije).
fn harm_of(push: i64) -> i64 {
    push.saturating_neg().max(0)
}

/// Isti uzrok iste varijable u jednom retku (npr. trošak situacije i njezina trajna povišica).
fn merged(lines: &[LineDto]) -> Vec<LineDto> {
    let mut out: Vec<LineDto> = Vec::new();
    for l in lines {
        match out
            .iter_mut()
            .find(|o| o.var == l.var && o.cause == l.cause && o.unit == l.unit)
        {
            Some(o) => o.amount = o.amount.saturating_add(l.amount),
            None => out.push(l.clone()),
        }
    }
    out
}

/// S čim se uspoređuje „prije" mjeseca: prošli mjesec, a u prvom mjesecu igre (`first`, iz
/// `DebriefInput::prev_month`) početno stanje hotela. Jedno mjesto za sve usporedbe u činjenicama.
fn earlier(first: bool, weekly: bool) -> &'static str {
    if first {
        "na početku igre"
    } else {
        per(weekly, "prošli mjesec", "prošli tjedan")
    }
}

/// Ovaj mjesec prema ranijem: „više nego prošli mjesec", „manje nego na početku igre", „isto kao …".
fn compared(now_vs_before: std::cmp::Ordering, first: bool, weekly: bool) -> String {
    let than = earlier(first, weekly);
    match now_vs_before {
        std::cmp::Ordering::Greater => format!("više nego {than}"),
        std::cmp::Ordering::Less => format!("manje nego {than}"),
        std::cmp::Ordering::Equal => format!("isto kao {than}"),
    }
}

/// Opterećenje osoblja kao pokretač Operative; izostaje kad je i prije i poslije bilo u redu.
/// Šteta je porast kazne opterećenja s težinom 25 (njezin udio u Operativi).
fn load_driver(before: u16, after: u16, first: bool, weekly: bool) -> Option<DriverFact> {
    if before <= LOAD_LIMIT_HUNDREDTHS && after <= LOAD_LIMIT_HUNDREDTHS {
        return None;
    }
    let per_day = |l: u16| {
        format!(
            "{} noćenja po zaposlenom na dan",
            decimal_hundredths(u64::from(l))
        )
    };
    let state = if after > LOAD_LIMIT_HUNDREDTHS {
        "osoblje je preopterećeno"
    } else {
        "u redu"
    };
    let penalty = |l: u16| i64::from(load_penalty(l).unwrap_or_default());
    let because = "preopterećenog osoblja (premalo ljudi za broj gostiju)".to_owned();
    // Veća kazna nego prošli mjesec gura Operativu dolje, manja gore.
    let pushed = (penalty(before) - penalty(after)) * 25;
    Some(DriverFact {
        source: DriverSource::StaffLoad,
        what: LOAD_NAME,
        meaning: LOAD_MEANING,
        changes: vec![
            ChangeFact {
                cause: per(
                    weekly,
                    "opterećenje u ovom mjesecu",
                    "opterećenje u ovom tjednu",
                )
                .to_owned(),
                amount: format!("{}, {state}", per_day(after)),
                harm: harm_of(pushed),
                effect: Effect::of(pushed),
                because: because.clone(),
            },
            // Prošli mjesec (ili početak igre) je samo usporedba, ne uzrok.
            ChangeFact {
                cause: format!("opterećenje {}", earlier(first, weekly)),
                amount: per_day(before),
                harm: 0,
                effect: Effect::Neutral,
                because,
            },
        ],
    })
}

/// Stopa odlazaka ovog mjeseca prema prošlom (za prvi mjesec: početno stanje hotela), prvi retci
/// pokretača „Zaposleni". Z = 60 % moral + 40 % zadržavanje, a zadržavanje = 10 000 − 12,5 · stopa u
/// bazama (`sim_hotel::retention`). Indeks zato ne miče broj otišlih (razina, ovisi i o broju ljudi) nego
/// promjena stope: guranje = 40 · Δzadržavanja = −500 po bazi rasta stope, isto mjerilo kao moral ×60.
/// Kad je zadržavanje već na nuli (stopa 8 % i više) i prije i poslije, indeks se zbog odlazaka ne miče:
/// siva, i iznos to kaže. Stopa slijedi moral na kraju mjeseca (`sim_hotel::turnover_bp`): 3 % do 60
/// bodova morala, iznad toga raste; zato rečenica „zašto" kaže i mehanizam.
/// `first`: prvi mjesec igre nema prošlog, usporedba je s početkom igre („na početku igre").
fn turnover_changes(before: u16, after: u16, first: bool, weekly: bool) -> [ChangeFact; 2] {
    let share = |bp: u16| format!("{} %", decimal_hundredths(u64::from(bp)));
    let than = earlier(first, weekly);
    // Na početku igre nitko još nije otišao: to je udio koji hotel tada ima, ne prošli događaj.
    let prev_cause = if first {
        format!("udio osoblja koje daje otkaz {than}")
    } else {
        format!("udio osoblja koje je dalo otkaz {than}")
    };
    let vs = compared(after.cmp(&before), first, weekly);
    let kept = |bp: u16| i64::from(retention(bp).unwrap_or_default());
    let pushed = (kept(after) - kept(before)) * 40;
    // Udio se promijenio, a indeks nije: oba mjeseca su na dnu zadržavanja.
    let floor = if after != before && pushed == 0 {
        ", ali od 8 % naviše taj je dio indeksa već na najnižem, pa se ne miče"
    } else {
        ""
    };
    let because = format!(
        "većeg udjela osoblja koje je dalo otkaz nego {than}: kad moral padne ispod 60 bodova, više ljudi daje otkaz"
    );
    [
        ChangeFact {
            cause: per(
                weekly,
                "udio osoblja koje je dalo otkaz ovaj mjesec",
                "udio osoblja koje je dalo otkaz ovaj tjedan",
            )
            .to_owned(),
            amount: format!("{}, {vs}{floor}", share(after)),
            harm: harm_of(pushed),
            effect: Effect::of(pushed),
            because: because.clone(),
        },
        // Prošli mjesec (ili početak igre) je samo usporedba, ne uzrok.
        ChangeFact {
            cause: prev_cause,
            amount: share(before),
            harm: 0,
            effect: Effect::Neutral,
            because,
        },
    ]
}

// ---------------------------------------------------------------- činjenice

fn grade_words(q: u8) -> &'static str {
    match q {
        4.. => "dobro",
        3 => "srednje",
        _ => "loše",
    }
}

pub(crate) fn direction(change: i64) -> &'static str {
    match change.signum() {
        1 => "porastao",
        -1 => "pao",
        _ => "isti",
    }
}

/// Plaće riječima uz podatak tko ih je postavio: mentor ne smije studentu pripisati stanje koje je preuzeo s profilom
/// hotela („takve su od početka") niti promjenu koju je napravio netko drugi.
fn wage_note(input: &DebriefInput, before_round: &str) -> String {
    let now = input.plan.wage_pct;
    let level = match now.cmp(&100) {
        std::cmp::Ordering::Less => Some("niže od uobičajenih"),
        std::cmp::Ordering::Greater => Some("više od uobičajenih"),
        std::cmp::Ordering::Equal => None,
    };
    let (was, start) = (input.wage_before_pct, input.wage_start_pct);
    let history = if now != was {
        let verb = if now < was { "spustio" } else { "podigao" };
        Some(format!("student ih je {verb} sa {was} % na {now} %"))
    } else if now == start && input.wage_ever_changed {
        // Student ih je mijenjao pa vratio: ne tvrdi se ni da ih nije dirao ni da ih je on postavio.
        Some("kao na početku igre".to_owned())
    } else if now == start {
        // Nepromijenjene plaće se pripisuju profilu samo kad odstupaju od uobičajenih.
        level.map(|_| "takve su od početka, student ih nije mijenjao".to_owned())
    } else {
        Some(format!(
            "student ih je ranije promijenio sa {start} % na {now} %, a {before_round} ih nije mijenjao"
        ))
    };
    match (level, history) {
        (Some(l), Some(h)) => format!(" ({l}; {h})"),
        (Some(l), None) => format!(" ({l})"),
        (None, Some(h)) => format!(" ({h})"),
        (None, None) => String::new(),
    }
}

/// Je li odluka ista kao u prošlom krugu; `None` u prvom krugu (nema prošle odluke, samo početno stanje).
fn unchanged_note(same: Option<bool>, before_round: &str) -> String {
    match same {
        Some(true) => format!(", isto kao {before_round}"),
        Some(false) => format!(", promijenjeno u odnosu na {before_round}"),
        None => String::new(),
    }
}

fn plan_facts(input: &DebriefInput) -> PlanFacts {
    let weekly = is_weekly(input);
    let p = &input.plan;
    let reference = input.price_ref_cents.max(1);
    let off = round_div((p.price_cents - reference) * 100, reference);
    let usual = per(weekly, "ovaj mjesec", "ovaj tjedan");
    let rel = match off.signum() {
        0 => format!("kao uobičajena cijena za {usual}"),
        1 => format!("{off} % iznad uobičajene cijene za {usual}"),
        _ => format!(
            "{} % ispod uobičajene cijene za {usual}",
            off.unsigned_abs()
        ),
    };
    let n = u64::from(p.seasonal_target);
    let seasonal = if n == 0 {
        "bez sezonskih radnika".to_owned()
    } else {
        format!(
            "{n} {}",
            plural(
                n,
                ["sezonski radnik", "sezonska radnika", "sezonskih radnika"]
            )
        )
    };
    let vs_last = compared(
        p.seasonal_target.cmp(&input.seasonal_before),
        input.prev_round.is_none(),
        weekly,
    );
    let before = per(weekly, "prošli mjesec", "prošli tjedan");
    let wage_note = wage_note(input, before);
    let prev = input.plan_before.as_ref();
    let training_note = unchanged_note(prev.map(|b| b.training == p.training), before);
    let maintenance_note = unchanged_note(prev.map(|b| b.maintenance == p.maintenance), before);
    PlanFacts {
        room_price: format!("{} po noći, {rel}", euros(p.price_cents, false)),
        marketing: if p.marketing_cents == 0 {
            "bez marketinga".to_owned()
        } else {
            euros(p.marketing_cents, false)
        },
        seasonal: format!("{seasonal}, {vs_last}"),
        wages: format!("{} % uobičajene plaće{wage_note}", p.wage_pct),
        training: match p.training {
            TrainingCode::None => "bez obuke",
            TrainingCode::Basic => per(
                weekly,
                "osnovna obuka (učinak stiže idući mjesec)",
                "osnovna obuka (učinak stiže idući tjedan)",
            ),
            TrainingCode::Full => per(
                weekly,
                "puna obuka (učinak stiže idući mjesec)",
                "puna obuka (učinak stiže idući tjedan)",
            ),
        }
        .to_owned()
            + &training_note,
        maintenance: match p.maintenance {
            MaintenanceCode::Deferred => per(
                weekly,
                "odgođeno (oprema se ovaj mjesec nije popravljala)",
                "odgođeno (oprema se ovaj tjedan nije popravljala)",
            ),
            MaintenanceCode::Regular => "redovno",
            MaintenanceCode::Intensive => "pojačano",
        }
        .to_owned()
            + &maintenance_note,
        renovation: renewal_plan_text(input).unwrap_or_else(|| {
            if p.renovation {
                per(
                    weekly,
                    "pokrenuta: plaća se sada, a sobe su bolje od idućeg mjeseca",
                    "pokrenuta: plaća se sada, a sobe su bolje od idućeg tjedna",
                )
            } else {
                "ne"
            }
            .to_owned()
        }),
        reply_reviews: if p.reply_reviews { "da" } else { "ne" }.to_owned(),
    }
}

pub fn debrief_facts(input: &DebriefInput) -> DebriefFacts {
    let titles: BTreeMap<&str, &str> = input
        .situations
        .iter()
        .map(|s| (s.code.as_str(), s.title.as_str()))
        .collect();
    let lines = merged(&input.lines);
    let room_revenue: i64 = lines
        .iter()
        .filter(|l| l.var == VarCode::Revenue && l.cause == CauseDto::Baseline)
        .map(|l| l.amount)
        .sum();
    let maintained: i64 = lines
        .iter()
        .filter(|l| {
            l.var == VarCode::Equipment
                && matches!(l.cause, CauseDto::Maintenance | CauseDto::Renovation)
        })
        .map(|l| l.amount.max(0))
        .sum();
    // Prikazani retci morala (oni koji zaokruženo nisu nula); sama ukupna promjena je tada uzrok.
    let morale_shown: Vec<&LineDto> = lines
        .iter()
        .filter(|l| l.var == VarCode::Morale && amount_text(l, true).is_some())
        .collect();
    let morale_alone = matches!(morale_shown.as_slice(), [l] if l.cause == CauseDto::Baseline);
    // Prvi mjesec kaže igra (`prev_month`), ne kalendar: prije njega je početno stanje hotela.
    let first = input.prev_round.is_none();
    let weekly = is_weekly(input);
    let var_driver = |var: VarCode| -> Option<DriverFact> {
        let rate = if var == VarCode::Staff {
            turnover_changes(
                input.turnover_before_bp,
                input.turnover_after_bp,
                first,
                weekly,
            )
            .to_vec()
        } else {
            Vec::new()
        };
        let changes: Vec<ChangeFact> = rate
            .into_iter()
            .chain(lines.iter().filter(|l| l.var == var).filter_map(|l| {
                let pushed = push(l, room_revenue, maintained, morale_alone);
                let mut amount = amount_text(l, shows_sign(l.var, &l.cause))?;
                if l.var == VarCode::Staff && l.cause == CauseDto::Turnover {
                    amount = with_departments(amount, l.amount, &input.department_departures);
                }
                Some(ChangeFact {
                    amount,
                    cause: cause_label(l.var, &l.cause, &titles, weekly),
                    because: cause_because(l.var, &l.cause, &titles, weekly),
                    harm: harm_of(pushed),
                    effect: Effect::of(pushed),
                })
            }))
            // Obnova v2: retci koje objašnjenje kruga nema (v1 ih nikad ne dobiva).
            .chain(match var {
                VarCode::Equipment => equipment_rows(input),
                VarCode::Revenue => revenue_rows(input),
                _ => Vec::new(),
            })
            .collect();
        (!changes.is_empty()).then(|| DriverFact {
            source: DriverSource::Var(var),
            what: var_name(var),
            meaning: var_meaning(var, weekly),
            changes,
        })
    };
    let indices = Index::ALL
        .iter()
        .zip(input.indices_before.iter().zip(&input.indices_after))
        .map(|(&index, (&b, &a))| {
            let before = round_div(i64::from(b), 100);
            let after = round_div(i64::from(a), 100);
            IndexFact {
                index,
                name: index_name(index),
                before,
                after,
                change: after - before,
                direction: direction(after - before),
                drivers: index_drivers(index)
                    .iter()
                    .filter_map(|d| match d {
                        Driver::Var(v) => var_driver(VarCode::from(v)),
                        Driver::StaffLoad => load_driver(
                            input.load_before_hundredths,
                            input.load_after_hundredths,
                            first,
                            weekly,
                        ),
                    })
                    .collect(),
            }
        })
        .collect();
    let situations = input
        .situations
        .iter()
        .map(|s| SituationFact {
            title: s.title.clone(),
            grade: format!("{}/5", s.quality),
            grade_words: grade_words(s.quality),
            free_text: s.free_text,
            chosen: match (&s.chosen_label, s.free_text && !s.unanswered) {
                (Some(l), _) => Some(l.clone()),
                (None, true) => Some("vlastiti odgovor, napisan svojim riječima".to_owned()),
                (None, false) => None,
            },
            unanswered: s.unanswered,
            delegated: s.delegated,
            rationale: s.rationale.clone(),
            rule: RuleFacts::of(s),
            quality: s.quality,
            chosen_label: s.chosen_label.clone(),
        })
        .collect();
    let progress = round_div(i64::from(input.progress_hundredths), 100);
    DebriefFacts {
        hotel: input.hotel_name.clone(),
        month: place_text(input),
        next_month: input.next_round.map(next_text),
        progress: if progress == 0 {
            "0 bodova".to_owned()
        } else {
            points(progress)
        },
        cash_end: euros(input.cash_end_cents, false),
        index_scale: per(
            weekly,
            "Indeksi idu do 100 bodova; 50 bodova je prosjek usporedivog hotela u istom mjesecu. Indeks pokazuje kako stojiš u odnosu na njega, pa može pasti i kad on napreduje brže od tebe.",
            "Indeksi idu do 100 bodova; 50 bodova je prosjek usporedivog hotela u istom tjednu. Indeks pokazuje kako stojiš u odnosu na njega, pa može pasti i kad on napreduje brže od tebe.",
        ),
        plan: plan_facts(input),
        indices,
        situations,
        investments: investment_facts(input),
        finance_note: finance_note(input),
    }
}

// ---------------------------------------------------------------- mentor bez AI-ja

/// Jedan savjet po indeksu, za najgori indeks mjeseca.
pub(crate) fn advice(i: Index) -> &'static str {
    match i {
        Index::Guests => {
            "Gledaj boravak očima gosta: cijena treba odgovarati onome što dobije, a uredne sobe i odmoran tim najbrže podižu zadovoljstvo."
        }
        Index::Reviews => {
            "Ocjena na portalima prati zadovoljstvo gostiju; odgovori na recenzije pomažu, ali najviše pomaže gost koji ode zadovoljan."
        }
        Index::Staff => {
            "Razgovaraj s ljudima prije nego problem naraste: poštena plaća, razumno opterećenje i obuka drže tim na okupu."
        }
        Index::Operations => {
            "Ne štedi na održavanju i ne preopterećuj osoblje; obuka nadoknađuje znanje koje odlazi s iskusnim ljudima."
        }
        Index::Finance => {
            "Prije svakog troška pitaj se što će vratiti i provjeri odgovara li cijena sobe onome što gosti dobivaju."
        }
    }
}

pub(crate) fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// Zašto je indeks pao: najveća šteta među njegovim pokretačima, rečenicom.
fn why_fell(i: &IndexFact) -> String {
    let worst = i
        .drivers
        .iter()
        .flat_map(|d| d.changes.iter().map(move |c| (d.source, c)))
        .filter(|(_, c)| c.harm > 0)
        .fold(
            None,
            |best: Option<(DriverSource, &ChangeFact)>, cur| match best {
                Some(b) if b.1.harm >= cur.1.harm => Some(b),
                _ => Some(cur),
            },
        );
    match worst {
        Some((source, c)) => {
            let then = consequence(source);
            let tail = if then.is_empty() {
                String::new()
            } else {
                format!(", {then}")
            };
            format!(
                "Indeks {} pao je ponajviše zbog {}{tail}.",
                i.name, c.because
            )
        }
        None => format!(
            "Indeks {} pao je bez jednog velikog krivca: mjeri se prema usporedivom hotelu, pa pada i kad drugi napreduju brže.",
            i.name
        ),
    }
}

/// Treba li za ovaj krug zvati AI mentora (P3): v1 i mjesečna v2 uvijek; tjedna samo kad u krugu završava mjesec.
pub fn wants_ai_mentor(g: &Game, r: RoundKey) -> Result<bool, GameError> {
    let period = g.round_info(r)?.period;
    Ok(period.kind == sim_hotel_v2::PeriodKind::Month || period.month_ending_in().is_some())
}

/// Kratki tjedni sažetak bez AI-ja („tri uzroka") za tjedan u kojem ne završava mjesec.
pub fn weekly_summary(input: &DebriefInput) -> String {
    let f = debrief_facts(input);
    let mut out = format!(
        "{} je napredak tvog hotela od početka igre {}.\n\n",
        capitalize(&f.month),
        f.progress
    );
    let mut moved: Vec<&IndexFact> = f.indices.iter().filter(|i| i.change != 0).collect();
    // Stabilno sortiranje: jednaki ostaju u redoslijedu INDEX_ORDER.
    moved.sort_by_key(|i| std::cmp::Reverse(i.change.abs()));
    let causes: Vec<String> = moved
        .iter()
        .take(3)
        .map(|i| {
            if i.change < 0 {
                why_fell(i)
            } else {
                format!(
                    "Indeks {} porastao je s {} na {}.",
                    i.name, i.before, i.after
                )
            }
        })
        .collect();
    out.push_str("Tri uzroka\n");
    if causes.is_empty() {
        out.push_str("Ovaj tjedan se nijedan indeks nije pomaknuo.");
    } else {
        out.push_str(&causes.join("\n"));
    }
    out.push_str("\n\nPuni pregled s mentorom stiže na kraju mjeseca.");
    out
}

/// Mentor bez AI-ja (ARHITEKTURA §8.5 „bez AI-ja"): ista četiri dijela kao AI, iz istih činjenica.
pub fn fallback_debrief(input: &DebriefInput) -> String {
    let f = debrief_facts(input);
    let mut out = format!(
        "{} je napredak tvog hotela od početka igre {}.\n\n",
        capitalize(&f.month),
        f.progress
    );

    let mut risen: Vec<&IndexFact> = f.indices.iter().filter(|i| i.change > 0).collect();
    risen.sort_by_key(|i| std::cmp::Reverse(i.change));
    let mut good: Vec<String> = risen
        .iter()
        .take(2)
        .map(|i| {
            format!(
                "Indeks {} porastao je s {} na {}.",
                i.name, i.before, i.after
            )
        })
        .collect();
    good.extend(
        f.situations
            .iter()
            .filter(|s| s.quality >= 4)
            .map(|s| format!("Situacija „{}“ riješena je dobro ({}).", s.title, s.grade)),
    );
    if good.is_empty() {
        good.push(
            per(
                is_weekly(input),
                "Ovaj mjesec nijedan indeks nije porastao i nijedna situacija nije riješena dobro.",
                "Ovaj tjedan nijedan indeks nije porastao i nijedna situacija nije riješena dobro.",
            )
            .to_owned(),
        );
    }
    out.push_str(&format!("Što je išlo dobro\n{}\n\n", good.join(" ")));

    let mut fallen: Vec<&IndexFact> = f.indices.iter().filter(|i| i.change < 0).collect();
    fallen.sort_by_key(|i| i.change);
    fallen.truncate(2);
    let mut bad: Vec<String> = fallen
        .iter()
        .map(|i| format!("Indeks {} pao je s {} na {}.", i.name, i.before, i.after))
        .collect();
    bad.extend(f.situations.iter().filter(|s| s.quality <= 2).map(|s| {
        match (s.unanswered, &s.chosen_label) {
            _ if s.delegated => format!(
                "Situacija „{}“ prepuštena je voditelju ({}).",
                s.title, s.grade
            ),
            (true, _) => format!(
                "Na situaciju „{}“ nije bilo odgovora, pa je dobila najnižu ocjenu ({}).",
                s.title, s.grade
            ),
            (false, Some(label)) => format!(
                "Situacija „{}“ riješena je loše ({}): izabran je odgovor „{label}“.",
                s.title, s.grade
            ),
            (false, None) => format!("Situacija „{}“ riješena je loše ({}).", s.title, s.grade),
        }
    }));
    if bad.is_empty() {
        bad.push("Nijedan indeks nije pao i nijedna situacija nije riješena loše.".to_owned());
    }
    out.push_str(&format!("Što nije valjalo\n{}\n\n", bad.join(" ")));

    let why: Vec<String> = fallen.iter().map(|i| why_fell(i)).collect();
    let why = if why.is_empty() {
        "Nema pada koji treba objasniti.".to_owned()
    } else {
        why.join(" ")
    };
    out.push_str(&format!("Zašto\n{why}\n\n"));

    // Najgori indeks: najveći pad, a kad ništa nije palo, najniži na kraju mjeseca.
    let worst = fallen.first().copied().or_else(|| {
        f.indices
            .iter()
            .fold(None, |best: Option<&IndexFact>, cur| match best {
                Some(b) if b.after <= cur.after => Some(b),
                _ => Some(cur),
            })
    });
    let when = f.next_month.as_deref().unwrap_or("sljedeći put");
    if let Some(w) = worst {
        out.push_str(&format!("Što probati {when}\n{}\n\n", advice(w.index)));
    }

    for s in f.situations.iter().filter(|s| s.rule.neutral.is_some()) {
        out.push_str(&format!(
            "Napomena: situacija „{}“ dobila je neutralnu ocjenu {} jer odgovor nije stigao biti ocijenjen; čeka pregled.\n\n",
            s.title, s.grade
        ));
    }
    for note in fallback_notes(input) {
        out.push_str(&note);
        out.push_str("\n\n");
    }
    out.push_str("AI objašnjenje danas nije bilo dostupno, ovo je sažetak iz brojki.");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const VARS: [VarCode; 10] = [
        VarCode::Sold,
        VarCode::Revenue,
        VarCode::Costs,
        VarCode::Cash,
        VarCode::Staff,
        VarCode::Morale,
        VarCode::Experience,
        VarCode::Equipment,
        VarCode::Satisfaction,
        VarCode::Rating,
    ];

    fn causes() -> Vec<CauseDto> {
        vec![
            CauseDto::Baseline,
            CauseDto::Price,
            CauseDto::Reputation,
            CauseDto::Marketing,
            CauseDto::Groups,
            CauseDto::Capacity,
            CauseDto::OtherRevenue,
            CauseDto::Wage,
            CauseDto::Load,
            CauseDto::Training,
            CauseDto::Hires,
            CauseDto::Release,
            CauseDto::Turnover,
            CauseDto::Maintenance,
            CauseDto::Renovation,
            CauseDto::ReplyReviews,
            CauseDto::FixedCosts,
            CauseDto::Interest,
            CauseDto::Situation("S9".into()),
        ]
    }

    /// Svaki par varijable i uzroka ima vlastitu frazu u genitivu, bez općeg „stavke …".
    #[test]
    fn every_cause_has_its_own_because_phrase() {
        let titles = BTreeMap::from([("S9", "Izmišljena situacija")]);
        for v in VARS {
            for c in causes() {
                let b = cause_because(v, &c, &titles, false);
                let w = cause_because(v, &c, &titles, true);
                assert!(!w.contains("mjesec"), "{v:?} {c:?}: {w}");
                assert!(!b.is_empty() && !b.contains("stavk"), "{v:?} {c:?}: {b}");
                assert!(!b.contains("S9"), "{b}");
            }
        }
        assert_eq!(
            cause_because(VarCode::Sold, &CauseDto::Marketing, &titles, false),
            "premalo marketinga"
        );
    }

    /// Tekstovi o odlascima kažu „60 bodova" i „8 %": drže se formule iz jezgre.
    #[test]
    fn turnover_words_follow_core_formula() {
        assert_eq!(sim_hotel::turnover_bp(6_000).unwrap(), 300);
        assert!(sim_hotel::turnover_bp(5_900).unwrap() > 300);
        assert_eq!(retention(799).unwrap(), 12);
        assert_eq!(retention(800).unwrap(), 0);
        let [now, last] = turnover_changes(375, 443, false, false);
        assert_eq!((now.harm, now.effect), (500 * 68, Effect::Down));
        assert_eq!((last.harm, last.effect), (0, Effect::Neutral));
    }

    #[test]
    fn decimals_and_plurals() {
        assert_eq!(decimal_hundredths(230), "2,3");
        assert_eq!(decimal_hundredths(185), "1,85");
        assert_eq!(decimal_hundredths(200), "2");
        assert_eq!(people_text(&CauseDto::Turnover, 3), "otišle 3 osobe");
        assert_eq!(people_text(&CauseDto::Hires, 6), "zaposleno 6 osoba");
        assert_eq!(people_text(&CauseDto::Hires, 1), "zaposlena 1 osoba");
        assert_eq!(people_text(&CauseDto::Release, -2), "otpuštene 2 osobe");
        assert_eq!(people_text(&CauseDto::Turnover, 21), "otišla 21 osoba");
    }
}
