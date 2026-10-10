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

/// Iste brojke kao `KpiInput::from_report`, samo iz oblika spremljenog u dnevniku.
pub fn report_kpi(r: &MonthReportDto, cat: &Catalog) -> Result<MonthKpi, GameError> {
    let category = Category::from_code(&r.before.category)
        .ok_or_else(|| corrupt(format!("kategorija {}", r.before.category)))?;
    let spec = cat
        .category(category)
        .map_err(|e| corrupt(format!("{e:?}")))?;
    let input = KpiInput {
        rooms: r.before.rooms,
        days: Month::from(r.month).days(),
        available: r.market.available,
        sold: r.market.sold,
        rooms_revenue_cents: r.money.rooms_revenue_cents,
        other_revenue_cents: r.money.other_revenue_cents,
        labor_cents: r.money.labor_cents,
        costs_cents: r.money.costs_cents,
        gop_cents: r.money.gop_cents,
        cpor_cents: spec.cpor_cents,
    };
    kpi(&input).map_err(|e| corrupt(format!("pokazatelji: {e:?}")))
}

impl Game {
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
                kpi: report_kpi(&res.report, &self.content.catalog)?,
            });
        }
        Ok(out)
    }
}
