//! Mjerilo AI presuda (DEMO §11 t. 7): koliko se AI slaže s Leonovim ocjenama (±1) na S1 i S2.
//! Logika je ovdje, `bin/mjerilo.rs` samo zove `run`. Testovi ne zovu mrežu.

use app::{AiError, AiPort};
use content::{Content, Snapshot, repo_dir};
use game::command::new_game;
use game::{AssessInput, Game, GameId, GuestId, HotelId, OfferedForJudge, Recorded};
use serde::Deserialize;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Najmanji udio slaganja (unutar ±1) u promilima: 800 = 80 %.
pub const THRESHOLD_PERMILLE: u32 = 800;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub text: String,
    pub leon_quality: Option<u8>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub ai_quality: u8,
    pub leon_quality: u8,
}

/// Koliko slučajeva Leon još nije ocijenio.
pub fn ungraded(cases: &[Case]) -> usize {
    cases.iter().filter(|c| c.leon_quality.is_none()).count()
}

/// Udio slaganja unutar ±1, u promilima (cjelobrojno; 800 = 80 %). Prazan ulaz daje 0.
pub fn agreement_permille(outcomes: &[Outcome]) -> u32 {
    let total = u32::try_from(outcomes.len()).unwrap_or(u32::MAX);
    if total == 0 {
        return 0;
    }
    let close = outcomes
        .iter()
        .filter(|o| o.ai_quality.abs_diff(o.leon_quality) <= 1)
        .count();
    u32::try_from(close)
        .unwrap_or(u32::MAX)
        .saturating_mul(1000)
        / total
}

/// Izlazni kod mjerila za jednu šifru: 2 = mjerenje nije moguće (barem jedan poziv je pao na AiError),
/// 1 = ne slaže se (ispod praga), 0 = slaže se. Nevaljana presuda NIJE greška poziva, nego neslaganje.
pub fn verdict_exit(errors: usize, outcomes: &[Outcome]) -> u8 {
    if errors > 0 {
        2
    } else if agreement_permille(outcomes) >= THRESHOLD_PERMILLE {
        0
    } else {
        1
    }
}

/// Odbij ocjene izvan 1..=5 (poruka s rednim brojem slučaja).
pub fn validate_cases(code: &str, cases: &[Case]) -> Result<(), String> {
    for (i, c) in cases.iter().enumerate() {
        if let Some(q) = c.leon_quality
            && !(1..=5).contains(&q)
        {
            return Err(format!(
                "{code}, slučaj {}: leon_quality {q} nije u rasponu 1..=5",
                i + 1
            ));
        }
    }
    Ok(())
}

fn build_game() -> Result<Game, String> {
    let snap = Snapshot::read(&repo_dir()).map_err(|e| format!("sadržaj: {e:?}"))?;
    let hotels = [11, 12, 13, 14].map(|n| HotelId(Uuid::from_u128(n)));
    let guest = GuestId(Uuid::from_u128(99));
    let events: Vec<Recorded> = new_game(GameId(Uuid::from_u128(1)), 42, &snap, guest, hotels)
        .map_err(|e| format!("igra: {e:?}"))?
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect();
    Game::fold(&events).map_err(|e| format!("igra: {e:?}"))
}

/// Ulaz kakav bi vidio sudac u igri, ali bez razgovora (studentov tekst je jedini ulaz).
fn assess_input(content: &Content, code: &str, text: &str) -> Result<AssessInput, String> {
    let t = content
        .situations
        .iter()
        .find(|t| t.code == code)
        .ok_or_else(|| format!("nepoznata situacija {code}"))?;
    let ai = content
        .ai
        .get(code)
        .filter(|a| a.free_text)
        .ok_or_else(|| format!("{code} nema slobodni odgovor"))?;
    let offered = t
        .answers
        .iter()
        .zip(0u8..)
        .map(|(a, index)| OfferedForJudge {
            index,
            label: a.label.clone(),
            criteria: ai
                .criteria
                .get(usize::from(index))
                .cloned()
                .unwrap_or_default(),
        })
        .collect();
    Ok(AssessInput {
        situation: code.to_owned(),
        title: t.title.clone(),
        rubric: ai.rubric.clone(),
        character: ai.character.as_ref().map(|c| c.name.clone()),
        offered,
        transcript: Vec::new(),
        response: text.to_owned(),
    })
}

fn load_cases(code: &str) -> Result<Vec<Case>, String> {
    let path = repo_dir().join(format!("mjerilo/{code}.json"));
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let cases: Vec<Case> =
        serde_json::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
    validate_cases(code, &cases)?;
    Ok(cases)
}

fn clip(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .take(60)
        .collect()
}

/// Pokreće mjerilo za zadane šifre; vraća izlazni kod (0 slaže se, 1 ne slaže se, 2 mjerenje nije moguće).
pub async fn run(codes: &[String]) -> i32 {
    if codes.is_empty() {
        eprintln!("Upotreba: mjerilo S1 [S2]");
        return 2;
    }
    let mut loaded = Vec::new();
    for code in codes {
        match load_cases(code) {
            Ok(cases) => loaded.push((code.clone(), cases)),
            Err(e) => {
                eprintln!("Ne mogu učitati odgovore za {code}: {e}");
                return 2;
            }
        }
    }
    let mut blocked = false;
    for (code, cases) in &loaded {
        let n = ungraded(cases);
        if n > 0 {
            println!(
                "Leon još nije ocijenio {n} od {} odgovora za {code} — mjerenje nije moguće",
                cases.len()
            );
            blocked = true;
        }
    }
    if blocked {
        return 2;
    }
    let Some(key) = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
    else {
        eprintln!("ANTHROPIC_API_KEY nije postavljen — mjerenje nije moguće");
        return 2;
    };
    let content = match content::load(&repo_dir()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Ne mogu učitati sadržaj: {e:?}");
            return 2;
        }
    };
    let game = match build_game() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let port = crate::Anthropic::new(key, crate::Models::from_env());
    let mut exit = 0u8;
    let mut errors = 0usize;
    let mut first_error: Option<String> = None;
    let mut durations: Vec<Duration> = Vec::new();
    for (code, cases) in &loaded {
        println!(
            "
{code}
  #  Leon  AI  razlika  sekunde  tekst"
        );
        let mut outcomes = Vec::new();
        let mut code_errors = 0usize;
        for (i, case) in cases.iter().enumerate() {
            let leon = case.leon_quality.unwrap_or(1);
            let n = i + 1;
            let text = clip(&case.text);
            let input = match assess_input(&content, code, &case.text) {
                Ok(input) => input,
                Err(e) => {
                    println!(
                        "{n:>3}  {leon:>4}   —        —        —  {text}
       greška: {e}"
                    );
                    code_errors += 1;
                    first_error.get_or_insert(e);
                    continue;
                }
            };
            let started = Instant::now();
            let called: Result<_, AiError> = port.assess(input).await;
            let took = started.elapsed();
            durations.push(took);
            let secs = took.as_secs_f64();
            let verdict = match called {
                Ok(out) => game
                    .validate_verdict(code, out.value)
                    .map_err(|e| format!("nevaljana presuda: {e:?}")),
                Err(e) => {
                    let reason = format!("AI nije odgovorio: {e:?}");
                    println!(
                        "{n:>3}  {leon:>4}   —        —  {secs:>7.1}  {text}
       greška: {reason}"
                    );
                    code_errors += 1;
                    first_error.get_or_insert(reason);
                    continue;
                }
            };
            match verdict {
                Ok(v) => {
                    let diff = i16::from(v.quality) - i16::from(leon);
                    println!(
                        "{n:>3}  {leon:>4}  {:>2}  {diff:>+7}  {secs:>7.1}  {text}",
                        v.quality
                    );
                    outcomes.push(Outcome {
                        ai_quality: v.quality,
                        leon_quality: leon,
                    });
                }
                Err(reason) => {
                    println!(
                        "{n:>3}  {leon:>4}   —        —  {secs:>7.1}  {text}
       neslaganje: {reason}"
                    );
                    // Nevaljana presuda broji se kao neslaganje: ocjena najdalja od Leonove.
                    outcomes.push(Outcome {
                        ai_quality: if leon >= 3 { 1 } else { 5 },
                        leon_quality: leon,
                    });
                }
            }
        }
        errors += code_errors;
        let p = agreement_permille(&outcomes);
        let code_exit = verdict_exit(code_errors, &outcomes);
        exit = exit.max(code_exit);
        println!(
            "Slaganje {code}: {}.{} % (traži se najmanje {} %) — {}",
            p / 10,
            p % 10,
            THRESHOLD_PERMILLE / 10,
            match code_exit {
                0 => "ZADOVOLJAVA",
                1 => "NE ZADOVOLJAVA",
                _ => "NEPOUZDANO (pozivi nisu uspjeli)",
            }
        );
    }
    if !durations.is_empty() {
        let total: f64 = durations.iter().map(Duration::as_secs_f64).sum();
        let max = durations.iter().max().map_or(0.0, Duration::as_secs_f64);
        println!(
            "
Trajanje poziva: prosjek {:.1} s, najdulje {max:.1} s ({} poziva)",
            total / durations.len() as f64,
            durations.len()
        );
    }
    if errors > 0 {
        println!(
            "mjerenje nije moguće: {errors} poziva nije uspjelo (prvi razlog: {})",
            first_error.unwrap_or_default()
        );
    }
    i32::from(exit)
}
