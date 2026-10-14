//! Obnova v2 (obnova-v2-spec §2–§3): knjiga imovine jednog dijela (odjel ili zgrada), krivulja učinka, raspored
//! plaćanja po krugovima mjeseca i amortizacija. Čiste funkcije koje zovu obračun kruga, plan i (kasnije) prikaz
//! cijena odluka, da sve brojke dolaze s jednog mjesta. Samo cijeli brojevi kroz `engine::calc`.

use engine::calc::{self as c, Calc};
use sim_hotel::Month;

use crate::calendar::{Period, is_last_round_of_month, prorate_part, rounds_left_in_month};

/// Krivulja i granice obnove jednog dijela; sve nule = obnova nije ponuđena.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenewalSpec {
    /// G: najveći učinak u jednom mjesecu (5000 = 50 bodova), nikad se ne dosegne.
    pub gain_pts: i32,
    /// K: iznos koji daje pola G.
    pub half_cents: i64,
    /// Najviše u jednom mjesecu; 0 = obnova nije ponuđena.
    pub max_monthly_cents: i64,
    /// Korak iznosa.
    pub step_cents: i64,
    /// Vijek za amortizaciju u mjesecima; 0 = bez amortizacije.
    pub life_months: u16,
}

impl RenewalSpec {
    /// Obnova je ponuđena (najveći mjesečni iznos veći od 0).
    pub fn offered(&self) -> bool {
        self.max_monthly_cents > 0
    }
}

/// Obnova odlučena u mjesecu `month`; zaključana dok traje taj mjesec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Renewal {
    /// `start_month()` kruga odluke.
    pub month: Month,
    /// A: odabrani iznos za mjesec.
    pub total_cents: i64,
    /// `⌊A / n0⌋`, n0 = krugova mjeseca od kruga odluke (uključivo); zadnji krug nosi ostatak.
    pub installment_cents: i64,
    /// Plaćeno do sada (nakon obračuna kruga).
    pub paid_cents: i64,
    /// E: učinak cijelog iznosa (F3), fiksan pri odluci.
    pub gain_pts: i32,
}

/// Knjiga imovine jednog dijela (odjel ili zgrada).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetBook {
    pub renewal: Option<Renewal>,
    /// Σ obnova iz završenih mjeseci = osnovica amortizacije.
    pub capitalized_cents: i64,
}

impl AssetBook {
    /// Obnova odlučena u mjesecu `m` (zaključana do kraja mjeseca).
    pub fn locked_in(&self, m: Month) -> Option<&Renewal> {
        self.renewal.as_ref().filter(|r| r.month == m)
    }

    /// Uloženo do sada: kapitalizirano + plaćeno u tekućem mjesecu.
    pub fn invested_cents(&self) -> i64 {
        let paid = self.renewal.map_or(0, |r| r.paid_cents);
        self.capitalized_cents.saturating_add(paid)
    }

    /// Obnova iz ranijeg mjeseca prelazi u kapitalizirano; ostaje samo obnova mjeseca `m`.
    pub fn roll(&mut self, m: Month) -> Calc<()> {
        if let Some(r) = self.renewal
            && r.month != m
        {
            self.capitalized_cents = c::narrow(c::add(
                i128::from(self.capitalized_cents),
                i128::from(r.paid_cents),
            )?)?;
            self.renewal = None;
        }
        Ok(())
    }
}

/// F3: `E(A) = div_round(G·A, A + K)`; `A ≤ 0` daje 0. Vrijedi `E(K) = G/2` i `E(A) < G`.
pub fn gain_pts(spec: &RenewalSpec, amount_cents: i64) -> Calc<i32> {
    if amount_cents <= 0 {
        return Ok(0);
    }
    let a = i128::from(amount_cents);
    c::narrow(c::div_round(
        c::mul(i128::from(spec.gain_pts), a)?,
        c::add(a, i128::from(spec.half_cents))?,
    )?)
}

/// Raspored iznosa `A` na `n0` krugova mjeseca (F4): rata `⌊A / n0⌋` u svakom krugu osim zadnjeg, a zadnji nosi
/// ostatak `A − rata·(n0 − 1)`. Isto pravilo plaća obračun (`renew_round`) i pokazuje izbor iznosa u planu.
pub fn installments(amount_cents: i64, n0: u8) -> Calc<(i64, i64)> {
    let a = i128::from(amount_cents);
    let n = i128::from(n0.max(1));
    let installment = c::div_floor(a, n)?;
    let last = c::sub(a, c::mul(installment, c::sub(n, 1)?)?)?;
    Ok((c::narrow(installment)?, c::narrow(last)?))
}

/// Bodovi koje je obnova donijela do sada (prije granice 100): `div_round(E·plaćeno, A)`; zbroj Δ kruga do sada.
pub fn gained_pts(r: &Renewal) -> Calc<i32> {
    if r.total_cents <= 0 {
        return Ok(0);
    }
    c::narrow(c::div_round(
        c::mul(i128::from(r.gain_pts), i128::from(r.paid_cents))?,
        i128::from(r.total_cents),
    )?)
}

/// Obnova jednog dijela u krugu: plaćeno u krugu i bodovi koje krug donosi (prije granice 100).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoundRenewal {
    pub cents: i64,
    pub delta_pts: i32,
}

/// F4–F5 za jedan dio u krugu `p` s planiranim iznosom `amount` (validiran planom): knjiga se zarola u mjesec
/// kruga, nova obnova nastaje ako mjesec još nema obnovu i iznos je veći od 0, pa se plaća rata kruga (zadnji krug
/// mjeseca plaća ostatak). Bodovi kruga su `div_round(E·plaćeno_poslije, A) − div_round(E·plaćeno_prije, A)`, pa je
/// zbroj u mjesecu točno E.
pub fn renew_round(
    book: &mut AssetBook,
    spec: &RenewalSpec,
    amount_cents: i64,
    p: &Period,
) -> Calc<RoundRenewal> {
    let m = p.start_month();
    book.roll(m)?;
    if book.locked_in(m).is_none() && amount_cents > 0 {
        let (installment_cents, _) = installments(amount_cents, rounds_left_in_month(p))?;
        book.renewal = Some(Renewal {
            month: m,
            total_cents: amount_cents,
            installment_cents,
            paid_cents: 0,
            gain_pts: gain_pts(spec, amount_cents)?,
        });
    }
    let Some(r) = book.renewal.as_mut().filter(|r| r.month == m) else {
        return Ok(RoundRenewal::default());
    };
    let before = i128::from(r.paid_cents);
    let pay = if is_last_round_of_month(p) {
        c::sub(i128::from(r.total_cents), before)?
    } else {
        i128::from(r.installment_cents)
    };
    let after = c::add(before, pay)?;
    r.paid_cents = c::narrow(after)?;
    let (e, a) = (i128::from(r.gain_pts), i128::from(r.total_cents));
    let delta = c::sub(
        c::div_round(c::mul(e, after)?, a)?,
        c::div_round(c::mul(e, before)?, a)?,
    )?;
    Ok(RoundRenewal {
        cents: c::narrow(pay)?,
        delta_pts: c::narrow(delta)?,
    })
}

/// Mjesečna amortizacija osnovice: `div_round(osnovica, vijek)`; vijek 0 = bez amortizacije.
pub fn monthly_depreciation(base_cents: i64, life_months: u16) -> Calc<i64> {
    if life_months == 0 {
        return Ok(0);
    }
    c::narrow(c::div_round(
        i128::from(base_cents),
        i128::from(life_months),
    )?)
}

/// F9 (O13): amortizacija kruga nakon plaćanja. Za svaki dio kruga u mjesecu `m` osnovica je kapitalizirano plus
/// plaćeno u obnovi ranijeg mjeseca (imovina je u upotrebi od idućeg mjeseca), a iznos je razmjerni dio mjesečne
/// amortizacije (`prorate_part`), pa je zbroj po mjesecu u tjednoj i mjesečnoj igri isti do centa.
pub fn depreciation(book: &AssetBook, life_months: u16, p: &Period) -> Calc<i64> {
    if life_months == 0 {
        return Ok(0);
    }
    let mut total: i128 = 0;
    for part in p.parts() {
        let earlier = book
            .renewal
            .filter(|r| r.month < part.month)
            .map_or(0, |r| r.paid_cents);
        let base = c::narrow(c::add(
            i128::from(book.capitalized_cents),
            i128::from(earlier),
        )?)?;
        let monthly = monthly_depreciation(base, life_months)?;
        total = c::add(total, i128::from(prorate_part(monthly, &part)?))?;
    }
    c::narrow(total)
}

/// Najmanji iznos u koraku (`k·korak`, k = 1, 2, …) čiji učinak doseže `need` bodova, inače najveći mjesečni
/// iznos (roboti, §5.1). Obnova koja nije ponuđena daje 0.
pub fn smallest_amount_for(spec: &RenewalSpec, need_pts: i32) -> Calc<i64> {
    if !spec.offered() || spec.step_cents <= 0 {
        return Ok(0);
    }
    let mut amount = spec.step_cents;
    while amount <= spec.max_monthly_cents {
        if gain_pts(spec, amount)? >= need_pts {
            return Ok(amount);
        }
        amount = c::narrow(c::add(i128::from(amount), i128::from(spec.step_cents))?)?;
    }
    Ok(spec.max_monthly_cents)
}
