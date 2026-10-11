//! Katalog v2: v1 katalog (`base`) i koeficijenti po vrsti razdoblja (MODEL_V2 §4.3). Pomaci prema cilju
//! (moral, zadovoljstvo, ocjena) ovise o duljini kruga; mjesečni koeficijenti su v1 konstante.

use std::collections::BTreeMap;

use sim_hotel::{CatalogError, Month, MonthTable, Profile};

use crate::calendar::PeriodKind;
use crate::department::{DeptCode, DeptSpec, RevenueModel, UndistributedKind, UsaliClass};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodCoefficients {
    /// Udio razlike do ciljnog morala koji se zatvori u jednom krugu (bp).
    pub morale_step_bp: u16,
    /// Udio razlike do ciljnog zadovoljstva gostiju u jednom krugu (bp).
    pub satisfaction_step_bp: u16,
    /// Udio razlike do ciljne ocjene u jednom krugu (bp).
    pub rating_step_bp: u16,
    /// Najveća promjena broja sezonaca u jednom krugu (ljudi).
    pub seasonal_step: u16,
}

/// Gospodarstvo v2: gosti po sobi, granice razine cijene, popis fiksnih naknada (osiguranje, najam, porez).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EconomyV2 {
    pub guests_per_room_hundredths: u16,
    pub price_level_min_pct: u16,
    pub price_level_max_pct: u16,
    pub fixed_charges: BTreeMap<String, FixedChargeSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedChargeSpec {
    pub name: String,
}

/// Početno stanje jednog odjela u profilu hotela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptStart {
    /// Ime odjela u ovom profilu; `None` = ime iz kataloga.
    pub name: Option<String>,
    pub open: bool,
    pub permanent: u16,
    pub seasonal: u16,
    pub condition_pts: i32,
    pub satisfaction_pts: i32,
    /// Stalni mjesečni trošak u ovom profilu; `None` = iz `DeptSpec`.
    pub fixed_monthly_cents: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileV2 {
    pub departments: BTreeMap<DeptCode, DeptStart>,
    pub fixed_charges_cents: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub base: sim_hotel::Catalog,
    pub periods: BTreeMap<PeriodKind, PeriodCoefficients>,
    pub departments: BTreeMap<DeptCode, DeptSpec>,
    pub profiles: BTreeMap<Profile, ProfileV2>,
    pub economy: EconomyV2,
}

fn step_within(v: u16, field: &str) -> Result<(), CatalogError> {
    if (1..=10_000).contains(&v) {
        Ok(())
    } else {
        Err(CatalogError::OutOfRange(field.to_string()))
    }
}

impl Catalog {
    pub fn coeffs(&self, k: PeriodKind) -> Result<&PeriodCoefficients, CatalogError> {
        self.periods
            .get(&k)
            .ok_or_else(|| CatalogError::Missing(format!("v2.periods.{}", k.code())))
    }

    pub fn dept(&self, c: &DeptCode) -> Result<&DeptSpec, CatalogError> {
        self.departments
            .get(c)
            .ok_or_else(|| CatalogError::Missing(format!("v2.departments.{}", c.as_str())))
    }

    pub fn profile_v2(&self, p: Profile) -> Result<&ProfileV2, CatalogError> {
        self.profiles
            .get(&p)
            .ok_or_else(|| CatalogError::Missing(format!("v2.profiles.{}", p.code())))
    }

    /// Ime odjela u profilu (ime iz profila ako postoji, inače iz kataloga).
    pub fn dept_name(&self, p: Profile, c: &DeptCode) -> Result<String, CatalogError> {
        let own = self
            .profile_v2(p)?
            .departments
            .get(c)
            .and_then(|d| d.name.clone());
        match own {
            Some(n) => Ok(n),
            None => Ok(self.dept(c)?.name.clone()),
        }
    }

    /// v1 provjera pa koeficijenti obje vrste razdoblja: pomaci u `1..=10_000` bp, korak sezonaca barem 1.
    pub fn validate(&self) -> Result<(), CatalogError> {
        self.base.validate()?;
        for k in PeriodKind::ALL {
            let p = self.coeffs(k)?;
            let field = |name: &str| format!("v2.periods.{}.{name}", k.code());
            step_within(p.morale_step_bp, &field("morale_step_bp"))?;
            step_within(p.satisfaction_step_bp, &field("satisfaction_step_bp"))?;
            step_within(p.rating_step_bp, &field("rating_step_bp"))?;
            if p.seasonal_step < 1 {
                return Err(CatalogError::OutOfRange(field("seasonal_step")));
            }
        }
        self.validate_departments()
    }

    /// Odjeli, profili i gospodarstvo v2 (MODEL_V2 §3.2).
    fn validate_departments(&self) -> Result<(), CatalogError> {
        let rooms_ok = self
            .departments
            .get(&DeptCode::rooms())
            .is_some_and(|d| d.class == UsaliClass::Rooms && d.revenue == RevenueModel::Rooms);
        if !rooms_ok {
            return Err(CatalogError::Missing("v2.departments.rooms".to_string()));
        }
        // Izravni trošak po prodanoj noći ima jedan izvor: obračun Smještaja i KPI čitaju v1 `cpor_cents`
        // kategorije; `direct_per_use_cents` Smještaja u departments.json mora biti isti broj, inače bi ekran
        // odjela i obračun pričali dvije priče.
        let rooms_direct = self
            .departments
            .get(&DeptCode::rooms())
            .map_or(0, |d| d.direct_per_use_cents);
        if self
            .base
            .categories
            .values()
            .any(|c| c.cpor_cents != rooms_direct)
        {
            return Err(CatalogError::OutOfRange(
                "v2.departments.rooms.direct_per_use_cents".to_string(),
            ));
        }
        for kind in UNDISTRIBUTED {
            let n = self
                .departments
                .values()
                .filter(|d| d.class == UsaliClass::Undistributed(kind))
                .count();
            if n != 1 {
                return Err(CatalogError::OutOfRange(format!(
                    "v2.departments.{}",
                    UsaliClass::Undistributed(kind).code()
                )));
            }
        }
        let e = &self.economy;
        if !(90 <= e.price_level_min_pct
            && e.price_level_min_pct < 100
            && 100 < e.price_level_max_pct
            && e.price_level_max_pct <= 150)
        {
            return Err(CatalogError::OutOfRange(
                "v2.economy.price_level_pct".to_string(),
            ));
        }
        for (code, d) in &self.departments {
            let staffed = self
                .profiles
                .values()
                .filter_map(|p| p.departments.get(code))
                .any(|s| s.permanent > 0 || s.seasonal > 0);
            // Neraspoređeni odjeli imaju stalne ljude i kapacitet 0.
            if !d.class.is_undistributed()
                && (d.seasonal_max > 0 || staffed)
                && d.capacity_tenths_per_staff_day == 0
            {
                return Err(CatalogError::NotPositive(format!(
                    "v2.departments.{}.capacity_tenths_per_staff_day",
                    code.as_str()
                )));
            }
        }
        for (profile, spec) in &self.base.profiles {
            let p = self.profile_v2(*profile)?;
            let label = |f: &str| format!("v2.profiles.{}.{f}", profile.code());
            for (code, d) in &self.departments {
                let required = d.class == UsaliClass::Rooms || d.class.is_undistributed();
                if required && !p.departments.contains_key(code) {
                    return Err(CatalogError::Missing(label(&format!(
                        "departments.{}",
                        code.as_str()
                    ))));
                }
            }
            // Sezonci su točno v1 profil (sezonski plan hotela je v1 kategorija); stalnih smije biti više od v1, ali
            // višak smije biti samo u neraspoređenim odjelima (Uprava, Prodaja, Održavanje, Energija).
            let (mut permanent, mut seasonal, mut operated) = (0_u32, 0_u32, 0_u32);
            for (code, s) in &p.departments {
                let Some(d) = self.departments.get(code) else {
                    return Err(CatalogError::Missing(format!(
                        "v2.departments.{}",
                        code.as_str()
                    )));
                };
                // Početni sezonci moraju biti u granicama odjela i zatvoren odjel bez sezonaca, inače zadani plan
                // ne može u jednom koraku doći do valjanog stanja i igra zapne pri obračunu.
                if s.seasonal > d.seasonal_max {
                    return Err(CatalogError::OutOfRange(label(&format!(
                        "departments.{}.seasonal",
                        code.as_str()
                    ))));
                }
                if !s.open && s.seasonal > 0 {
                    return Err(CatalogError::OutOfRange(label(&format!(
                        "departments.{}.open",
                        code.as_str()
                    ))));
                }
                for v in [s.condition_pts, s.satisfaction_pts] {
                    if !(0_i32..=10_000_i32).contains(&v) {
                        return Err(CatalogError::OutOfRange(label(&format!(
                            "departments.{}",
                            code.as_str()
                        ))));
                    }
                }
                permanent = permanent.saturating_add(u32::from(s.permanent));
                seasonal = seasonal.saturating_add(u32::from(s.seasonal));
                if !d.class.is_undistributed() {
                    operated = operated.saturating_add(u32::from(s.permanent));
                }
            }
            let v1_permanent = u32::from(spec.permanent);
            if permanent < v1_permanent
                || operated > v1_permanent
                || seasonal != u32::from(spec.seasonal)
            {
                return Err(CatalogError::OutOfRange(label("departments.staff")));
            }
            self.validate_staffing(*profile, spec, p)?;
            for name in p.fixed_charges_cents.keys() {
                if !e.fixed_charges.contains_key(name) {
                    return Err(CatalogError::Missing(format!(
                        "v2.economy.fixed_charges.{name}"
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Catalog {
    /// Norma osoblja (🟨 Task 23b): za svaki odjel s normom početni ljudi profila (stalni + sezonci) moraju biti
    /// u rasponu kategorije profila, `ljudi · 1000 ∈ [min · sobe, max · sobe]` (norma je u desetinkama na 100
    /// soba). Odjel s normom bez retka za kategoriju profila, ili norma s `min` 0 ili `min > max`, je greška
    /// sadržaja odjela.
    fn validate_staffing(
        &self,
        profile: Profile,
        spec: &sim_hotel::ProfileSpec,
        p: &ProfileV2,
    ) -> Result<(), CatalogError> {
        let rooms = u64::from(spec.rooms);
        for (code, d) in self
            .departments
            .iter()
            .filter(|(_, d)| !d.staffing.is_empty())
        {
            let dept_label = || format!("v2.departments.{}.staffing", code.as_str());
            if d.staffing.values().any(|n| {
                n.min_tenths_per_100_rooms == 0
                    || n.min_tenths_per_100_rooms > n.max_tenths_per_100_rooms
            }) {
                return Err(CatalogError::OutOfRange(dept_label()));
            }
            let Some(n) = d.staffing.get(&spec.category) else {
                return Err(CatalogError::OutOfRange(dept_label()));
            };
            let people = p.departments.get(code).map_or(0_u64, |s| {
                u64::from(s.permanent).saturating_add(u64::from(s.seasonal))
            });
            let scaled = people.saturating_mul(1_000);
            let min = u64::from(n.min_tenths_per_100_rooms).saturating_mul(rooms);
            let max = u64::from(n.max_tenths_per_100_rooms).saturating_mul(rooms);
            if !(min..=max).contains(&scaled) {
                return Err(CatalogError::OutOfRange(format!(
                    "v2.profiles.{}.departments.{}.staffing",
                    profile.code(),
                    code.as_str()
                )));
            }
        }
        Ok(())
    }
}

const UNDISTRIBUTED: [UndistributedKind; 4] = [
    UndistributedKind::AdminGeneral,
    UndistributedKind::SalesMarketing,
    UndistributedKind::Maintenance,
    UndistributedKind::Utilities,
];

fn no_effect() -> sim_hotel::LevelEffect {
    sim_hotel::LevelEffect {
        cost_cents: 0,
        equipment_pts: 0,
    }
}

/// Odjel bez osoblja, troškova i prihoda; polazište za degenerirane odjele.
fn bare(name: &str, class: UsaliClass, revenue: RevenueModel) -> DeptSpec {
    DeptSpec {
        name: name.to_string(),
        class,
        revenue,
        cost_of_sales_bp: 0,
        direct_per_use_cents: 0,
        wage_cents: 0,
        capacity_tenths_per_staff_day: 0,
        wear_pts_per_month: 0,
        maintenance: [no_effect(), no_effect(), no_effect()],
        fixed_monthly_cents: 0,
        per_occupied_room_cents: 0,
        seasonal_max: 0,
        seasonal_plan: MonthTable([0; 4]),
        can_close: false,
        staffing: BTreeMap::new(),
    }
}

fn start(permanent: u16, seasonal: u16, condition_pts: i32, satisfaction_pts: i32) -> DeptStart {
    DeptStart {
        name: None,
        open: true,
        permanent,
        seasonal,
        condition_pts,
        satisfaction_pts,
        fixed_monthly_cents: None,
    }
}

/// Degenerirani odjeli: Smještaj s v1 brojkama, `ostalo` (v1 `other_revenue_bp`) i četiri prazna neraspoređena.
fn degenerate_departments(base: &sim_hotel::Catalog) -> BTreeMap<DeptCode, DeptSpec> {
    // Plaća, trošak po prodanoj sobi i plan sezonaca su po kategoriji; referenca je kategorija prvog profila.
    let category = base
        .profiles
        .values()
        .next()
        .map(|p| p.category)
        .or_else(|| base.categories.keys().next().copied());
    let cat = category.and_then(|c| base.categories.get(&c));
    let a = &base.actions;
    let mut rooms = bare("Smještaj", UsaliClass::Rooms, RevenueModel::Rooms);
    rooms.wage_cents = cat.map_or(0, |c| c.wage_cents);
    rooms.direct_per_use_cents = cat.map_or(0, |c| c.cpor_cents);
    rooms.capacity_tenths_per_staff_day = 24;
    rooms.wear_pts_per_month = 400_i32;
    rooms.maintenance = [
        a.maintenance_deferred.clone(),
        a.maintenance_regular.clone(),
        a.maintenance_intensive.clone(),
    ];
    rooms.seasonal_max = a.seasonal_max;
    rooms.seasonal_plan = cat.map_or(MonthTable([0; 4]), |c| {
        MonthTable(Month::SEASON.map(|m| c.seasonal_plan.get(m)))
    });
    let other = bare(
        "Ostali prihodi",
        UsaliClass::Rentals,
        RevenueModel::ShareOfRooms {
            bp: base.economy.other_revenue_bp,
        },
    );
    let undist = |name: &str, k: UndistributedKind| {
        bare(name, UsaliClass::Undistributed(k), RevenueModel::None)
    };
    BTreeMap::from([
        (DeptCode::rooms(), rooms),
        (DeptCode("ostalo".to_string()), other),
        (
            DeptCode("admin".to_string()),
            undist("Uprava", UndistributedKind::AdminGeneral),
        ),
        (
            DeptCode("sales".to_string()),
            undist("Prodaja i marketing", UndistributedKind::SalesMarketing),
        ),
        (
            DeptCode("maintenance".to_string()),
            undist("Održavanje", UndistributedKind::Maintenance),
        ),
        (
            DeptCode("utilities".to_string()),
            undist("Energija", UndistributedKind::Utilities),
        ),
    ])
}

fn degenerate_profiles(base: &sim_hotel::Catalog) -> BTreeMap<Profile, ProfileV2> {
    let mut out = BTreeMap::new();
    for (profile, spec) in &base.profiles {
        let mut departments = BTreeMap::from([
            (
                DeptCode::rooms(),
                start(
                    spec.permanent,
                    spec.seasonal,
                    spec.equipment_pts,
                    spec.satisfaction_pts,
                ),
            ),
            (DeptCode("ostalo".to_string()), start(0, 0, 10_000, 7_000)),
            (DeptCode("sales".to_string()), start(0, 0, 10_000, 0)),
            (DeptCode("maintenance".to_string()), start(0, 0, 10_000, 0)),
            (DeptCode("utilities".to_string()), start(0, 0, 10_000, 0)),
        ]);
        departments.insert(
            DeptCode("admin".to_string()),
            DeptStart {
                fixed_monthly_cents: Some(spec.fixed_cents),
                ..start(0, 0, 10_000, 0)
            },
        );
        out.insert(
            *profile,
            ProfileV2 {
                departments,
                fixed_charges_cents: BTreeMap::new(),
            },
        );
    }
    out
}

/// Katalog v2 koji u mjesečnoj igri daje točno v1 (konstante iz `sim_hotel::resolve`); tjedni koeficijenti
/// su procjena [PROC] do sadržaja iz `content/hotel/v2/periods.json` (Task 4).
pub fn degenerate(base: &sim_hotel::Catalog) -> Catalog {
    Catalog {
        base: base.clone(),
        periods: BTreeMap::from([
            (
                PeriodKind::Month,
                PeriodCoefficients {
                    morale_step_bp: 3_000,
                    satisfaction_step_bp: 5_000,
                    rating_step_bp: 2_500,
                    seasonal_step: base.actions.seasonal_step,
                },
            ),
            (
                PeriodKind::Week,
                PeriodCoefficients {
                    morale_step_bp: 800,
                    satisfaction_step_bp: 1_500,
                    rating_step_bp: 700,
                    seasonal_step: 5,
                },
            ),
        ]),
        departments: degenerate_departments(base),
        profiles: degenerate_profiles(base),
        economy: EconomyV2 {
            guests_per_room_hundredths: 100,
            price_level_min_pct: 90,
            price_level_max_pct: 120,
            fixed_charges: BTreeMap::new(),
        },
    }
}
