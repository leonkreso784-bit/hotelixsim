use engine::{Rng, label};
use sim_hotel_v2::streams::{Purpose, stream};

#[test]
fn every_purpose_has_its_own_stable_v2_label() {
    let labels: Vec<&str> = Purpose::ALL.iter().map(|p| p.label()).collect();
    assert_eq!(
        labels,
        [
            "v2:odlasci",
            "v2:kvarovi",
            "v2:recenzije",
            "v2:burza-rada",
            "v2:situacije"
        ]
    );
}

#[test]
fn stream_is_derive_over_purpose_period_slot_and_key() {
    let mut a = stream(7, Purpose::Departures, 3, 2, 99);
    let mut b = Rng::derive(7, &[label("v2:odlasci"), 3, 2, 99]);
    assert_eq!(a.next_u64(), b.next_u64());
}

#[test]
fn streams_differ_when_any_input_differs() {
    let first = |r: &mut Rng| r.next_u64();
    let base = first(&mut stream(7, Purpose::Departures, 3, 2, 99));
    for other in [
        stream(8, Purpose::Departures, 3, 2, 99),
        stream(7, Purpose::Reviews, 3, 2, 99),
        stream(7, Purpose::Departures, 4, 2, 99),
        stream(7, Purpose::Departures, 3, 1, 99),
        stream(7, Purpose::Departures, 3, 2, 98),
    ] {
        let mut o = other;
        assert_ne!(first(&mut o), base);
    }
}

/// Tokovi odlazaka i kvarova daju iste brojeve kao prije toka situacija (J-2): vrijednosti su izmjerene na 100d2d1.
#[test]
fn departure_and_breakdown_streams_are_unchanged_by_situations() {
    let mut got = Vec::new();
    for p in [Purpose::Departures, Purpose::Breakdowns] {
        for (seed, period, slot, key) in [(1_u64, 0_u8, 1_u8, 0_u64), (42, 16, 4, label("rooms"))] {
            let mut r = stream(seed, p, period, slot, key);
            got.push((r.next_u64(), r.next_u64()));
        }
    }
    assert_eq!(
        got,
        [
            (4_846_013_232_754_066_818, 7_219_567_834_123_768_461),
            (6_024_826_633_194_157_566, 11_805_059_198_328_912_699),
            (13_946_364_245_300_130_179, 11_413_433_627_109_248_004),
            (10_315_926_080_618_713_976, 4_758_768_568_458_004_081),
        ]
    );
}

#[test]
fn situations_stream_is_its_own() {
    let mut a = stream(7, Purpose::Situations, 3, 2, 99);
    let mut b = Rng::derive(7, &[label("v2:situacije"), 3, 2, 99]);
    assert_eq!(a.next_u64(), b.next_u64());
    for other in [Purpose::Departures, Purpose::Breakdowns] {
        assert_ne!(
            stream(7, Purpose::Situations, 3, 2, 99).next_u64(),
            stream(7, other, 3, 2, 99).next_u64()
        );
    }
}
