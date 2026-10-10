//! Sloj 0 — hotelski pokazatelji (MODEL_V2 §5, PROSIRENJA P2): čiste funkcije nad obračunatim mjesecom.
//! Ništa se ne sprema; računa se pri čitanju iz `MonthReport` (ili iz istih brojki spremljenih u dnevniku).
//!
//! Formule (USALI/STR, `docs/research/HOTELSKI_POKAZATELJI.md` §1), s jednim pravilom za ponudu soba:
//! sve „po sobi" dijeli se sa **svim** sobama × dani (`room_nights`), kao GOPPAR u obračunu, pa vrijedi
//! RevPAR = ADR × popunjenost. Sobe izvan funkcije prikazuju se zasebno kao izgubljene noći.
//! Iznosi su u centima, udjeli u baznim bodovima (10 000 = 100 %); dijeljenje nulom daje `None`.

use engine::calc::{self as c, Calc, CalcError};

use crate::catalog::{Catalog, CatalogError};
use crate::resolve::MonthReport;

/// Brojke mjeseca iz kojih se računaju pokazatelji (sve postoje u `MonthReport`; izravni trošak po noći je
/// iz kataloga kategorije).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KpiInput {
    pub rooms: u16,
    pub days: u8,
    /// Raspoložive noći (sobe u funkciji × dani).
    pub available: u32,
    pub sold: u32,
    pub rooms_revenue_cents: i64,
    pub other_revenue_cents: i64,
    pub labor_cents: i64,
    /// Svi operativni troškovi mjeseca (zbroj kojim obračun dolazi do GOP-a).
    pub costs_cents: i64,
    pub gop_cents: i64,
    /// Izravni trošak po prodanoj noći (`cpor_cents` kategorije u katalogu).
    pub cpor_cents: i64,
}

impl KpiInput {
    pub fn from_report(r: &MonthReport, cat: &Catalog) -> Result<KpiInput, CatalogError> {
        Ok(KpiInput {
            rooms: r.before.rooms,
            days: r.month.days(),
            available: r.market.available,
            sold: r.market.sold,
            rooms_revenue_cents: r.money.rooms_revenue_cents,
            other_revenue_cents: r.money.other_revenue_cents,
            labor_cents: r.money.labor_cents,
            costs_cents: r.money.costs_cents,
            gop_cents: r.money.gop_cents,
            cpor_cents: cat.category(r.before.category)?.cpor_cents,
        })
    }
}

/// Pokazatelji jednog obračunatog mjeseca.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthKpi {
    /// Sve sobe × dani mjeseca (ponuda).
    pub room_nights: u32,
    pub sold_nights: u32,
    /// Noći soba izvan funkcije (oprema): ponuda koja se nije mogla prodati.
    pub lost_nights: u32,
    /// Prodane noći ÷ sve noći.
    pub occupancy_bp: Option<i64>,
    /// Prihod od soba ÷ prodane noći (prosječna cijena prodane sobe).
    pub adr_cents: Option<i64>,
    /// Prihod od soba ÷ sve noći.
    pub revpar_cents: Option<i64>,
    pub rooms_revenue_cents: i64,
    /// Prihod od soba + ostali prihod.
    pub total_revenue_cents: i64,
    /// Ukupni prihod ÷ sve noći.
    pub trevpar_cents: Option<i64>,
    pub gop_cents: i64,
    /// GOP ÷ ukupni prihod.
    pub gop_margin_bp: Option<i64>,
    /// GOP ÷ sve noći (ista formula kao `MoneyOutcome::goppar_cents`).
    pub goppar_cents: Option<i64>,
    pub labor_cents: i64,
    /// Trošak rada ÷ ukupni prihod.
    pub labor_share_bp: Option<i64>,
    /// Izravni trošak soba ÷ prodane noći.
    pub cpor_cents: Option<i64>,
    /// Popunjenost pri kojoj bi GOP bio nula (vidi `break_even_bp`); može biti i iznad 10 000.
    pub break_even_occupancy_bp: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KpiError {
    Calc(CalcError),
    Catalog(CatalogError),
}

impl From<CalcError> for KpiError {
    fn from(e: CalcError) -> Self {
        KpiError::Calc(e)
    }
}
impl From<CatalogError> for KpiError {
    fn from(e: CatalogError) -> Self {
        KpiError::Catalog(e)
    }
}

/// `a / b` zaokruženo kao u obračunu; `None` kad je djelitelj nula (ili negativan).
fn ratio(a: i128, b: i128) -> Calc<Option<i64>> {
    if b <= 0 {
        return Ok(None);
    }
    Ok(Some(c::narrow(c::div_round(a, b)?)?))
}

/// Točka pokrića: popunjenost (bp) pri kojoj je GOP = 0, uz **isti ADR** (i isti udio ostalog prihoda) i
/// **iste troškove koji ne ovise o prodanim noćima**.
///
/// ```text
/// izravno   = CPOR × prodano                 (jedini trošak koji raste s prodanim noćima)
/// doprinos  = ukupni prihod − izravno        (što donosi prodaja, prije fiksnih troškova)
/// fiksno    = troškovi − izravno             (plaće, održavanje, marketing, fiksni troškovi…)
/// GOP(x)    = x × doprinos / prodano − fiksno, za x prodanih noći
/// GOP(x)=0  ⇒ x* = fiksno × prodano / doprinos
/// pokriće   = x* / sve noći = fiksno × prodano × 10 000 / (doprinos × sve noći)   [bp]
/// ```
///
/// `None` kad nema prodaje, soba ili kad prodaja ne donosi pozitivan doprinos (nikad se ne pokriva);
/// 0 kad fiksni troškovi nisu pozitivni (GOP je pozitivan i bez prodaje).
fn break_even_bp(i: &KpiInput, total: i128, room_nights: i128) -> Calc<Option<i64>> {
    let sold = i128::from(i.sold);
    if sold == 0 || room_nights == 0 {
        return Ok(None);
    }
    let direct = c::mul(i128::from(i.cpor_cents), sold)?;
    let contribution = c::sub(total, direct)?;
    if contribution <= 0 {
        return Ok(None);
    }
    let fixed = c::sub(i128::from(i.costs_cents), direct)?;
    if fixed <= 0 {
        return Ok(Some(0));
    }
    ratio(
        c::mul(c::mul(fixed, sold)?, 10_000)?,
        c::mul(contribution, room_nights)?,
    )
}

pub fn kpi(i: &KpiInput) -> Calc<MonthKpi> {
    let room_nights = c::mul(i128::from(i.rooms), i128::from(i.days))?;
    let sold = i128::from(i.sold);
    let lost = c::sub(room_nights, i128::from(i.available))?.max(0);
    let rooms_revenue = i128::from(i.rooms_revenue_cents);
    let total = c::add(rooms_revenue, i128::from(i.other_revenue_cents))?;
    let gop = i128::from(i.gop_cents);
    let labor = i128::from(i.labor_cents);
    Ok(MonthKpi {
        room_nights: c::narrow(room_nights)?,
        sold_nights: i.sold,
        lost_nights: c::narrow(lost)?,
        occupancy_bp: ratio(c::mul(sold, 10_000)?, room_nights)?,
        adr_cents: ratio(rooms_revenue, sold)?,
        revpar_cents: ratio(rooms_revenue, room_nights)?,
        rooms_revenue_cents: i.rooms_revenue_cents,
        total_revenue_cents: c::narrow(total)?,
        trevpar_cents: ratio(total, room_nights)?,
        gop_cents: i.gop_cents,
        gop_margin_bp: ratio(c::mul(gop, 10_000)?, total)?,
        goppar_cents: ratio(gop, room_nights)?,
        labor_cents: i.labor_cents,
        labor_share_bp: ratio(c::mul(labor, 10_000)?, total)?,
        cpor_cents: ratio(c::mul(i128::from(i.cpor_cents), sold)?, sold)?,
        break_even_occupancy_bp: break_even_bp(i, total, room_nights)?,
    })
}

/// Pokazatelji obračunatog mjeseca iz izvještaja jezgre.
pub fn month_kpi(r: &MonthReport, cat: &Catalog) -> Result<MonthKpi, KpiError> {
    Ok(kpi(&KpiInput::from_report(r, cat)?)?)
}
