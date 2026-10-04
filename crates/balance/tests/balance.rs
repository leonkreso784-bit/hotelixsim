#[test]
fn report_runs_on_repo_content_and_is_reproducible() {
    let c = content::load(&content::repo_dir()).unwrap();
    let r = balance::run(&c, &balance::Options { seeds: 5 }).unwrap();
    assert_eq!(r.starts.len(), 3);
    assert_eq!(r.medians.len(), 12);
    assert_eq!(r.probes.len(), 12);
    assert!(r.reproducible);
    let r2 = balance::run(&c, &balance::Options { seeds: 5 }).unwrap();
    assert_eq!(balance::render(&r), balance::render(&r2));
}

#[test]
fn failures_name_the_broken_check() {
    let mut c = content::load(&content::repo_dir()).unwrap();
    c.catalog
        .profiles
        .get_mut(&sim_hotel::Profile::C)
        .unwrap()
        .satisfaction_pts = 9_000;
    let r = balance::run(&c, &balance::Options { seeds: 2 }).unwrap();
    let f = balance::failures(&r, &c.catalog);
    assert!(f.iter().any(|m| m.contains("§11.1")), "{f:?}");
}

#[test]
fn s3_gop_gain_is_reported_for_august_and_june() {
    let c = content::load(&content::repo_dir()).unwrap();
    let r = balance::run(&c, &balance::Options { seeds: 1 }).unwrap();
    let g = r.s3_gop_gain_cents.expect("S3 ima grupu u kolovozu");
    assert_ne!(g.august_cents, 0);
    assert_ne!(g.june_cents, 0);
    let text = balance::render(&r);
    assert!(text.contains("S3 grupa, dobit GOP-a"), "{text}");
}

/// ADR-014 t. 1: u rasprodanom kolovozu ista ponuda donosi manje nego u lipnju; provjera to čuva.
#[test]
fn s3_check_passes_on_repo_content() {
    let c = content::load(&content::repo_dir()).unwrap();
    let r = balance::run(&c, &balance::Options { seeds: 1 }).unwrap();
    let g = r.s3_gop_gain_cents.expect("S3 ima grupu u kolovozu");
    assert!(g.august_cents < g.june_cents, "{g:?}");
    assert!(r.august_displacement);
    let f = balance::failures(&r, &c.catalog);
    assert!(!f.iter().any(|m| m.contains("§7 S3")), "{f:?}");
}

/// Stanje prije kalibracije (kolovoz 95,5 %, nije rasprodan): lekcija je obrnuta i provjera mora pasti.
#[test]
fn s3_check_fires_when_august_is_not_sold_out() {
    let mut c = content::load(&content::repo_dir()).unwrap();
    c.catalog
        .categories
        .get_mut(&sim_hotel::Category::Four)
        .unwrap()
        .base_occupancy_bp
        .0[2] = 9_550;
    let r = balance::run(&c, &balance::Options { seeds: 1 }).unwrap();
    let g = r.s3_gop_gain_cents.expect("S3 ima grupu u kolovozu");
    assert!(g.august_cents >= g.june_cents, "{g:?}");
    let f = balance::failures(&r, &c.catalog);
    assert!(f.iter().any(|m| m.contains("§7 S3")), "{f:?}");
}
