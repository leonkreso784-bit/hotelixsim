//! Popunjavanje primjerka: imena i opisi uloga, brojevna mjesta. Svaki primjerak ima svoj tok kocke (ključ je
//! šifra predloška), pa izbor drugih predložaka u krugu ne mijenja njegove brojeve; imena se u krugu ne ponavljaju
//! dok ih popis ima dovoljno.

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

/// Primjerak predloška `t` za krug `p`, s odjelom `department`; `used` su imena već dana u ovom krugu.
pub(super) fn fill(
    t: &SituationSpec,
    department: Option<DeptCode>,
    p: &Period,
    seed: u64,
    slot: u8,
    names: &NamePool,
    used: &mut BTreeSet<String>,
) -> Result<DrawnSituation, ResolveError> {
    let mut rng = stream(seed, Purpose::Situations, p.index, slot, label(&t.code));
    let mut roles = BTreeMap::new();
    for (key, role) in &t.roles {
        let all = names.list(role.source, role.gender);
        let free: Vec<&String> = all.iter().filter(|n| !used.contains(*n)).collect();
        let pool: Vec<&String> = if free.is_empty() {
            all.iter().collect()
        } else {
            free
        };
        let i = index(&mut rng, pool.len())
            .map_err(|_| CatalogError::Missing(format!("names za ulogu {key} ({})", t.code)))?;
        let name = pool.get(i).map(|n| (*n).clone()).unwrap_or_default();
        used.insert(name.clone());
        let role_type = if role.types.is_empty() {
            None
        } else {
            role.types.get(index(&mut rng, role.types.len())?).cloned()
        };
        roles.insert(key.clone(), RoleFill { name, role_type });
    }
    let mut numbers = BTreeMap::new();
    for (key, param) in &t.numbers {
        numbers.insert(key.clone(), number(&mut rng, param)?);
    }
    Ok(DrawnSituation {
        code: format!("{}.{}", t.code, p.code()),
        template: t.code.clone(),
        period: *p,
        kind: t.kind,
        weight: t.weight,
        scope: t.scope,
        major: t.major,
        department,
        roles,
        numbers,
    })
}
