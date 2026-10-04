use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut seeds: u64 = 1000;
    let mut dir: PathBuf = PathBuf::from("content/hotel");
    let mut check = false;
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
            _ => return usage(&format!("nepoznati argument {a}")),
        }
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
    let failures = balance::failures(&report, &content.catalog);
    if failures.is_empty() {
        println!("Sve provjere prolaze.");
    } else {
        println!("Provjere koje padaju ({}):", failures.len());
        for f in &failures {
            println!("  - {f}");
        }
        if check {
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("{msg}\nuporaba: balance [--seeds N] [--content DIR] [--check]");
    ExitCode::from(2)
}
