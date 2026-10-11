use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sim_hotel_v2::PeriodKind;

fn main() -> ExitCode {
    let mut seeds: u64 = 1000;
    let mut dir: PathBuf = PathBuf::from("content/hotel");
    let mut check = false;
    let mut v2 = false;
    let mut kind = PeriodKind::Week;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seeds" => match args.next().and_then(|v| v.parse().ok()) {
                Some(0) => return usage("--seeds traži barem 1"),
                Some(n) => seeds = n,
                None => return usage("--seeds traži cijeli broj"),
            },
            "--content" => match args.next() {
                Some(d) => dir = PathBuf::from(d),
                None => return usage("--content traži direktorij"),
            },
            "--check" => check = true,
            "--model" => match args.next().as_deref() {
                Some("v1") => v2 = false,
                Some("v2") => v2 = true,
                _ => return usage("--model traži v1 ili v2"),
            },
            "--round" => match args.next().and_then(|v| PeriodKind::from_code(&v)) {
                Some(k) => kind = k,
                None => return usage("--round traži month ili week"),
            },
            _ => return usage(&format!("nepoznati argument {a}")),
        }
    }
    if v2 {
        return run_v2(&dir, seeds, kind, check);
    }
    let content = match content::load(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sadržaj se ne može učitati: {e:?}");
            return ExitCode::from(2);
        }
    };
    let report = match balance::run(&content, &balance::Options { seeds }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("simulacija nije uspjela: {e:?}");
            return ExitCode::from(2);
        }
    };
    print!("{}", balance::render(&report));
    finish(&balance::failures(&report, &content.catalog), check)
}

/// Balans v2 (`--model v2`): sadržaj iz DIR i DIR/v2, krugovi tjedni ili mjeseci.
fn run_v2(dir: &Path, seeds: u64, kind: PeriodKind, check: bool) -> ExitCode {
    let content = match content::Snapshot::read_v2(dir).and_then(|s| s.build()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sadržaj v2 se ne može učitati: {e:?}");
            return ExitCode::from(2);
        }
    };
    let report = match balance::v2::run_v2(&content, &balance::v2::OptionsV2 { seeds, kind }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("simulacija v2 nije uspjela: {e:?}");
            return ExitCode::from(2);
        }
    };
    println!("Model v2, krugovi: {}", kind.code());
    print!("{}", balance::v2::render_v2(&report));
    finish(&balance::v2::failures_v2(&report), check)
}

/// Ispis provjera koje padaju; uz `--check` izlazni kod 1.
fn finish(failures: &[String], check: bool) -> ExitCode {
    if failures.is_empty() {
        println!("Sve provjere prolaze.");
    } else {
        println!("Provjere koje padaju ({}):", failures.len());
        for f in failures {
            println!("  - {f}");
        }
        if check {
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}

fn usage(msg: &str) -> ExitCode {
    eprintln!(
        "{msg}\nuporaba: balance [--seeds N] [--content DIR] [--check] [--model v1|v2] [--round month|week]"
    );
    ExitCode::from(2)
}
