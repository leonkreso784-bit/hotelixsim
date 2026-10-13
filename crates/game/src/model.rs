//! JEDINO mjesto grananja v1/v2 (Global Constraints: „Jedno mjesto grananja"). Ostatak paketa `game`, kao i
//! `app` i `api`, zove samo metode `Model`, `CoreState` i `CorePlan`; test `dispatch_lives_in_one_place` to čuva.
//! v1 grane zovu točno isti v1 kod kao prije sloja 1 (stara igra daje iste brojke i isti JSON).

use content::Content;
use engine::Rng;
use sim_hotel::{
    Category, IndexSet, MonthPlan, Profile, SituationOutcome, SituationTemplate, Strategy,
    month_indices, resolve_month,
};
use sim_hotel_v2::calendar::{avg_i64, month_period};
use sim_hotel_v2::department::PeriodAccounts;
use sim_hotel_v2::situations::{DrawView, DrawnSituation, TemplateSet};
use sim_hotel_v2::{
    Period, PeriodKind, PeriodPlan, PeriodReport, ResolveError, RoundHotel, collective_for_round,
    draw_for_round_with, period_indices, resolve_period, run_season_with,
};

use crate::command::{DeptLimitsView, PlanLimits, plan_limits};
use crate::costs::v2_costs;
use crate::dto::{MonthReportDto, PlanDto};
use crate::dto_v2::DetailDto;
use crate::error::{GameError, corrupt};
use crate::event::{HotelResolution, RoundDefDto};
use crate::round::RoundKey;

/// Kojim se modelom igra računa (iz snimka pravila u `GameCreated`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    V1,
    V2,
}

/// Stanje hotela na početku kruga, u obliku svog modela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreState {
    V1(sim_hotel::HotelState),
    V2(Box<sim_hotel_v2::HotelState>),
}

/// Plan kruga, u obliku svog modela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorePlan {
    V1(MonthPlan),
    V2(PeriodPlan),
}

/// Referentna tablica indeksa po kategoriji, u obliku svog modela.
#[derive(Clone, Debug)]
pub(crate) enum CoreRefs {
    V1(sim_hotel::RefTable),
    V2(sim_hotel_v2::RefTable),
}

/// Jedan krug igre: ključ i kalendar (za v1 uvijek cijeli mjesec).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundInfo {
    pub key: RoundKey,
    pub period: Period,
}

/// Obračun jednog hotela u krugu: v1 sažetak (uvijek), v2 dio (samo v2) i indeksi nakon kruga.
pub(crate) struct Resolved {
    pub report: MonthReportDto,
    pub detail: Option<DetailDto>,
    pub indices: IndexSet,
}

/// Ulaz izvlačenja situacija jednog hotela u krugu (`Model::draw_collective`, `Model::draw_situations`).
pub(crate) struct DrawInput<'a> {
    /// Stanje hotela na početku kruga.
    pub state: &'a CoreState,
    /// Cijeli obračun prošlog kruga; `None` u prvom krugu.
    pub previous: Option<&'a PeriodReport>,
    /// Pet pokazatelja na početku kruga.
    pub indices: &'a IndexSet,
    pub round: &'a RoundInfo,
    /// Primjerci ovog hotela iz prijašnjih krugova i kolektivni, svaki jednom.
    pub history: &'a [DrawnSituation],
    pub seed: u64,
    pub slot: u8,
}

/// v1 igra od sloja 1 dalje: lipanj i srpanj (kao prije).
const V1_MONTHS: [sim_hotel::Month; 2] = [sim_hotel::Month::Jun, sim_hotel::Month::Jul];

fn v2_catalog(c: &Content) -> Result<&sim_hotel_v2::Catalog, GameError> {
    c.v2.as_ref()
        .ok_or_else(|| corrupt("snimak v2 igre nema katalog v2"))
}

fn mismatch() -> GameError {
    corrupt("stanje, plan ili tablica nisu modela igre")
}

fn debug(e: impl std::fmt::Debug) -> GameError {
    corrupt(format!("{e:?}"))
}

impl Model {
    pub fn of(content: &Content, snapshot_model: content::Model) -> Model {
        match (snapshot_model, &content.v2) {
            (content::Model::V2, Some(_)) => Model::V2,
            // `Snapshot::build` odbija v2 snimak bez odjeljka v2, pa je ovo uvijek v1.
            (content::Model::V2, None) | (content::Model::V1, _) => Model::V1,
        }
    }

    /// Nova igra (app `start_demo`): snimak sadržaja modela iz postavki i zadana vrsta kruga (v1 mjesec, v2 tjedan).
    /// Jedino mjesto koje grana na `content::Model` izvan `Model::of`.
    pub fn new_game_snapshot<'a, T>(
        snapshot_model: content::Model,
        v1: &'a T,
        v2: &'a T,
    ) -> (&'a T, PeriodKind) {
        match snapshot_model {
            content::Model::V1 => (v1, PeriodKind::Month),
            content::Model::V2 => (v2, PeriodKind::Week),
        }
    }

    /// "v1" | "v2" — jedini put kojim app/api pretvaraju model u svoj kod (bez `match` na `Model` izvan ove datoteke).
    pub fn code(self) -> &'static str {
        match self {
            Model::V1 => "v1",
            Model::V2 => "v2",
        }
    }

    pub fn core_version(self) -> &'static str {
        match self {
            Model::V1 => sim_hotel::CORE_VERSION,
            Model::V2 => sim_hotel_v2::CORE_VERSION,
        }
    }

    /// Provjera verzije pri čitanju igre: v2 igra druge verzije jezgre v2 je zastarjela (🟨 P11); v1 se ovdje ne
    /// provjerava (samo u `resolve_job`, kao prije), pa je stara v1 igra nikad ne dobije.
    pub(crate) fn check_version(self, created: &str) -> Result<(), GameError> {
        match self {
            Model::V1 => Ok(()),
            Model::V2 if created == sim_hotel_v2::CORE_VERSION => Ok(()),
            Model::V2 => Err(GameError::Outdated {
                created: created.to_owned(),
                current: sim_hotel_v2::CORE_VERSION.to_owned(),
            }),
        }
    }

    /// Krugovi nove igre: kodovi za `GameCreated.months` i (samo v2) kalendar za `GameCreated.rounds`.
    /// v1 je uvijek lipanj–srpanj po mjesecima; v1 po tjednima nije dopuštena. v2 je cijela sezona (🟨 P6).
    pub(crate) fn new_rounds(
        self,
        kind: PeriodKind,
    ) -> Result<(Vec<RoundKey>, Vec<RoundDefDto>), GameError> {
        match (self, kind) {
            (Model::V1, PeriodKind::Month) => {
                Ok((V1_MONTHS.iter().map(|&m| m.into()).collect(), Vec::new()))
            }
            (Model::V1, PeriodKind::Week) => Err(GameError::InvalidSettings),
            (Model::V2, kind) => {
                let mut months = Vec::new();
                let mut defs = Vec::new();
                for p in sim_hotel_v2::season_periods(kind) {
                    let code = RoundKey::parse(&p.code())
                        .ok_or_else(|| corrupt(format!("krug {}", p.code())))?;
                    months.push(code);
                    defs.push(RoundDefDto {
                        code,
                        kind: p.kind.code().to_owned(),
                        index: p.index,
                        start_day: p.start_day,
                        days: p.days,
                    });
                }
                Ok((months, defs))
            }
        }
    }

    /// Krugovi igre iz `GameCreated`: v1 iz mjeseci (`month_period`), v2 iz spremljenog kalendara, čiji kodovi
    /// moraju biti točno `months` (inače je dnevnik pokvaren).
    pub(crate) fn rounds(
        self,
        months: &[RoundKey],
        defs: &[RoundDefDto],
    ) -> Result<Vec<RoundInfo>, GameError> {
        match self {
            Model::V1 => {
                if !defs.is_empty() {
                    return Err(corrupt("v1 igra s kalendarom krugova"));
                }
                months
                    .iter()
                    .map(|&key| {
                        let m = key
                            .require_month()
                            .map_err(|_| corrupt("v1 krug nije mjesec"))?;
                        Ok(RoundInfo {
                            key,
                            period: month_period(m),
                        })
                    })
                    .collect()
            }
            Model::V2 => {
                let codes: Vec<RoundKey> = defs.iter().map(|d| d.code).collect();
                if codes != months {
                    return Err(corrupt("kalendar krugova se ne slaže s popisom krugova"));
                }
                defs.iter()
                    .map(|d| {
                        let kind = PeriodKind::from_code(&d.kind)
                            .ok_or_else(|| corrupt(format!("vrsta kruga {}", d.kind)))?;
                        let period = Period {
                            kind,
                            index: d.index,
                            start_day: d.start_day,
                            days: d.days,
                        };
                        if period.code() != d.code.code() {
                            return Err(corrupt(format!(
                                "krug {} nije {}",
                                d.code.code(),
                                period.code()
                            )));
                        }
                        Ok(RoundInfo {
                            key: d.code,
                            period,
                        })
                    })
                    .collect()
            }
        }
    }

    pub(crate) fn start_state(self, c: &Content, p: Profile) -> Result<CoreState, GameError> {
        match self {
            Model::V1 => {
                let spec = c.catalog.profile(p).map_err(debug)?;
                Ok(CoreState::V1(
                    sim_hotel::HotelState::from_profile(spec).map_err(debug)?,
                ))
            }
            Model::V2 => Ok(CoreState::V2(Box::new(
                sim_hotel_v2::HotelState::from_profile(v2_catalog(c)?, p).map_err(debug)?,
            ))),
        }
    }

    /// v1: tablica za cijelu sezonu (kao prije). v2: tablica za krugove ove igre (`period_indices` traži iste krugove).
    pub(crate) fn refs(
        self,
        c: &Content,
        category: Category,
        rounds: &[RoundInfo],
    ) -> Result<CoreRefs, GameError> {
        match self {
            Model::V1 => Ok(CoreRefs::V1(
                sim_hotel::RefTable::compute(&c.catalog, category).map_err(debug)?,
            )),
            Model::V2 => {
                let periods: Vec<Period> = rounds.iter().map(|r| r.period).collect();
                Ok(CoreRefs::V2(
                    sim_hotel_v2::RefTable::compute(v2_catalog(c)?, category, &periods)
                        .map_err(debug)?,
                ))
            }
        }
    }

    pub(crate) fn start_indices(
        self,
        s: &CoreState,
        c: &Content,
        r: &CoreRefs,
    ) -> Result<IndexSet, GameError> {
        match (self, s, r) {
            (Model::V1, CoreState::V1(s), CoreRefs::V1(r)) => {
                sim_hotel::start_indices(s, &c.catalog, r).map_err(debug)
            }
            (Model::V2, CoreState::V2(s), CoreRefs::V2(r)) => {
                sim_hotel_v2::start_indices(s, v2_catalog(c)?, r).map_err(debug)
            }
            _ => Err(mismatch()),
        }
    }

    /// Zadani plan kruga (DEMO §6.1; v2 s referencom razdoblja u cijelom euru).
    pub(crate) fn default_plan(
        self,
        s: &CoreState,
        r: &RoundInfo,
        c: &Content,
    ) -> Result<CorePlan, GameError> {
        match (self, s) {
            (Model::V1, CoreState::V1(s)) => Ok(CorePlan::V1(
                MonthPlan::default_for(s, r.key.require_month()?, &c.catalog).map_err(debug)?,
            )),
            (Model::V2, CoreState::V2(s)) => Ok(CorePlan::V2(
                PeriodPlan::default_for(s, &r.period, v2_catalog(c)?).map_err(debug)?,
            )),
            _ => Err(mismatch()),
        }
    }

    /// Plan ranijeg kruga za ovaj krug. v2: `carry_into` (stisne u granice novog kruga, bez obnove).
    /// v1 plan ne prenosi: krug bez `PlanSet` dobiva zadani plan, kao prije sloja 1.
    pub(crate) fn carry_plan(
        self,
        prev: &CorePlan,
        s: &CoreState,
        r: &RoundInfo,
        c: &Content,
    ) -> Result<CorePlan, GameError> {
        match (self, prev, s) {
            (Model::V1, CorePlan::V1(_), CoreState::V1(_)) => self.default_plan(s, r, c),
            (Model::V2, CorePlan::V2(p), CoreState::V2(s)) => Ok(CorePlan::V2(
                p.carry_into(s, &r.period, v2_catalog(c)?).map_err(debug)?,
            )),
            _ => Err(mismatch()),
        }
    }

    /// Plan drugog modela je `WrongModel`; pravila plana daju `Plan` (v1) ili `PlanV2` (v2).
    pub(crate) fn validate_plan(
        self,
        p: &CorePlan,
        s: &CoreState,
        r: &RoundInfo,
        c: &Content,
    ) -> Result<(), GameError> {
        match (self, p, s) {
            (Model::V1, CorePlan::V1(p), CoreState::V1(s)) => p
                .validate(s, r.key.require_month()?, &c.catalog)
                .map_err(GameError::Plan),
            (Model::V2, CorePlan::V2(p), CoreState::V2(s)) => p
                .validate(s, &r.period, v2_catalog(c)?)
                .map_err(GameError::PlanV2),
            (Model::V1, CorePlan::V2(_), _) | (Model::V2, CorePlan::V1(_), _) => {
                Err(GameError::WrongModel)
            }
            _ => Err(mismatch()),
        }
    }

    pub(crate) fn robot_plan(
        self,
        st: Strategy,
        s: &CoreState,
        r: &RoundInfo,
        c: &Content,
        rng: &mut Rng,
    ) -> Result<CorePlan, GameError> {
        match (self, s) {
            (Model::V1, CoreState::V1(s)) => Ok(CorePlan::V1(
                st.plan(s, r.key.require_month()?, &c.catalog, rng)
                    .map_err(debug)?,
            )),
            (Model::V2, CoreState::V2(s)) => Ok(CorePlan::V2(
                sim_hotel_v2::robot_plan(st, s, &r.period, v2_catalog(c)?, rng).map_err(debug)?,
            )),
            _ => Err(mismatch()),
        }
    }

    /// Obračun jednog hotela u krugu. `report` je uvijek v1 oblika (za v2 sažetak kruga), `detail` samo za v2.
    pub(crate) fn resolve(
        self,
        s: &CoreState,
        r: &RoundInfo,
        p: &CorePlan,
        o: &[SituationOutcome],
        c: &Content,
        refs: &CoreRefs,
    ) -> Result<Resolved, GameError> {
        match (self, s, p, refs) {
            (Model::V1, CoreState::V1(s), CorePlan::V1(p), CoreRefs::V1(refs)) => {
                let report =
                    resolve_month(s, r.key.require_month()?, p, o, &c.catalog).map_err(debug)?;
                let indices = month_indices(&report, &c.catalog, refs).map_err(debug)?;
                Ok(Resolved {
                    report: MonthReportDto::from(&report),
                    detail: None,
                    indices,
                })
            }
            (Model::V2, CoreState::V2(s), CorePlan::V2(p), CoreRefs::V2(refs)) => {
                let cat = v2_catalog(c)?;
                let report = resolve_period(s, &r.period, p, o, cat).map_err(debug)?;
                let indices = period_indices(&report, cat, refs).map_err(debug)?;
                Ok(Resolved {
                    report: MonthReportDto::from(&report.summary),
                    detail: Some(DetailDto {
                        after: (&report.after).into(),
                        accounts: Some((&report.accounts).into()),
                    }),
                    indices,
                })
            }
            _ => Err(mismatch()),
        }
    }

    /// Stanje nakon obračunatog kruga: v1 iz `report.after`, v2 iz `detail.after`.
    pub(crate) fn state_after(self, res: &HotelResolution) -> Result<CoreState, GameError> {
        match self {
            Model::V1 => Ok(CoreState::V1(sim_hotel::HotelState::try_from(
                &res.report.after,
            )?)),
            Model::V2 => {
                let d = res
                    .detail
                    .as_ref()
                    .ok_or_else(|| corrupt("v2 obračun bez stanja v2"))?;
                Ok(CoreState::V2(Box::new(sim_hotel_v2::HotelState::try_from(
                    &d.after,
                )?)))
            }
        }
    }

    /// Tijelo zahtjeva plana (`Game::plan_from_dto`): v1 plan nema odjela, pa je tijelo s `departments`
    /// neispravno (`GameError::PlanBody` → 422), ne tiho zanemareno.
    pub(crate) fn check_plan_body(self, d: &PlanDto) -> Result<(), GameError> {
        match self {
            Model::V1 if !d.departments.is_empty() => Err(GameError::PlanBody),
            Model::V1 | Model::V2 => Ok(()),
        }
    }

    /// v2 plan traži postavke odjela (`seasonal_target` i `maintenance` iz DTO-a se ne čitaju); bez njih je to
    /// pokvaren dnevnik. Zahtjev bez odjela dopunjuje `Game::plan_from_dto` (`needs_departments`).
    pub(crate) fn plan_from_dto(self, d: &PlanDto) -> Result<CorePlan, GameError> {
        match self {
            Model::V1 => Ok(CorePlan::V1(MonthPlan::from(d))),
            Model::V2 => Ok(CorePlan::V2(PeriodPlan::try_from(d)?)),
        }
    }

    /// v2 tijelo plana bez odjela (API do Taska 15): odjeli iz važećeg plana `eff`, pa ukupni sezonci i razina
    /// održavanja iz tijela (`PeriodPlan::with_totals`), da se studentova promjena ne izgubi. v1 tijelo je cijeli
    /// plan. Granice provjerava `validate_plan` pri spremanju.
    pub(crate) fn plan_from_dto_without_departments(
        self,
        d: &PlanDto,
        eff: &CorePlan,
        s: &CoreState,
        r: &RoundInfo,
        c: &Content,
    ) -> Result<CorePlan, GameError> {
        match (self, eff, s) {
            (Model::V1, _, _) => self.plan_from_dto(d),
            (Model::V2, CorePlan::V2(eff), CoreState::V2(s)) => {
                let mut full = d.clone();
                full.departments = PlanDto::from(eff).departments;
                let plan = PeriodPlan::try_from(&full)?;
                Ok(CorePlan::V2(
                    plan.with_totals(
                        s,
                        &r.period,
                        v2_catalog(c)?,
                        d.seasonal_target,
                        d.maintenance.into(),
                    )
                    .map_err(debug)?,
                ))
            }
            _ => Err(mismatch()),
        }
    }

    /// Treba li tijelu plana dopuniti postavke odjela iz važećeg plana (v2 zahtjev bez `departments`; do Taska 15
    /// `PlanBody` ih nema). v1 plan nema odjela.
    pub(crate) fn needs_departments(self, d: &PlanDto) -> bool {
        match self {
            Model::V1 => false,
            Model::V2 => d.departments.is_empty(),
        }
    }

    /// Granice izbornika radnji (iste brojke koje provjerava `validate_plan`).
    pub(crate) fn plan_limits(
        self,
        s: &CoreState,
        r: &RoundInfo,
        c: &Content,
    ) -> Result<PlanLimits, GameError> {
        match (self, s) {
            (Model::V1, CoreState::V1(s)) => plan_limits(s, r.key.require_month()?, &c.catalog),
            (Model::V2, CoreState::V2(s)) => {
                let cat = v2_catalog(c)?;
                let l = PeriodPlan::limits(s, &r.period, cat).map_err(debug)?;
                let (costs, dept_costs) = v2_costs(s, &r.period, cat)?;
                Ok(PlanLimits {
                    price_min_cents: l.price_min_cents,
                    price_max_cents: l.price_max_cents,
                    marketing_max_cents: l.marketing_max_cents,
                    seasonal_min: l.seasonal_min,
                    seasonal_max: l.seasonal_max,
                    wage_min_pct: l.wage_min_pct,
                    wage_max_pct: l.wage_max_pct,
                    renovation_available: l.renovation_available,
                    seasonal_step_total: Some(l.seasonal_step_total),
                    departments: l
                        .departments
                        .iter()
                        .map(|(code, d)| {
                            let own = dept_costs
                                .get(code.as_str())
                                .copied()
                                .ok_or_else(|| corrupt("odjel bez cijena odluka"))?;
                            Ok((
                                code.as_str().to_owned(),
                                DeptLimitsView {
                                    seasonal_min: d.seasonal_min,
                                    seasonal_max: d.seasonal_max,
                                    price_level_min_pct: d.price_level_min_pct,
                                    price_level_max_pct: d.price_level_max_pct,
                                    can_close: d.can_close,
                                    maintenance: own.maintenance,
                                    seasonal_worker_cents: own.seasonal_worker_cents,
                                },
                            ))
                        })
                        .collect::<Result<_, GameError>>()?,
                    costs,
                })
            }
            _ => Err(mismatch()),
        }
    }

    /// Katalog v2 za poglede odjela i računa dobiti; v1 igra ih nema (`WrongModel`).
    pub(crate) fn require_v2(self, c: &Content) -> Result<&sim_hotel_v2::Catalog, GameError> {
        match self {
            Model::V1 => Err(GameError::WrongModel),
            Model::V2 => v2_catalog(c),
        }
    }

    /// Stanje v2 (odjeli) iz stanja igre; v1 stanje je `WrongModel`.
    pub(crate) fn v2_state(self, s: &CoreState) -> Result<&sim_hotel_v2::HotelState, GameError> {
        match (self, s) {
            (Model::V1, _) => Err(GameError::WrongModel),
            (Model::V2, CoreState::V2(s)) => Ok(s),
            _ => Err(mismatch()),
        }
    }

    /// Računi odjela jednog obračunatog kruga (v2); v1 obračun ih nema (`WrongModel`).
    pub(crate) fn accounts(self, res: &HotelResolution) -> Result<PeriodAccounts, GameError> {
        match self {
            Model::V1 => Err(GameError::WrongModel),
            Model::V2 => {
                let a = res
                    .detail
                    .as_ref()
                    .and_then(|d| d.accounts.as_ref())
                    .ok_or_else(|| corrupt("v2 obračun bez računa odjela"))?;
                PeriodAccounts::try_from(a)
            }
        }
    }

    /// Budžet (🟨 P8): račun odjela svakog kruga sezone kad hotel od početnog stanja profila u svakom krugu igra
    /// zadani plan (`PeriodPlan::default_for`), bez situacija; isti kalendar kao igra. Redom `rounds`.
    pub(crate) fn budget_accounts(
        self,
        c: &Content,
        profile: Profile,
        rounds: &[RoundInfo],
        refs: &CoreRefs,
    ) -> Result<Vec<PeriodAccounts>, GameError> {
        let (cat, refs) = match (self, refs) {
            (Model::V1, _) => return Err(GameError::WrongModel),
            (Model::V2, CoreRefs::V2(r)) => (v2_catalog(c)?, r),
            _ => return Err(mismatch()),
        };
        let kind = rounds
            .first()
            .ok_or_else(|| corrupt("igra bez krugova"))?
            .period
            .kind;
        let season = run_season_with(
            profile,
            kind,
            cat,
            &[],
            refs,
            &mut |s, p| PeriodPlan::default_for(s, p, cat).map_err(ResolveError::Catalog),
            &mut |_| Err(ResolveError::BadChoice),
        )
        .map_err(debug)?;
        if season.periods.len() != rounds.len() {
            return Err(corrupt("budžet nema isti broj krugova kao igra"));
        }
        Ok(season
            .periods
            .into_iter()
            .map(|(report, _)| report.accounts)
            .collect())
    }

    /// Izravni trošak po prodanoj noći: `cpor_cents` kategorije, isto polje koje čita obračun (v1 i v2 Smještaj;
    /// `Catalog::validate` v2 traži da je `direct_per_use_cents` Smještaja isti broj).
    pub(crate) fn rooms_direct_cost(
        self,
        c: &Content,
        category: Category,
    ) -> Result<i64, GameError> {
        let base = match self {
            Model::V1 => &c.catalog,
            Model::V2 => &v2_catalog(c)?.base,
        };
        Ok(base.category(category).map_err(debug)?.cpor_cents)
    }

    /// Ishod situacije za obračun: v1 kakav jest; v2 uz oznaku „samo nagrada“ (S-4) bez štete
    /// (`sim_hotel_v2::situations::reward_only`), za ponuđen, slobodan i neodgovoren odgovor. Oznaku određuje
    /// pozivatelj (`Game::no_penalty`): predložak primjerka iz dnevnika, ne rastavljanje šifre.
    pub(crate) fn situation_outcome(
        self,
        o: SituationOutcome,
        no_penalty: bool,
    ) -> SituationOutcome {
        match self {
            Model::V1 => o,
            Model::V2 if no_penalty => sim_hotel_v2::situations::reward_only(o),
            Model::V2 => o,
        }
    }

    /// Izvlači li igra situacije iz predložaka: samo v2 igra čiji snimak ima predloške. Je li to uistinu tako i za
    /// ovu igru, odlučuje dnevnik (`SituationsDrawn` u prvom krugu), ne snimak.
    pub(crate) fn can_draw(self, c: &Content) -> bool {
        self == Model::V2 && c.situation_templates.is_some()
    }

    /// Kolektivna situacija kruga, jedna za sve hotele igre: ulaz su svi hoteli (granica grupa, slobodna mjesta,
    /// povijest), isti kod kao u sezoni robota (`collective_for_round`). Samo v2.
    pub(crate) fn draw_collective(
        self,
        c: &Content,
        set: &TemplateSet,
        all: &[DrawInput],
    ) -> Result<Option<DrawnSituation>, GameError> {
        let Some(first) = all.first() else {
            return Ok(None);
        };
        let mut hotels = Vec::with_capacity(all.len());
        for i in all {
            hotels.push(RoundHotel {
                view: DrawView {
                    state: self.v2_state(i.state)?,
                    previous: i.previous,
                    indices: i.indices,
                },
                history: i.history,
                slot: i.slot,
            });
        }
        collective_for_round(
            &c.situations,
            set,
            first.seed,
            &first.round.period,
            &hotels,
            v2_catalog(c)?,
        )
        .map_err(debug)
    }

    /// Primjerci jednog hotela u krugu uz zadanu kolektivnu: isti poziv generatora kao sezona robota
    /// (`draw_for_round_with`), sa sidrima S1–S4 iz sadržaja. Samo v2.
    pub(crate) fn draw_situations(
        self,
        c: &Content,
        set: &TemplateSet,
        i: &DrawInput,
        shared: Option<&DrawnSituation>,
    ) -> Result<Vec<DrawnSituation>, GameError> {
        let state = self.v2_state(i.state)?;
        let view = DrawView {
            state,
            previous: i.previous,
            indices: i.indices,
        };
        let (_, found) = draw_for_round_with(
            &c.situations,
            set,
            (i.seed, i.slot),
            &i.round.period,
            (&view, i.history),
            (v2_catalog(c)?, shared),
        )
        .map_err(debug)?;
        Ok(found)
    }

    /// Cijeli obračun jednog hotela u krugu (v2) — za izvlačenje idućeg kruga, koje gleda obračun prošlog.
    pub(crate) fn period_report(
        self,
        s: &CoreState,
        r: &RoundInfo,
        p: &CorePlan,
        o: &[SituationOutcome],
        c: &Content,
    ) -> Result<PeriodReport, GameError> {
        match (self, s, p) {
            (Model::V2, CoreState::V2(s), CorePlan::V2(p)) => {
                resolve_period(s, &r.period, p, o, v2_catalog(c)?).map_err(debug)
            }
            (Model::V1, _, _) => Err(GameError::WrongModel),
            _ => Err(mismatch()),
        }
    }

    /// Situacije kruga: v1 po mjesecu, v2 u krugu koji sadrži 1. dan mjeseca situacije (🟨 P13).
    pub(crate) fn situations_in<'a>(
        self,
        r: &RoundInfo,
        all: &'a [SituationTemplate],
    ) -> Vec<&'a SituationTemplate> {
        match self {
            Model::V1 => {
                let month = r.key.month();
                all.iter().filter(|t| Some(t.month) == month).collect()
            }
            Model::V2 => sim_hotel_v2::situations_for(&r.period, all),
        }
    }

    /// Jedini put do kataloga za igru koja može biti v2: v1 čita `content.catalog`, v2 `base` kataloga v2.
    pub(crate) fn catalog(self, c: &Content) -> Result<&sim_hotel::Catalog, GameError> {
        match self {
            Model::V1 => Ok(&c.catalog),
            Model::V2 => Ok(&v2_catalog(c)?.base),
        }
    }

    /// Uobičajena cijena sobe u krugu (za mentora): v1 mjesečna referenca, v2 prosjek po danima kruga.
    pub(crate) fn price_ref_cents(
        self,
        category: Category,
        r: &RoundInfo,
        c: &Content,
    ) -> Result<i64, GameError> {
        let spec = self.catalog(c)?.category(category).map_err(debug)?;
        match self {
            Model::V1 => Ok(spec.price_ref_cents.get(r.key.require_month()?)),
            Model::V2 => avg_i64(&spec.price_ref_cents, &r.period).map_err(debug),
        }
    }
}

impl CoreState {
    /// v1 oblik stanja (pogledi, mentor, indeksi); za v1 je to samo stanje.
    pub fn summary(&self) -> sim_hotel::HotelState {
        match self {
            CoreState::V1(s) => s.clone(),
            CoreState::V2(s) => s.summary(),
        }
    }

    pub fn category(&self) -> Category {
        match self {
            CoreState::V1(s) => s.category,
            CoreState::V2(s) => s.category,
        }
    }
}

impl CorePlan {
    pub fn to_dto(&self) -> PlanDto {
        match self {
            CorePlan::V1(p) => p.into(),
            CorePlan::V2(p) => p.into(),
        }
    }
}

impl From<MonthPlan> for CorePlan {
    fn from(p: MonthPlan) -> Self {
        CorePlan::V1(p)
    }
}
