//! Primjerak predloška kao situacija s ponuđenim odgovorima (J-3): brojevna mjesta iz primjerka, iznosi
//! (`price_pct`, `wage_pct`) iz kataloga za kategoriju hotela i mjesec kruga. Tekstove s popunjenim mjestima
//! slaže igra; ovdje je samo ono što treba obračunu i robotima.

use sim_hotel::{Catalog, Category, Mechanic, Month, OfferedAnswer, SituationTemplate};
use sim_hotel_v2::situations::DrawnSituation;

use crate::ContentError;
use crate::templates::{Param, Quantity, SituationTemplates, Template, TemplateMechanic};

/// Što treba za brojeve jednog primjerka.
struct Amounts<'a> {
    t: &'a Template,
    d: &'a DrawnSituation,
    /// Cijena noćenja mjeseca kruga za kategoriju hotela (centi).
    price_cents: i64,
    /// Mjesečna plaća jednog zaposlenog za kategoriju hotela (centi).
    wage_cents: i64,
}

fn invalid<T>(d: &DrawnSituation, msg: String) -> Result<T, ContentError> {
    Err(ContentError::Invalid(format!(
        "primjerak {}: {msg}",
        d.code
    )))
}

/// `postotak × osnovica / 100`, zaokruženo na cent (polovica prema gore).
fn pct_of(pct: u16, base_cents: i64) -> Option<i64> {
    let v = i128::from(pct).checked_mul(i128::from(base_cents))?;
    i64::try_from(v.checked_add(50)?.checked_div(100)?).ok()
}

impl Amounts<'_> {
    /// Broj iz mehanike: zadan ili brojevno mjesto (`range`, `pick`) kako ga je primjerak izvukao.
    fn number(&self, q: &Quantity) -> Result<i64, ContentError> {
        match q {
            Quantity::Fixed(n) => Ok(*n),
            Quantity::Param(name) => match self.d.numbers.get(name) {
                Some(n) => Ok(*n),
                None => invalid(self.d, format!("mjesto '{name}' nema broj")),
            },
        }
    }

    fn small(&self, q: &Quantity) -> Result<u16, ContentError> {
        let n = self.number(q)?;
        u16::try_from(n).or_else(|_| invalid(self.d, format!("broj {n} izvan 0..=65535")))
    }

    /// Novčani iznos: zadan (`0`) ili postotak cijene noćenja / plaće.
    fn money(&self, q: &Quantity) -> Result<i64, ContentError> {
        let name = match q {
            Quantity::Fixed(n) => return Ok(*n),
            Quantity::Param(name) => name,
        };
        let v = match self.t.params.get(name) {
            Some(Param::PricePct(p)) => pct_of(*p, self.price_cents),
            Some(Param::WagePct(w)) => pct_of(*w, self.wage_cents),
            _ => return invalid(self.d, format!("mjesto '{name}' nije iznos")),
        };
        v.map_or_else(|| invalid(self.d, format!("iznos '{name}' prevelik")), Ok)
    }

    fn mechanics(&self, list: &[TemplateMechanic]) -> Result<Vec<Mechanic>, ContentError> {
        list.iter()
            .map(|m| {
                Ok(match m {
                    TemplateMechanic::ServiceRecovery { compensation } => {
                        Mechanic::ServiceRecovery {
                            compensation_cents: self.money(compensation)?,
                        }
                    }
                    TemplateMechanic::EquipmentRepair { urgent } => {
                        Mechanic::EquipmentRepair { urgent: *urgent }
                    }
                    TemplateMechanic::StaffRetention {
                        raise_one_pct,
                        bonus,
                    } => Mechanic::StaffRetention {
                        raise_one_pct: self.small(raise_one_pct)?,
                        bonus_cents: self.money(bonus)?,
                    },
                    TemplateMechanic::GroupBooking {
                        accept,
                        rooms,
                        nights,
                        discount_bp,
                        month,
                    } => Mechanic::GroupBooking {
                        accept: *accept,
                        rooms: self.small(rooms)?,
                        nights: self.small(nights)?,
                        discount_bp: self.small(discount_bp)?,
                        month: match Month::from_code(month) {
                            Some(m) => m,
                            None => return invalid(self.d, format!("nepoznat mjesec '{month}'")),
                        },
                    },
                    TemplateMechanic::CrisisCommunication { backup } => {
                        Mechanic::CrisisCommunication {
                            backup_cents: self.money(backup)?,
                        }
                    }
                })
            })
            .collect()
    }
}

impl SituationTemplates {
    /// Ima li situacija oznaku „samo nagrada" (`Template::no_penalty`): `code` je šifra predloška ili primjerka
    /// (`P-POHVALA.t03`, predložak je dio prije prve točke); nepoznata šifra nema.
    pub fn no_penalty(&self, code: &str) -> bool {
        let template = code.split('.').next().unwrap_or(code);
        self.templates
            .iter()
            .any(|t| t.code == template && t.no_penalty)
    }

    /// Primjerak `d` kao situacija s ponuđenim odgovorima za hotel kategorije `category`: šifra primjerka,
    /// mjesec obračuna kruga (`Period::situation_month`), kvaliteta odgovora iz rubrike, iznosi iz kataloga.
    pub fn instance(
        &self,
        d: &DrawnSituation,
        base: &Catalog,
        category: Category,
    ) -> Result<SituationTemplate, ContentError> {
        let Some(t) = self.templates.iter().find(|t| t.code == d.template) else {
            return invalid(d, format!("nepoznat predložak '{}'", d.template));
        };
        let month = d.period.situation_month();
        let spec = base
            .category(category)
            .map_err(|e| ContentError::Invalid(format!("primjerak {}: {e:?}", d.code)))?;
        let a = Amounts {
            t,
            d,
            price_cents: spec.price_ref_cents.get(month),
            wage_cents: spec.wage_cents,
        };
        let answers = t
            .answers
            .iter()
            .map(|x| {
                Ok(OfferedAnswer {
                    label: x.label.clone(),
                    mechanics: a.mechanics(&x.mechanics)?,
                    quality: x.quality(),
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?;
        Ok(SituationTemplate {
            code: d.code.clone(),
            month,
            title: t.title.clone(),
            answers,
            unanswered: a.mechanics(&t.unanswered)?,
        })
    }
}
