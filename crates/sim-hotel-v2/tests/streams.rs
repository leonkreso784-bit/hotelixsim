use engine::{Rng, label};
use sim_hotel_v2::streams::{Purpose, stream};

#[test]
fn every_purpose_has_its_own_stable_v2_label() {
    let labels: Vec<&str> = Purpose::ALL.iter().map(|p| p.label()).collect();
    assert_eq!(
        labels,
        ["v2:odlasci", "v2:kvarovi", "v2:recenzije", "v2:burza-rada"]
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
