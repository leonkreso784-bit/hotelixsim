use engine::{Rng, label};

#[test]
fn next_u64_matches_reference_seed_1234567() {
    let mut rng = Rng::from_seed(1_234_567);
    let got: Vec<u64> = (0..5).map(|_| rng.next_u64()).collect();
    assert_eq!(
        got,
        [
            6_457_827_717_110_365_317,
            3_203_168_211_198_807_973,
            9_817_491_932_198_370_423,
            4_593_380_528_125_082_431,
            16_408_922_859_458_223_821,
        ]
    );
}

#[test]
fn next_u64_matches_reference_seed_0() {
    let mut rng = Rng::from_seed(0);
    let got: Vec<u64> = (0..3).map(|_| rng.next_u64()).collect();
    assert_eq!(
        got,
        [
            16_294_208_416_658_607_535,
            7_960_286_522_194_355_700,
            487_617_019_471_545_679
        ]
    );
}

#[test]
fn label_is_fnv1a_64() {
    assert_eq!(label(""), 14_695_981_039_346_656_037);
    assert_eq!(label("trigger:T1"), 17_546_483_036_339_428_452);
}

#[test]
fn derive_matches_reference() {
    let mut rng = Rng::derive(42, &[3, 7, label("trigger:T1")]);
    let got: Vec<u64> = (0..3).map(|_| rng.next_u64()).collect();
    assert_eq!(
        got,
        [
            15_554_128_182_792_330_207,
            9_332_619_251_129_265_620,
            12_170_803_838_678_117_234
        ]
    );
}

#[test]
fn derive_streams_are_independent() {
    // Dodavanje novog bacanja u jednom toku ne smije pomaknuti drugi tok (ARHITEKTURA §7.2).
    let mut a = Rng::derive(42, &[3, 7, label("trigger:T1")]);
    let mut b = Rng::derive(42, &[3, 7, label("trigger:T2")]);
    let _ = b.next_u64();
    let _ = b.next_u64();
    assert_eq!(a.next_u64(), 15_554_128_182_792_330_207);
}

#[test]
fn range_matches_reference() {
    let mut rng = Rng::from_seed(99);
    let got: Vec<u64> = (0..5).map(|_| rng.range(0, 10_000)).collect();
    assert_eq!(got, [403, 4_564, 9_627, 807, 5_076]);

    let mut rng = Rng::from_seed(99);
    let got: Vec<u64> = (0..5).map(|_| rng.range(7, 13)).collect();
    assert_eq!(got, [12, 7, 8, 12, 11]);
}

#[test]
fn range_stays_in_bounds() {
    let mut rng = Rng::from_seed(5);
    for _ in 0..10_000 {
        let x = rng.range(10, 13);
        assert!((10..13).contains(&x), "izvan raspona: {x}");
    }
}

#[test]
fn chance_bp_extremes() {
    let mut rng = Rng::from_seed(7);
    assert!((0..1_000).all(|_| !rng.chance_bp(0)));
    assert!((0..1_000).all(|_| rng.chance_bp(10_000)));
}

#[test]
#[should_panic(expected = "lo < hi")]
fn range_rejects_empty() {
    Rng::from_seed(1).range(5, 5);
}

#[test]
#[should_panic(expected = "bp <= 10000")]
fn chance_bp_rejects_over_100_percent() {
    Rng::from_seed(1).chance_bp(10_001);
}

#[test]
fn derived_streams_differ_by_label_order_and_seed() {
    let t1 = label("trigger:T1");
    let t2 = label("trigger:T2");
    let first = |mut r: Rng| r.next_u64();
    assert_ne!(
        first(Rng::derive(7, &[1, t1])),
        first(Rng::derive(7, &[1, t2]))
    );
    assert_ne!(
        first(Rng::derive(7, &[1, t1])),
        first(Rng::derive(7, &[t1, 1]))
    );
    assert_ne!(
        first(Rng::derive(7, &[1, t1])),
        first(Rng::derive(8, &[1, t1]))
    );
}

#[test]
fn range_near_2_pow_63_rejects_draws_and_returns_modulo_of_accepted_ones() {
    // raspon blizu 2^63 odbacuje gotovo polovicu izvlačenja, pa se grana odbacivanja stvarno izvršava
    let mut rng = Rng::from_seed(42);
    let mut raw = Rng::from_seed(42);
    let hi = (1_u64 << 63) + 1;
    let threshold = hi.wrapping_neg() % hi;
    let mut consumed = 0_u32;
    for _ in 0..1_000 {
        let v = rng.range(0, hi);
        assert!(v < hi);
        let expected = loop {
            let r = raw.next_u64();
            consumed += 1;
            if r >= threshold {
                break r % hi;
            }
        };
        assert_eq!(v, expected);
    }
    assert!(consumed > 1_000, "nijedno izvlačenje nije odbačeno");
}
