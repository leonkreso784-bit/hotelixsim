//! Popunjavanje primjerka: imena i opisi uloga, brojevna mjesta. Svaki primjerak ima svoj tok kocke (ključ je
//! šifra predloška, uz redni broj za ponovljeni primjerak u istom krugu), pa izbor drugih predložaka u krugu ne
//! mijenja njegove brojeve. Imena se u krugu ne ponavljaju dok ih popis ima dovoljno, a ponovljeni predložak
//! dobiva drugi lik: ime (i opis) koje nije imao nijedan raniji primjerak tog predloška u sezoni.

use std::collections::{BTreeMap, BTreeSet};

use engine::calc::{self as c, CalcError};
use engine::{Rng, label};
use sim_hotel::CatalogError;

use super::template::{NamePool, NumberParam, SituationSpec};
use super::{DrawnSituation, RoleFill};
use crate::calendar::Period;
use crate::department::DeptCode;
use crate::resolve::ResolveError;
use crate::streams::{Purpose, stream};

/// Jednoliko jedan od `len` (indeks).
fn index(rng: &mut Rng, len: usize) -> Result<usize, ResolveError> {
    let n = u64::try_from(len).map_err(|_| CalcError::Overflow)?;
    if n == 0 {
        return Err(CalcError::BadDivisor.into());
    }
    Ok(usize::try_from(rng.range(0, n)).map_err(|_| CalcError::Overflow)?)
}

fn number(rng: &mut Rng, p: &NumberParam) -> Result<i64, ResolveError> {
    match p {
        NumberParam::Range(lo, hi) => {
            let span = c::add(c::sub(i128::from(*hi), i128::from(*lo))?, 1)?;
            let span = u64::try_from(span).map_err(|_| CalcError::BadDivisor)?;
            if span == 0 {
                return Err(CalcError::BadDivisor.into());
            }
            let offset = rng.range(0, span);
            Ok(c::narrow(c::add(i128::from(*lo), i128::from(offset))?)?)
        }
        NumberParam::Pick(v) => {
            let i = index(rng, v.len())?;
            v.get(i).copied().ok_or_else(|| CalcError::Overflow.into())
        }
    }
}

/// Za koga i gdje se primjerak puni: krug, sjeme, utor i imena likova.
pub(super) struct FillCtx<'a> {
    pub p: &'a Period,
    pub seed: u64,
    pub slot: u8,
    pub names: &'a NamePool,
}

/// Mjesto primjerka: odjel, dan sezone i redni broj istog predloška u krugu.
pub(super) struct Place {
    pub department: Option<DeptCode>,
    pub day: u16,
    pub occurrence: u8,
}

/// Prvi neprazan popis po redu sklonosti.
fn first_nonempty(tiers: [Vec<&String>; 4]) -> Vec<&String> {
    tiers
        .into_iter()
        .find(|t| !t.is_empty())
        .unwrap_or_default()
}

/// Primjerak predloška `t`; `earlier` su raniji primjerci istog predloška u sezoni (drugi lik), `used` imena već
/// dana u ovom krugu.
pub(super) fn fill(
    t: &SituationSpec,
    at: Place,
    ctx: &FillCtx,
    earlier: &[&DrawnSituation],
    used: &mut BTreeSet<String>,
) -> Result<DrawnSituation, ResolveError> {
    let key = if at.occurrence <= 1 {
        label(&t.code)
    } else {
        label(&format!("{}#{}", t.code, at.occurrence))
    };
    let mut rng = stream(ctx.seed, Purpose::Situations, ctx.p.index, ctx.slot, key);
    let avoid: BTreeSet<&str> = earlier
        .iter()
        .flat_map(|e| e.roles.values().map(|r| r.name.as_str()))
        .collect();
    let mut roles = BTreeMap::new();
    for (key, role) in &t.roles {
        let all = ctx.names.list(role.source, role.gender);
        let only = |f: &dyn Fn(&String) -> bool| all.iter().filter(|n| f(n)).collect::<Vec<_>>();
        // Bez ranijih primjeraka predložak bira samo među imenima slobodnim u krugu (kao prije ponavljanja).
        let pool = first_nonempty([
            only(&|n| !used.contains(n) && !avoid.contains(n.as_str())),
            only(&|n| !avoid.contains(n.as_str())),
            only(&|n| !used.contains(n)),
            all.iter().collect(),
        ]);
        let i = index(&mut rng, pool.len())
            .map_err(|_| CatalogError::Missing(format!("names za ulogu {key} ({})", t.code)))?;
        let name = pool.get(i).map(|n| (*n).clone()).unwrap_or_default();
        used.insert(name.clone());
        let seen_types: BTreeSet<&str> = earlier
            .iter()
            .filter_map(|e| e.roles.get(key).and_then(|r| r.role_type.as_deref()))
            .collect();
        let fresh: Vec<&String> = role
            .types
            .iter()
            .filter(|ty| !seen_types.contains(ty.as_str()))
            .collect();
        let types: Vec<&String> = if fresh.is_empty() {
            role.types.iter().collect()
        } else {
            fresh
        };
        let role_type = if types.is_empty() {
            None
        } else {
            types
                .get(index(&mut rng, types.len())?)
                .map(|ty| (*ty).clone())
        };
        roles.insert(key.clone(), RoleFill { name, role_type });
    }
    let mut numbers = BTreeMap::new();
    for (key, param) in &t.numbers {
        numbers.insert(key.clone(), number(&mut rng, param)?);
    }
    let code = if at.occurrence <= 1 {
        format!("{}.{}", t.code, ctx.p.code())
    } else {
        format!("{}.{}.{}", t.code, ctx.p.code(), at.occurrence)
    };
    Ok(DrawnSituation {
        code,
        template: t.code.clone(),
        period: *ctx.p,
        day: at.day,
        occurrence: at.occurrence,
        slot: ctx.slot,
        kind: t.kind,
        weight: t.weight,
        scope: t.scope,
        major: t.major,
        department: at.department,
        roles,
        numbers,
    })
}
