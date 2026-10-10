//! Mentor govori ljudski (ADR-023): iz `DebriefInput` se grade činjenice samo s ljudskim vrijednostima
//! (imena indeksa, cijeli bodovi, euri, naslovi situacija). I AI mentor i mentor bez AI-ja pišu iz njih,
//! pa nijedna interna jedinica (stotinke, centi, baze, kodovi) ne izlazi studentu.

use std::collections::BTreeMap;

use serde::Serialize;
use sim_hotel::{Index, Var, index_drivers};

use crate::ai_input::DebriefInput;
use crate::dto::{CauseDto, LineDto, MonthCode, UnitCode, VarCode};

const MINUS: char = '\u{2212}';

/// Činjenice za mentora; ključevi se čitaju kao rečenice, vrijednosti su već u ljudskim jedinicama.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DebriefFacts {
    pub hotel: String,
    /// Mjesec u lokativu s prijedlogom („u srpnju").
    pub mjesec: &'static str,
    /// `None` = bio je zadnji mjesec igre.
    pub iduci_mjesec: Option<&'static str>,
    pub napredak_od_pocetka: String,
    pub blagajna_na_kraju_mjeseca: String,
    pub ljestvica_indeksa: &'static str,
    pub indeksi: Vec<IndexFact>,
    pub situacije: Vec<SituationFact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IndexFact {
    #[serde(skip)]
    pub index: Index,
    pub naziv: &'static str,
    pub prije: i64,
    pub poslije: i64,
    /// Razlika zaokruženih vrijednosti, da se slaže s ekranom.
    pub promjena: i64,
    pub smjer: &'static str,
    pub sto_je_pokrenulo: Vec<DriverFact>,
}

/// Jedna varijabla koja ulazi u indeks, s promjenama ovog mjeseca.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DriverFact {
    #[serde(skip)]
    pub var: VarCode,
    pub sto: &'static str,
    pub znacenje: &'static str,
    pub promjene: Vec<ChangeFact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChangeFact {
    pub uzrok: String,
    pub iznos: String,
    /// Koliko je uzrok gurnuo indeks dolje; uspoređuje se samo unutar istog indeksa. 0 = nije šteta.
    #[serde(skip)]
    pub harm: i64,
    /// Uzrok u genitivu („zbog nižih plaća") za mentora bez AI-ja.
    #[serde(skip)]
    pub because: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SituationFact {
    pub naslov: String,
    pub ocjena: String,
    pub ocjena_rijecima: &'static str,
    pub slobodan_odgovor: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub bez_odgovora: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obrazlozenje: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neutralna_ocjena: Option<&'static str>,
    #[serde(skip)]
    pub quality: u8,
}

fn is_false(b: &bool) -> bool {
    !*b
}

// ---------------------------------------------------------------- nazivi

fn index_name(i: Index) -> &'static str {
    match i {
        Index::Guests => "Zadovoljstvo gostiju",
        Index::Reviews => "Recenzije",
        Index::Staff => "Zaposlenici",
        Index::Operations => "Operativa",
        Index::Finance => "Financije",
    }
}

/// Mjesec u lokativu („u lipnju").
fn month_locative(m: MonthCode) -> &'static str {
    match m {
        MonthCode::Jun => "u lipnju",
        MonthCode::Jul => "u srpnju",
        MonthCode::Aug => "u kolovozu",
        MonthCode::Sep => "u rujnu",
    }
}

fn var_of(v: VarCode) -> Var {
    match v {
        VarCode::Sold => Var::Sold,
        VarCode::Revenue => Var::Revenue,
        VarCode::Costs => Var::Costs,
        VarCode::Cash => Var::Cash,
        VarCode::Staff => Var::Staff,
        VarCode::Morale => Var::Morale,
        VarCode::Experience => Var::Experience,
        VarCode::Equipment => Var::Equipment,
        VarCode::Satisfaction => Var::Satisfaction,
        VarCode::Rating => Var::Rating,
    }
}

/// Redoslijed kojim se varijable navode uz indeks.
const VAR_ORDER: [VarCode; 10] = [
    VarCode::Satisfaction,
    VarCode::Rating,
    VarCode::Morale,
    VarCode::Staff,
    VarCode::Equipment,
    VarCode::Experience,
    VarCode::Sold,
    VarCode::Revenue,
    VarCode::Costs,
    VarCode::Cash,
];

/// Nazivi kao `var.*` u `packages/i18n/src/hr.ts`.
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

/// Kako čitati iznose uz varijablu (osobito moral, gdje su uzroci pritisci, a ne cijela promjena).
fn var_meaning(v: VarCode) -> &'static str {
    match v {
        VarCode::Sold => {
            "Koliko je noćenja hotel prodao. Postoci kažu koliko je zbog nečega potražnja bila veća ili manja."
        }
        VarCode::Revenue => "Novac koji je hotel dobio od gostiju ovaj mjesec, po izvoru.",
        VarCode::Costs => "Što je hotel platio ovaj mjesec, po vrsti troška.",
        VarCode::Cash => "Kako se mijenjao novac na računu hotela.",
        VarCode::Staff => {
            "Promjene u broju ljudi; odlasci su ljudi koji su dali otkaz i zamijenjeni su novima."
        }
        VarCode::Morale => {
            "Raspoloženje i volja tima, od 0 do 100. Moral se svaki mjesec pomakne samo dio puta prema razini koju guraju plaće, opterećenje, obuka i situacije; zato je ukupna promjena manja od zbroja tih pritisaka."
        }
        VarCode::Experience => {
            "Koliko tim zna i može, od 0 do 100. Raste radom i obukom, pada kad iskusni ljudi odu."
        }
        VarCode::Equipment => {
            "Stanje soba i opreme, od 0 do 100. Svaki mjesec se troši; održavanje i obnova je vraćaju."
        }
        VarCode::Satisfaction => "Koliko su gosti zadovoljni boravkom, od 0 do 100.",
        VarCode::Rating => {
            "Prosječna ocjena hotela na portalima za rezervacije, na ljestvici do 10."
        }
    }
}

/// Posljedica pada varijable, za rečenicu „zašto" mentora bez AI-ja.
fn var_consequence(v: VarCode) -> &'static str {
    match v {
        VarCode::Morale => "pa je tim slabije raspoložen i lakše odlazi",
        VarCode::Experience => "pa tim zna i može manje nego prije",
        VarCode::Equipment => "pa su sobe i oprema u lošijem stanju",
        VarCode::Costs => "pa je hotel potrošio više",
        VarCode::Cash => "pa je na računu ostalo manje novca",
        VarCode::Sold => "pa je prodano manje noćenja",
        // Kod gostiju i ocjene posljedica je sam indeks; mehanizam već stoji u uzroku.
        VarCode::Satisfaction | VarCode::Rating | VarCode::Staff | VarCode::Revenue => "",
    }
}

/// Uzrok u nominativu; „osnova" dobiva značenje po varijabli. Ostalo kao `cause.*` u `hr.ts`.
fn cause_label(v: VarCode, c: &CauseDto, titles: &BTreeMap<&str, &str>) -> String {
    let s = match (v, c) {
        (_, CauseDto::Situation(code)) => return format!("situacija „{}“", title(titles, code)),
        (VarCode::Sold, CauseDto::Baseline) => "osnovna potražnja za ovaj mjesec",
        (VarCode::Revenue, CauseDto::Baseline) => "noćenja pojedinačnih gostiju",
        (VarCode::Costs, CauseDto::Baseline) => "troškovi po prodanom noćenju",
        (VarCode::Cash, CauseDto::Baseline) => "zarada iz poslovanja (prihod minus troškovi)",
        (VarCode::Morale, CauseDto::Baseline) => "ukupna promjena morala u mjesecu",
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
        (_, CauseDto::FixedCosts) => "fiksni troškovi",
        (_, CauseDto::Interest) => "kamate",
    };
    s.to_owned()
}

/// Uzrok štete u genitivu, za „pao je ponajviše zbog …".
fn cause_because(v: VarCode, c: &CauseDto, label: &str, titles: &BTreeMap<&str, &str>) -> String {
    let s = match (v, c) {
        (_, CauseDto::Situation(code)) => {
            return format!("rješenja situacije „{}“", title(titles, code));
        }
        (VarCode::Morale, CauseDto::Wage) => "nižih plaća",
        (_, CauseDto::Load) => "prevelikog opterećenja osoblja",
        (VarCode::Morale, CauseDto::Release) => "otpuštanja dijela osoblja",
        (VarCode::Experience, CauseDto::Turnover) => "odlaska iskusnih ljudi",
        (VarCode::Equipment, CauseDto::Baseline) => {
            "trošenja opreme koje održavanje nije nadoknadilo"
        }
        (VarCode::Equipment, CauseDto::Maintenance) => "slabijeg održavanja",
        (VarCode::Satisfaction, CauseDto::Baseline) => {
            "slabije usluge, opreme ili cijene: gost to osjeti kad je tim umoran ili neiskusan, sobe istrošene ili cijena previsoka za ono što dobije"
        }
        (VarCode::Rating, CauseDto::Baseline) => {
            "nižeg zadovoljstva gostiju: nezadovoljni gosti pišu slabije recenzije"
        }
        (VarCode::Sold, CauseDto::Price) => "previsoke cijene",
        (VarCode::Sold, CauseDto::Reputation) => "slabijeg ugleda na portalima",
        (VarCode::Cash, CauseDto::Baseline) => "gubitka u poslovanju (troškovi veći od prihoda)",
        (VarCode::Cash, CauseDto::Renovation) => "ulaganja u obnovu",
        (VarCode::Cash, CauseDto::Interest) => "kamata na minus na računu",
        _ => return format!("stavke „{label}“"),
    };
    s.to_owned()
}

fn title<'a>(titles: &BTreeMap<&str, &'a str>, code: &str) -> &'a str {
    titles
        .get(code)
        .copied()
        .unwrap_or("situacija iz ovog mjeseca")
}

// ---------------------------------------------------------------- brojke

/// Cjelobrojno dijeljenje zaokruženo pola od nule (4850/100 → 49, 4849/100 → 48, −453/100 → −5).
fn round_div(v: i64, d: i64) -> i64 {
    let half = d / 2;
    if v >= 0 {
        (v + half) / d
    } else {
        -((-v + half) / d)
    }
}

/// Točka za tisuće: 1234567 → „1.234.567".
fn thousands(n: u64) -> String {
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

/// „+" ili „−" ispred (nula bez predznaka) kad je `signed`; inače samo iznos.
fn with_sign(n: i64, body: String, signed: bool) -> String {
    match (signed, n.signum()) {
        (true, 1) => format!("+{body}"),
        (_, -1) => format!("{MINUS}{body}"),
        _ => body,
    }
}

/// Hrvatski oblik uz broj: 1 bod, 2 boda, 5 bodova (21 bod, 12 bodova).
fn plural(n: u64, one: &'static str, few: &'static str, many: &'static str) -> &'static str {
    let (d, h) = (n % 10, n % 100);
    if d == 1 && h != 11 {
        one
    } else if (2..=4).contains(&d) && !(12..=14).contains(&h) {
        few
    } else {
        many
    }
}

fn euros(cents: i64, signed: bool) -> String {
    let e = round_div(cents, 100);
    with_sign(e, format!("{} €", thousands(e.unsigned_abs())), signed)
}

fn points(n: i64) -> String {
    let a = n.unsigned_abs();
    with_sign(
        n,
        format!("{} {}", thousands(a), plural(a, "bod", "boda", "bodova")),
        true,
    )
}

/// Iznos retka u ljudskoj jedinici; `None` kad je (zaokruženo) nula pa se redak izostavlja.
fn amount_text(l: &LineDto, signed: bool) -> Option<String> {
    let a = l.amount;
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
                    plural(n, "noćenje", "noćenja", "noćenja")
                ),
                signed,
            )
        }
        UnitCode::People => {
            let n = a.unsigned_abs();
            with_sign(
                a,
                format!("{n} {}", plural(n, "osoba", "osobe", "osoba")),
                signed,
            )
        }
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
    (a != 0).then_some(text)
}

/// Iznosi su razine (prihod, troškovi, osnovna potražnja, broj odlazaka), ne promjene: bez predznaka.
fn shows_sign(v: VarCode, c: &CauseDto) -> bool {
    !matches!(
        (v, c),
        (VarCode::Revenue | VarCode::Costs, _)
            | (VarCode::Sold, CauseDto::Baseline)
            | (VarCode::Staff, CauseDto::Turnover)
    )
}

/// Gruba težina štete za usporedbu unutar istog indeksa (mentor bez AI-ja bira najveću).
/// `room_revenue` (prihod od noćenja) pretvara postotak potražnje u izgubljeni prihod; `maintained`
/// je koliko su održavanje i obnova vratili opremu, pa trošenje je šteta samo u dijelu koji nisu pokrili.
fn harm(l: &LineDto, room_revenue: i64, maintained: i64) -> i64 {
    let loss = (-l.amount).max(0);
    let h = match (l.var, &l.cause) {
        (VarCode::Costs, CauseDto::Situation(_)) => l.amount.max(0),
        (VarCode::Revenue | VarCode::Costs | VarCode::Staff, _) => 0,
        (VarCode::Morale, CauseDto::Baseline) => 0,
        (VarCode::Sold, CauseDto::Price | CauseDto::Reputation | CauseDto::Marketing) => {
            room_revenue.max(1).saturating_mul(loss) / 10_000
        }
        (VarCode::Sold, _) => 0,
        (VarCode::Equipment, CauseDto::Baseline) => (loss - maintained).max(0) * 45,
        // Oprema teže ulazi u Operativu od iskustva (45 : 30 u `raw_indices`).
        (VarCode::Equipment, _) => loss * 45,
        (VarCode::Experience, _) => loss * 30,
        _ => loss,
    };
    h.max(0)
}

// ---------------------------------------------------------------- činjenice

fn grade_words(q: u8) -> &'static str {
    match q {
        4.. => "dobro",
        3 => "srednje",
        _ => "loše",
    }
}

fn direction(change: i64) -> &'static str {
    match change.signum() {
        1 => "porastao",
        -1 => "pao",
        _ => "isti",
    }
}

pub fn debrief_facts(input: &DebriefInput) -> DebriefFacts {
    let titles: BTreeMap<&str, &str> = input
        .situations
        .iter()
        .map(|s| (s.code.as_str(), s.title.as_str()))
        .collect();
    let room_revenue: i64 = input
        .lines
        .iter()
        .filter(|l| l.var == VarCode::Revenue && l.cause == CauseDto::Baseline)
        .map(|l| l.amount)
        .sum();
    let maintained: i64 = input
        .lines
        .iter()
        .filter(|l| {
            l.var == VarCode::Equipment
                && matches!(l.cause, CauseDto::Maintenance | CauseDto::Renovation)
        })
        .map(|l| l.amount.max(0))
        .sum();
    let drivers_of = |var: VarCode| -> Option<DriverFact> {
        let promjene: Vec<ChangeFact> = input
            .lines
            .iter()
            .filter(|l| l.var == var)
            .filter_map(|l| {
                let iznos = amount_text(l, shows_sign(l.var, &l.cause))?;
                let uzrok = cause_label(l.var, &l.cause, &titles);
                Some(ChangeFact {
                    because: cause_because(l.var, &l.cause, &uzrok, &titles),
                    harm: harm(l, room_revenue, maintained),
                    uzrok,
                    iznos,
                })
            })
            .collect();
        (!promjene.is_empty()).then(|| DriverFact {
            var,
            sto: var_name(var),
            znacenje: var_meaning(var),
            promjene,
        })
    };
    let indeksi = Index::ALL
        .iter()
        .zip(input.indices_before.iter().zip(&input.indices_after))
        .map(|(&index, (&b, &a))| {
            let prije = round_div(i64::from(b), 100);
            let poslije = round_div(i64::from(a), 100);
            let drivers = index_drivers(index);
            IndexFact {
                index,
                naziv: index_name(index),
                prije,
                poslije,
                promjena: poslije - prije,
                smjer: direction(poslije - prije),
                sto_je_pokrenulo: VAR_ORDER
                    .iter()
                    .filter(|&&v| drivers.contains(&var_of(v)))
                    .filter_map(|&v| drivers_of(v))
                    .collect(),
            }
        })
        .collect();
    let situacije = input
        .situations
        .iter()
        .map(|s| SituationFact {
            naslov: s.title.clone(),
            ocjena: format!("{}/5", s.quality),
            ocjena_rijecima: grade_words(s.quality),
            slobodan_odgovor: s.free_text,
            bez_odgovora: s.unanswered,
            obrazlozenje: s.rationale.clone(),
            neutralna_ocjena: s.neutral.then_some(
                "AI nije stigao ocijeniti odgovor, pa je dobio srednju ocjenu 3/5; odgovor čeka pregled.",
            ),
            quality: s.quality,
        })
        .collect();
    let progress = round_div(i64::from(input.progress_hundredths), 100);
    DebriefFacts {
        hotel: input.hotel_name.clone(),
        mjesec: month_locative(input.month),
        iduci_mjesec: input.next_month.map(month_locative),
        napredak_od_pocetka: if progress == 0 {
            "0 bodova".to_owned()
        } else {
            points(progress)
        },
        blagajna_na_kraju_mjeseca: euros(input.cash_end_cents, false),
        ljestvica_indeksa: "Indeksi su od 0 do 100; 50 je prosjek usporedivog hotela u istom mjesecu. Indeks pokazuje kako stojiš u odnosu na njega, pa može pasti i kad on napreduje brže od tebe.",
        indeksi,
        situacije,
    }
}

// ---------------------------------------------------------------- mentor bez AI-ja

/// Jedan savjet po indeksu, za najgori indeks mjeseca.
fn advice(i: Index) -> &'static str {
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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// Zašto je indeks pao: najveća šteta među njegovim varijablama, rečenicom.
fn why_fell(i: &IndexFact) -> String {
    let worst = i
        .sto_je_pokrenulo
        .iter()
        .flat_map(|d| d.promjene.iter().map(move |c| (d.var, c)))
        .filter(|(_, c)| c.harm > 0)
        .fold(
            None,
            |best: Option<(VarCode, &ChangeFact)>, cur| match best {
                Some(b) if b.1.harm >= cur.1.harm => Some(b),
                _ => Some(cur),
            },
        );
    match worst {
        Some((var, c)) => {
            let then = var_consequence(var);
            let tail = if then.is_empty() {
                String::new()
            } else {
                format!(", {then}")
            };
            format!(
                "Indeks {} pao je ponajviše zbog {}{tail}.",
                i.naziv, c.because
            )
        }
        None => format!(
            "Indeks {} pao je bez jednog velikog krivca: mjeri se prema usporedivom hotelu, pa pada i kad drugi napreduju brže.",
            i.naziv
        ),
    }
}

/// Mentor bez AI-ja (ARHITEKTURA §8.5 „bez AI-ja"): ista četiri dijela kao AI, iz istih činjenica.
pub fn fallback_debrief(input: &DebriefInput) -> String {
    let f = debrief_facts(input);
    let mut out = format!(
        "{} je napredak tvog hotela od početka igre {}.\n\n",
        capitalize(f.mjesec),
        f.napredak_od_pocetka
    );

    let mut risen: Vec<&IndexFact> = f.indeksi.iter().filter(|i| i.promjena > 0).collect();
    risen.sort_by_key(|i| std::cmp::Reverse(i.promjena));
    let mut good: Vec<String> = risen
        .iter()
        .take(2)
        .map(|i| {
            format!(
                "Indeks {} porastao je s {} na {}.",
                i.naziv, i.prije, i.poslije
            )
        })
        .collect();
    good.extend(
        f.situacije
            .iter()
            .filter(|s| s.quality >= 4)
            .map(|s| format!("Situacija „{}“ riješena je dobro ({}).", s.naslov, s.ocjena)),
    );
    if good.is_empty() {
        good.push(
            "Ovaj mjesec nijedan indeks nije porastao i nijedna situacija nije riješena dobro."
                .to_owned(),
        );
    }
    out.push_str(&format!("Što je išlo dobro\n{}\n\n", good.join(" ")));

    let mut fallen: Vec<&IndexFact> = f.indeksi.iter().filter(|i| i.promjena < 0).collect();
    fallen.sort_by_key(|i| i.promjena);
    fallen.truncate(2);
    let mut bad: Vec<String> = fallen
        .iter()
        .map(|i| format!("Indeks {} pao je s {} na {}.", i.naziv, i.prije, i.poslije))
        .collect();
    bad.extend(f.situacije.iter().filter(|s| s.quality <= 2).map(|s| {
        if s.bez_odgovora {
            format!(
                "Na situaciju „{}“ nije bilo odgovora, pa je dobila najnižu ocjenu ({}).",
                s.naslov, s.ocjena
            )
        } else {
            format!("Situacija „{}“ riješena je loše ({}).", s.naslov, s.ocjena)
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
        f.indeksi
            .iter()
            .fold(None, |best: Option<&IndexFact>, cur| match best {
                Some(b) if b.poslije <= cur.poslije => Some(b),
                _ => Some(cur),
            })
    });
    let when = f.iduci_mjesec.unwrap_or("sljedeći put");
    if let Some(w) = worst {
        out.push_str(&format!("Što probati {when}\n{}\n\n", advice(w.index)));
    }

    for s in f.situacije.iter().filter(|s| s.neutralna_ocjena.is_some()) {
        out.push_str(&format!(
            "Napomena: situacija „{}“ dobila je neutralnu ocjenu {} jer odgovor nije stigao biti ocijenjen; čeka pregled.\n\n",
            s.naslov, s.ocjena
        ));
    }
    out.push_str("AI objašnjenje danas nije bilo dostupno, ovo je sažetak iz brojki.");
    out
}
