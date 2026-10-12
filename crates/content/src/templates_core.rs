//! Pretvorba provjerenih predložaka sadržaja u jezgreni oblik za izvlačenje (`sim_hotel_v2::situations`).
//! Jezgra dobiva samo pravila pojave, uloge i brojevna mjesta; tekstovi i odgovori ostaju ovdje.

use sim_hotel::Month;
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations as core;

use crate::templates::{
    Bounds, Conditions, Gender, Kind, MajorSide, NameList, Param, RoleSource, Scope,
    SituationTemplates, Template, TemplateMechanic,
};

impl SituationTemplates {
    /// Jezgreni oblik (predlošci su već provjereni, pa pretvorba ne može pasti).
    pub fn to_core(&self) -> core::TemplateSet {
        core::TemplateSet {
            major: core::MajorRule {
                high_avg_min: self.major.high_avg_min,
                low_avg_max: self.major.low_avg_max,
                max_per_month: self.major.max_per_month,
            },
            templates: self.templates.iter().map(template).collect(),
            group_limit: self.groups_per_season,
            groups_per_month: self.groups_per_month,
            names: core::NamePool {
                guest: names(&self.names.guest),
                staff: names(&self.names.staff),
                partner: names(&self.names.partner),
            },
        }
    }
}

/// Skupina odjela iz sadržaja (`*staffed`, `*guest_facing`) ili jedan odjel; `hotel` je cijeli hotel.
pub(crate) fn dept_target(code: &str) -> core::DeptTarget {
    match code {
        "hotel" => core::DeptTarget::Hotel,
        "*staffed" => core::DeptTarget::Staffed,
        "*guest_facing" => core::DeptTarget::GuestFacing,
        other => core::DeptTarget::One(DeptCode(other.to_string())),
    }
}

fn template(t: &Template) -> core::SituationSpec {
    core::SituationSpec {
        code: t.code.clone(),
        kind: kind(t.kind),
        department: dept_target(&t.department),
        weight: t.weight,
        scope: match t.scope {
            Scope::Hotel => core::Scope::Hotel,
            Scope::All => core::Scope::All,
        },
        major: t.major.map(|m| match m {
            MajorSide::High => core::MajorSide::High,
            MajorSide::Low => core::MajorSide::Low,
        }),
        when: conditions(&t.when),
        chance: core::Chance {
            base: t.chance.base,
            boost: t
                .chance
                .boost
                .iter()
                .map(|b| core::Boost {
                    when: conditions(&b.when),
                    times: b.times,
                })
                .collect(),
        },
        repeat: core::Repeat {
            max_per_season: t.repeat.max_per_season,
            gap_rounds: t.repeat.gap_rounds,
        },
        roles: t
            .roles
            .iter()
            .map(|(k, r)| {
                let role = core::RoleSpec {
                    source: match r.from {
                        RoleSource::Guest => core::RoleSource::Guest,
                        RoleSource::Staff => core::RoleSource::Staff,
                        RoleSource::Partner => core::RoleSource::Partner,
                    },
                    gender: match r.gender {
                        Gender::F => core::Gender::F,
                        Gender::M => core::Gender::M,
                    },
                    types: r.types.clone(),
                };
                (k.clone(), role)
            })
            .collect(),
        numbers: t
            .params
            .iter()
            .filter_map(|(k, p)| {
                let n = match p {
                    Param::Range([lo, hi]) => core::NumberParam::Range(*lo, *hi),
                    Param::Pick(v) => core::NumberParam::Pick(v.clone()),
                    Param::PricePct(_) | Param::WagePct(_) | Param::ByDepartment(_) => {
                        return None;
                    }
                };
                Some((k.clone(), n))
            })
            .collect(),
        group_month: group_month(t),
    }
}

/// Mjesec dolaska grupe iz prvog odgovora koji je prihvaća (provjera jamči poznat mjesec).
fn group_month(t: &Template) -> Option<Month> {
    t.answers
        .iter()
        .flat_map(|a| &a.mechanics)
        .find_map(|m| match m {
            TemplateMechanic::GroupBooking {
                accept: true,
                month,
                ..
            } => Month::from_code(month),
            _ => None,
        })
}

fn kind(k: Kind) -> core::Kind {
    match k {
        Kind::Complaint => core::Kind::Complaint,
        Kind::Staff => core::Kind::Staff,
        Kind::Opportunity => core::Kind::Opportunity,
        Kind::Breakdown => core::Kind::Breakdown,
        Kind::Crisis => core::Kind::Crisis,
        Kind::Request => core::Kind::Request,
    }
}

fn bounds(b: &Option<Bounds>) -> Option<core::Bounds> {
    b.map(|b| core::Bounds {
        at_least: b.at_least,
        below: b.below,
    })
}

fn conditions(c: &Conditions) -> core::Conditions {
    core::Conditions {
        months: c
            .months
            .iter()
            .filter_map(|m| Month::from_code(m))
            .collect(),
        occupancy_bp: bounds(&c.occupancy_bp),
        morale_pts: bounds(&c.morale_pts),
        cash_cents: bounds(&c.cash_cents),
        group_accepted: c.group_accepted,
        dept_condition_pts: bounds(&c.dept_condition_pts),
        dept_satisfaction_pts: bounds(&c.dept_satisfaction_pts),
        dept_load_hundredths: bounds(&c.dept_load_hundredths),
        dept_staff: bounds(&c.dept_staff),
        dept_departures: bounds(&c.dept_departures),
    }
}

fn names(l: &NameList) -> core::NameList {
    core::NameList {
        f: l.f.clone(),
        m: l.m.clone(),
    }
}
