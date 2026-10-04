// Neovisni model jezgre (DEMO §3, §4, §8, §12 + ADR-013) u BigInt cijelim brojevima.
// Postoji da zlatne vrijednosti u Rust testovima NE dolaze iz koda koji testiraju: brojke u
// `crates/sim-hotel/tests/*` izračunate su ovdje, nad FIKSNIM testnim katalogom (TEST_CAT, brojke iz
// DEMO §2.1 prije kalibracije). Kalibracija mijenja samo `content/hotel/*.json`, pa ovaj model ostaje
// valjan. Promjena formule = promjena ovdje I u Rustu, i testovi moraju opet pasti pa proći.
// Pokretanje: `node scripts/oracle-jezgra.mjs` ispisuje sve zlatne vrijednosti iz plana jezgre.

export function divRound(a, b) {
  if (b <= 0n) throw new Error(`divRound: nazivnik ${b}`);
  if (a >= 0n) return (2n * a + b) / (2n * b);
  return -((-2n * a + b) / (2n * b));
}
export const clamp = (x, lo, hi) => (x < lo ? lo : x > hi ? hi : x);
const max = (a, b) => (a > b ? a : b);
const min = (a, b) => (a < b ? a : b);

export const MONTHS = ["Jun", "Jul", "Aug", "Sep"];
export const DAYS = { Jun: 30n, Jul: 31n, Aug: 31n, Sep: 30n };

export const TEST_CAT = {
  priceRef: { Jun: 13500n, Jul: 16500n, Aug: 17500n, Sep: 13500n },
  baseOcc: { Jun: 7200n, Jul: 7800n, Aug: 8300n, Sep: 7300n },
  seasonalPlan: { Jun: 20n, Jul: 20n, Aug: 20n, Sep: 20n },
  elasticityBp: -8000n,
  wageCents: 220000n,
  cporCents: 2200n,
  hireCents: 150000n,
  otherRevenueBp: 3000n,
  marketingHalfCents: 1500000n,
  marketingMaxBp: 1500n,
  maintenance: { Deferred: [300000n, 0n], Regular: [1500000n, 400n], Intensive: [3000000n, 800n] },
  training: { None: [0n, 0n, 0n], Basic: [500000n, 400n, 200n], Full: [1000000n, 700n, 200n] },
  renovation: { cost: 15000000n, equipment: 2500n, satisfaction: 300n },
  reply: { cost: 150000n, rating: 5n },
  gopparFloor: 1000n,
};

export const PROFILE_C = {
  rooms: 120n,
  permanent: 30n,
  seasonal: 20n,
  wagePct: 100n,
  extraWage: 0n,
  morale: 5500n,
  experience: 6000n,
  equipment: 6000n,
  satisfaction: 7000n,
  rating: 840n,
  cash: 30000000n,
  fixed: 12000000n,
  load: 173n,
};
export const PROFILE_A = {
  ...PROFILE_C,
  rating: 910n,
  satisfaction: 7800n,
  morale: 4200n,
  load: 195n,
  cash: 12000000n,
  fixed: 15000000n,
};
export const PROFILE_B = {
  ...PROFILE_C,
  rating: 800n,
  equipment: 4000n,
  experience: 5500n,
  cash: 60000000n,
  wagePct: 97n,
  morale: 5800n,
  fixed: 9000000n,
};

export const turnoverBp = (morale) => 300n + divRound(15n * max(0n, 6000n - morale), 100n);

export function startState(p) {
  return {
    ...p,
    turnover: turnoverBp(p.morale),
    pending: { exp: 0n, morale: 0n, renovation: false },
    groups: [],
  };
}

export function defaultPlan(s, m, cat = TEST_CAT) {
  const lo = max(0n, s.seasonal - 10n);
  const hi = min(40n, s.seasonal + 10n);
  return {
    price: cat.priceRef[m],
    marketing: 0n,
    seasonalTarget: clamp(cat.seasonalPlan[m], lo, hi),
    wagePct: s.wagePct,
    training: "None",
    maintenance: "Regular",
    renovation: false,
    reply: false,
  };
}

// sit = zbroj učinaka situacija: { sat, morale, rating, equipment, cost, extraWage, groups: [{ month, rn, price }] }
export function resolveMonth(s0, m, plan, sit = {}, cat = TEST_CAT) {
  const s = structuredClone(s0);
  const D = DAYS[m];
  for (const g of sit.groups ?? []) s.groups.push(g);
  s.extraWage += sit.extraWage ?? 0n;
  const pend = s.pending;
  // osoblje
  const hires = max(0n, plan.seasonalTarget - s.seasonal);
  const released = max(0n, s.seasonal - plan.seasonalTarget);
  s.seasonal = plan.seasonalTarget;
  s.wagePct = plan.wagePct;
  const staff = s.permanent + s.seasonal;
  // tržište (§3.1, P9–P15)
  const out = clamp(divRound(4000n - s.equipment, 400n), 0n, s.rooms);
  const avail = (s.rooms - out) * D;
  const ref = cat.priceRef[m];
  const fc = clamp(10000n + divRound(cat.elasticityBp * (plan.price - ref), ref), 3000n, 16000n);
  const fu = max(0n, 10000n + divRound(54n * (s.rating - 840n), 10n));
  const fm = 10000n + divRound(cat.marketingMaxBp * plan.marketing, plan.marketing + cat.marketingHalfCents);
  const demand = max(0n, divRound(s.rooms * D * cat.baseOcc[m] * fc * fu * fm, 10n ** 16n));
  const capStaff = (staff * D * 24n) / 10n;
  const groupsHere = s.groups.filter((g) => g.month === m);
  const groupRn = groupsHere.reduce((a, g) => a + g.rn, 0n);
  const sold = min(min(groupRn + demand, avail), capStaff);
  const groupSold = min(groupRn, sold);
  let left = groupSold;
  let groupRevenue = 0n;
  for (const g of groupsHere) {
    const t = min(g.rn, left);
    left -= t;
    groupRevenue += t * g.price;
  }
  const indiv = sold - groupSold;
  s.groups = s.groups.filter((g) => g.month !== m);
  // ljudi (§3.3, P16–P22)
  const load = staff === 0n ? 300n : clamp(divRound(sold * 100n, staff * D), 0n, 300n);
  const penalty = clamp(100n * (load - 180n), 0n, 6000n);
  const releasePenalty = released > 0n ? -300n : 0n;
  const moraleTarget = clamp(
    5500n +
      200n * (plan.wagePct - 100n) -
      divRound(penalty, 2n) +
      pend.morale +
      (sit.morale ?? 0n) +
      releasePenalty,
    0n,
    10000n,
  );
  const morale = clamp(s.morale + divRound(3000n * (moraleTarget - s.morale), 10000n), 0n, 10000n);
  const turnover = turnoverBp(morale);
  const departures = clamp(divRound(staff * turnover, 10000n), 0n, staff);
  const n = min(departures + hires, staff);
  const expMix = staff === 0n ? s.experience : divRound(s.experience * (staff - n) + 2000n * n, staff);
  const experience = clamp(expMix + 300n + pend.exp, 0n, 10000n);
  // novac (§3.2, P25–P28)
  const [trainCost, trainExp, trainMorale] = cat.training[plan.training];
  const roomsRevenue = indiv * plan.price + groupRevenue;
  const otherRevenue = divRound(roomsRevenue * cat.otherRevenueBp, 10000n);
  const labor =
    staff * (cat.wageCents / 100n) * plan.wagePct +
    s.extraWage +
    (hires + departures) * cat.hireCents +
    trainCost;
  const [maintCost, maintEff] = cat.maintenance[plan.maintenance];
  const replyCost = plan.reply ? cat.reply.cost : 0n;
  const costs =
    labor + cat.cporCents * sold + maintCost + plan.marketing + s.fixed + replyCost + (sit.cost ?? 0n);
  const gop = roomsRevenue + otherRevenue - costs;
  const capex = plan.renovation ? cat.renovation.cost : 0n;
  let cash = s.cash + gop - capex;
  let interest = 0n;
  if (cash < 0n) {
    interest = divRound(-cash, 100n);
    cash -= interest;
  }
  const goppar = divRound(gop, s.rooms * D);
  // oprema i gosti (§3.3, P19–P23)
  const equipment = clamp(
    s.equipment -
      400n -
      (load > 180n ? 100n : 0n) +
      maintEff +
      (pend.renovation ? cat.renovation.equipment : 0n) +
      (sit.equipment ?? 0n),
    0n,
    10000n,
  );
  const quality = divRound(4000n * experience + 3000n * morale + 3000n * (10000n - penalty), 10000n);
  const value = clamp(7000n - divRound(10000n * (plan.price - ref), ref), 0n, 10000n);
  const satTarget = divRound(4500n * quality + 3000n * equipment + 2500n * value, 10000n);
  const satisfaction = clamp(
    s.satisfaction +
      divRound(5000n * (satTarget - s.satisfaction), 10000n) +
      (sit.sat ?? 0n) +
      (pend.renovation ? cat.renovation.satisfaction : 0n),
    0n,
    10000n,
  );
  const ratingTarget = divRound(14000n + satisfaction, 25n);
  const rating = clamp(
    s.rating +
      divRound(2500n * (ratingTarget - s.rating), 10000n) +
      (plan.reply ? cat.reply.rating : 0n) +
      (sit.rating ?? 0n),
    100n,
    1000n,
  );
  Object.assign(s, { morale, experience, equipment, satisfaction, rating, cash, load, turnover });
  s.pending = { exp: trainExp, morale: trainMorale, renovation: plan.renovation };
  return {
    state: s,
    market: { out, avail, fc, fu, fm, demand, capStaff, groupRn, sold, groupSold, indiv },
    people: { hires, released, staff, load, penalty, moraleTarget, turnover, departures, expMix },
    money: { roomsRevenue, otherRevenue, labor, costs, gop, capex, interest, goppar },
    guests: { quality, value, satTarget, ratingTarget },
  };
}

// Sirove vrijednosti indeksa (DEMO §4); centriranje na referentni hotel u istom mjesecu (ADR-013).
export function rawIndices(s, goppar, labor, refGoppar, cat = TEST_CAT) {
  const retention = clamp(10000n - divRound(25n * s.turnover, 2n), 0n, 10000n);
  const penalty = clamp(100n * (s.load - 180n), 0n, 6000n);
  const gi = clamp(
    5000n + divRound(5000n * (goppar - refGoppar), max(refGoppar, cat.gopparFloor)),
    0n,
    10000n,
  );
  const liq = s.cash <= 0n ? 0n : divRound(5000n * s.cash, s.fixed + labor);
  return {
    G: s.satisfaction,
    R: divRound((s.rating - 700n) * 10000n, 280n),
    Z: divRound(6000n * s.morale + 4000n * retention, 10000n),
    O: divRound(4500n * s.equipment + 3000n * s.experience + 2500n * (10000n - penalty), 10000n),
    F: divRound(7000n * gi + 3000n * liq, 10000n),
  };
}
export function centred(raw, ref) {
  const o = {};
  for (const k of ["G", "R", "Z", "O", "F"]) o[k] = clamp(5000n + raw[k] - ref[k], 0n, 10000n);
  return o;
}
export const baseLabor = (s, cat = TEST_CAT) =>
  (s.permanent + s.seasonal) * (cat.wageCents / 100n) * s.wagePct + s.extraWage;

export function refTable(cat = TEST_CAT, ref = PROFILE_C) {
  let s = startState(ref);
  const june = resolveMonth(s, "Jun", defaultPlan(s, "Jun", cat), {}, cat);
  const g0 = june.money.goppar;
  const start = { raw: rawIndices(s, g0, baseLabor(s, cat), g0, cat), goppar: g0 };
  const months = {};
  for (const m of MONTHS) {
    const r = resolveMonth(s, m, defaultPlan(s, m, cat), {}, cat);
    s = r.state;
    months[m] = {
      raw: rawIndices(s, r.money.goppar, r.money.labor, r.money.goppar, cat),
      goppar: r.money.goppar,
    };
  }
  return { start, months };
}
export function startIndices(p, refs, cat = TEST_CAT) {
  const s = startState(p);
  const june = resolveMonth(s, "Jun", defaultPlan(s, "Jun", cat), {}, cat);
  return centred(rawIndices(s, june.money.goppar, baseLabor(s, cat), refs.start.goppar, cat), refs.start.raw);
}
export function monthIndices(r, m, refs, cat = TEST_CAT) {
  return centred(
    rawIndices(r.state, r.money.goppar, r.money.labor, refs.months[m].goppar, cat),
    refs.months[m].raw,
  );
}

// Relativna ocjena (DEMO §8, P31)
export function gain(start, end) {
  if (end >= start) return start === 10000n ? 0n : divRound(10000n * (end - start), 10000n - start);
  return divRound(10000n * (end - start), start);
}
export function progress(start, end) {
  return divRound(
    ["G", "R", "Z", "O", "F"].reduce((a, k) => a + gain(start[k], end[k]), 0n),
    5n,
  );
}

if (import.meta.url === `file://${process.argv[1]?.replaceAll("\\", "/").replace(/^([A-Za-z]):/, "/$1:")}`) {
  const show = (o) => JSON.stringify(o, (_k, v) => (typeof v === "bigint" ? v.toString() : v));
  const C = startState(PROFILE_C);
  const refs = refTable();
  console.log("refs", show(refs));
  for (const [name, p] of [
    ["A", PROFILE_A],
    ["B", PROFILE_B],
    ["C", PROFILE_C],
  ]) {
    console.log(`start ${name}`, show(startIndices(p, refs)));
  }
  const june = resolveMonth(C, "Jun", defaultPlan(C, "Jun"));
  console.log("C lipanj zadano", show({ ...june, state: june.state }));
}
