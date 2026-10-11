//! Sloj 0 — pokazatelji obračunatih mjeseci (MODEL_V2 §5). Ne spremaju se: računaju se pri čitanju iz izvještaja
//! koji su već u dnevniku (`HotelResolution::report`), formulama jezgre (`sim_hotel::kpi`).

use sim_hotel::{Catalog, Category, KpiInput, Month, MonthKpi, kpi};

use crate::dto::MonthReportDto;
use crate::error::{GameError, corrupt};
use crate::game::Game;
use crate::ids::HotelId;
use crate::round::RoundKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KpiMonth {
    pub round: RoundKey,
    pub kpi: MonthKpi,
}

/// Iste brojke kao `KpiInput::from_report`, samo iz oblika spremljenog u dnevniku (mjesečni krug).
pub fn report_kpi(r: &MonthReportDto, cat: &Catalog) -> Result<MonthKpi, GameError> {
    round_kpi(r, Month::from(r.month).days(), cat)
}

/// Pokazatelji kruga od `days` dana (tjedan ili mjesec; `RoundInfo::period.days`), s izravnim troškom po noći
/// iz v1 kataloga (v1 igra). Igra koja može biti v2 zove `Game::round_kpi_of`.
pub fn round_kpi(r: &MonthReportDto, days: u8, cat: &Catalog) -> Result<MonthKpi, GameError> {
    let category = report_category(r)?;
    let spec = cat
        .category(category)
        .map_err(|e| corrupt(format!("{e:?}")))?;
    kpi_with_cpor(r, days, spec.cpor_cents)
}

fn report_category(r: &MonthReportDto) -> Result<Category, GameError> {
    Category::from_code(&r.before.category)
        .ok_or_else(|| corrupt(format!("kategorija {}", r.before.category)))
}

fn kpi_with_cpor(r: &MonthReportDto, days: u8, cpor_cents: i64) -> Result<MonthKpi, GameError> {
    let input = KpiInput {
        rooms: r.before.rooms,
        days,
        available: r.market.available,
        sold: r.market.sold,
        rooms_revenue_cents: r.money.rooms_revenue_cents,
        other_revenue_cents: r.money.other_revenue_cents,
        labor_cents: r.money.labor_cents,
        costs_cents: r.money.costs_cents,
        gop_cents: r.money.gop_cents,
        cpor_cents,
    };
    kpi(&input).map_err(|e| corrupt(format!("pokazatelji: {e:?}")))
}

impl Game {
    /// Pokazatelji kruga igre: dani razdoblja iz kalendara kruga, izravni trošak po noći iz modela igre
    /// (v1 kategorija, v2 odjel Smještaj; `Model::rooms_direct_cost`).
    pub(crate) fn round_kpi_of(
        &self,
        r: &MonthReportDto,
        round: RoundKey,
    ) -> Result<MonthKpi, GameError> {
        let days = self.round_info(round)?.period.days;
        let cpor = self
            .model()
            .rooms_direct_cost(&self.content, report_category(r)?)?;
        kpi_with_cpor(r, days, cpor)
    }

    /// Pokazatelji svakog obračunatog mjeseca hotela, redom sezone; neobračunati mjeseci se preskaču.
    pub fn hotel_kpi(&self, id: HotelId) -> Result<Vec<KpiMonth>, GameError> {
        self.hotel(id)?;
        let mut out = Vec::new();
        for m in &self.rounds {
            let Some(rr) = self.resolved.get(m) else {
                continue;
            };
            let res = rr
                .hotels
                .iter()
                .find(|x| x.hotel_id == id)
                .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
            out.push(KpiMonth {
                round: rr.month,
                kpi: self.round_kpi_of(&res.report, rr.month)?,
            });
        }
        Ok(out)
    }
}
