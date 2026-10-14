//! Primjerak predloška kao situacija s ponuđenim odgovorima (J-3): brojevna mjesta iz primjerka, iznosi
//! (`price_pct`, `wage_pct`) iz kataloga za kategoriju hotela i mjesec kruga. Tekstovi (naslov, oznake odgovora i
//! sve što ide AI-ju: rubrika, lik) dolaze s popunjenim mjestima; nijedan ne smije zadržati `{…}` (G-2).

use sim_hotel::{Catalog, Category, Mechanic, Month, OfferedAnswer, SituationTemplate};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::DrawnSituation;

use crate::templates::{
    Gender, Param, Quantity, Role, RoleSource, SituationTemplates, Template, TemplateMechanic,
};
use crate::{Character, ContentError, SituationAi};

/// Primjerak s popunjenim tekstovima: situacija za obračun i podaci za AI sloj (lik, rubrika, kriteriji).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    pub situation: SituationTemplate,
    /// Opis situacije s popunjenim mjestima.
    pub description: String,
    pub ai: SituationAi,
}

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

/// Iznos u eurima za tekst: „90 €" ili „90,50 €".
fn euros(cents: i64) -> String {
    let (e, c) = (cents / 100, (cents % 100).abs());
    if c == 0 {
        format!("{e} €")
    } else {
        format!("{e},{c:02} €")
    }
}

/// Rečenica „tko je lik" za studenta: opis uloge primjerka, bez njega naziv uloge iz predloška (`title`), a bez
/// oba vrsta uloge (gost, zaposlenik odjela, vanjski suradnik) u rodu uloge.
fn role_sentence(role: &Role, role_type: Option<&str>, dept: Option<&str>) -> String {
    if let Some(ty) = role_type {
        return ty.to_owned();
    }
    if let Some(title) = &role.title {
        return title.clone();
    }
    let f = role.gender == Gender::F;
    match role.from {
        RoleSource::Guest if f => "gošća hotela".to_owned(),
        RoleSource::Guest => "gost hotela".to_owned(),
        RoleSource::Staff => format!(
            "{} odjela {}",
            if f { "zaposlenica" } else { "zaposlenik" },
            dept.unwrap_or("hotela")
        ),
        RoleSource::Partner if f => "vanjska suradnica hotela".to_owned(),
        RoleSource::Partner => "vanjski suradnik hotela".to_owned(),
    }
}

impl Amounts<'_> {
    /// Tekst s popunjenim mjestima: `{uloga.ime}`, `{uloga.tip}`, `{odjel}`, brojevno mjesto, iznos i tekst po odjelu.
    fn text(&self, dept: Option<&str>, src: &str) -> Result<String, ContentError> {
        let mut out = String::with_capacity(src.len());
        let mut rest = src;
        while let Some(start) = rest.find('{') {
            out.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            let Some(end) = after.find('}') else {
                return invalid(self.d, format!("nezatvoreno mjesto u '{src}'"));
            };
            out.push_str(&self.place(dept, &after[..end])?);
            rest = &after[end + 1..];
        }
        out.push_str(rest);
        Ok(out)
    }

    fn place(&self, dept: Option<&str>, name: &str) -> Result<String, ContentError> {
        if name == "odjel" {
            return dept.map(str::to_owned).map_or_else(
                || invalid(self.d, "mjesto {odjel} bez odjela".to_owned()),
                Ok,
            );
        }
        if let Some((key, part)) = name.split_once('.') {
            let Some(fill) = self.d.roles.get(key) else {
                return invalid(self.d, format!("uloga '{key}' nije popunjena"));
            };
            return match part {
                "ime" => Ok(fill.name.clone()),
                "tip" => fill
                    .role_type
                    .clone()
                    .map_or_else(|| invalid(self.d, format!("uloga '{key}' nema tip")), Ok),
                _ => invalid(self.d, format!("nepoznato mjesto '{name}'")),
            };
        }
        match self.t.params.get(name) {
            Some(Param::Range(_) | Param::Pick(_)) => match self.d.numbers.get(name) {
                Some(n) => Ok(n.to_string()),
                None => invalid(self.d, format!("mjesto '{name}' nema broj")),
            },
            Some(Param::PricePct(_) | Param::WagePct(_)) => {
                self.money(&Quantity::Param(name.to_owned())).map(euros)
            }
            Some(Param::ByDepartment(m)) => self
                .d
                .department
                .as_ref()
                .and_then(|c| m.get(c.as_str()))
                .cloned()
                .map_or_else(
                    || invalid(self.d, format!("mjesto '{name}' nema teksta za odjel")),
                    Ok,
                ),
            None => invalid(self.d, format!("nepoznato mjesto '{name}'")),
        }
    }

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

    /// Brojevi primjerka: predložak, mjesec obračuna i iznosi kataloga za kategoriju hotela.
    fn amounts<'a>(
        &'a self,
        d: &'a DrawnSituation,
        base: &Catalog,
        category: Category,
    ) -> Result<(Amounts<'a>, Month), ContentError> {
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
        Ok((a, month))
    }

    /// Primjerak `d` kao situacija s ponuđenim odgovorima za hotel kategorije `category`: šifra primjerka,
    /// mjesec obračuna kruga (`Period::situation_month`), kvaliteta odgovora iz rubrike, iznosi iz kataloga.
    /// Samo ono što treba obračunu i robotima: naslov i oznake ostaju kakvi su u predlošku (popunjene daje
    /// `instance_with`).
    pub fn instance(
        &self,
        d: &DrawnSituation,
        base: &Catalog,
        category: Category,
    ) -> Result<SituationTemplate, ContentError> {
        let (a, month) = self.amounts(d, base, category)?;
        let answers =
            a.t.answers
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
            title: a.t.title.clone(),
            answers,
            unanswered: a.mechanics(&a.t.unanswered)?,
        })
    }

    /// Primjerak s popunjenim tekstovima (naslov, oznake odgovora) i podacima za AI sloj (lik, rubrika, kriteriji);
    /// `dept_name` daje ime odjela za mjesto `{odjel}`. Uloge i brojevi primjerka moraju biti popunjeni.
    pub fn instance_with(
        &self,
        d: &DrawnSituation,
        base: &Catalog,
        category: Category,
        dept_name: &dyn Fn(&DeptCode) -> String,
    ) -> Result<Instance, ContentError> {
        let (a, month) = self.amounts(d, base, category)?;
        let t = a.t;
        let dept = d.department.as_ref().map(dept_name);
        let dept = dept.as_deref();
        let answers = t
            .answers
            .iter()
            .map(|x| {
                Ok(OfferedAnswer {
                    label: a.text(dept, &x.label)?,
                    mechanics: a.mechanics(&x.mechanics)?,
                    quality: x.quality(),
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?;
        let rubric = t
            .rubric
            .iter()
            .map(|r| a.text(dept, r))
            .collect::<Result<Vec<_>, _>>()?;
        let character = match &t.character {
            None => None,
            Some(c) => {
                let (Some(role), Some(fill)) = (t.roles.get(&c.role), d.roles.get(&c.role)) else {
                    return invalid(d, format!("lik: uloga '{}' nije popunjena", c.role));
                };
                Some(Character {
                    name: fill.name.clone(),
                    role: role_sentence(role, fill.role_type.as_deref(), dept),
                    brief: a.text(dept, &c.brief)?,
                    opening: a.text(dept, &c.opening)?,
                    fallback_reply: a.text(dept, &c.fallback_reply)?,
                })
            }
        };
        let criteria = t
            .answers
            .iter()
            .map(|x| {
                let mut c = x.criteria.clone();
                c.sort_unstable();
                c
            })
            .collect();
        Ok(Instance {
            description: a.text(dept, &t.description)?,
            situation: SituationTemplate {
                code: d.code.clone(),
                month,
                title: a.text(dept, &t.title)?,
                answers,
                unanswered: a.mechanics(&t.unanswered)?,
            },
            ai: SituationAi {
                rubric,
                free_text: character.is_some(),
                character,
                criteria,
            },
        })
    }
}
