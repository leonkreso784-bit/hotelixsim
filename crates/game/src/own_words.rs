//! Pravilo „vlastiti odgovor vrijedi više od klika" (ADR-047) i zaštita od zalijepljenih odgovora (kvar B13):
//! konstante, prepoznavanje prepisanog ili zalijepljenog teksta i tekstovi za studenta i mentora.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::DebriefSituation;

/// Najviša kvaliteta klika na ponuđeni odgovor u situaciji s likom (ADR-047 t. 1).
pub const CLICK_MAX_QUALITY: u8 = 4;
/// Kvaliteta neutralne presude kad AI ne stigne, uz `OwnWords` (ADR-047 t. 4); uz `Classic` je 3.
pub const NEUTRAL_OWN_WORDS_QUALITY: u8 = 4;
/// Kvaliteta zalijepljenih ponuđenih odgovora (B13): neutralna, kao šutnja.
pub const PASTED_QUALITY: u8 = 3;
/// 🟨 Prag prepisivanja: toliki dio riječi (bp) s obje strane mora biti zajednički.
pub const COPY_MATCH_BP: u32 = 7_000;

/// Obrazloženje presude za zalijepljene odgovore (vidi ga student i mentor).
pub const PASTED_RATIONALE: &str = "Zalijepio si ponuđene odgovore, a to nije tvoja odluka. Napiši svojim riječima što ćeš učiniti.";
/// Činjenica za mentora kad je ocjena klika ili prepisanog odgovora ograničena.
pub const CAPPED_FACT: &str = "Ocjena je ograničena na 4/5 jer je student kliknuo (ili prepisao) ponuđeni odgovor, a u ovoj situaciji je mogao odgovoriti svojim riječima; za 5/5 treba vlastiti odgovor.";
/// Činjenica za mentora kad je student zalijepio ponuđene odgovore.
pub const PASTED_FACT: &str = "Student je zalijepio ponuđene odgovore umjesto da napiše vlastitu odluku, pa je dobio srednju ocjenu 3/5 bez učinka odgovora.";

/// Riječi teksta: mala slova, č/ć→c, đ→d, š→s, ž→z; sve osim slova i brojki je razmak.
pub fn words(text: &str) -> Vec<String> {
    let norm: String = text
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'č' | 'ć' => 'c',
            'đ' => 'd',
            'š' => 's',
            'ž' => 'z',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect();
    norm.split_whitespace().map(str::to_owned).collect()
}

fn counts(ws: &[String]) -> BTreeMap<&str, u32> {
    let mut m = BTreeMap::new();
    for w in ws {
        *m.entry(w.as_str()).or_insert(0) += 1;
    }
    m
}

/// Broj zajedničkih riječi (multiskup).
fn common(a: &BTreeMap<&str, u32>, b: &BTreeMap<&str, u32>) -> u32 {
    a.iter()
        .map(|(w, &n)| n.min(b.get(w).copied().unwrap_or(0)))
        .sum()
}

fn len(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Redni broj ponuđenog odgovora koji je student prepisao: zajedničke riječi (multiskup) čine barem `COPY_MATCH_BP`
/// riječi teksta i riječi oznake; najveći udio pobjeđuje, kod jednakih manji indeks. Prazan tekst ili oznaka: `None`.
pub fn copied_answer(text: &str, labels: &[&str]) -> Option<usize> {
    let tw = words(text);
    if tw.is_empty() {
        return None;
    }
    let tc = counts(&tw);
    let mut best: Option<(u64, usize)> = None;
    for (i, label) in labels.iter().enumerate() {
        let lw = words(label);
        if lw.is_empty() {
            continue;
        }
        let c = u64::from(common(&tc, &counts(&lw)));
        let max = u64::from(len(tw.len().max(lw.len())));
        if c * 10_000 < u64::from(COPY_MATCH_BP) * max {
            continue;
        }
        let share = c * 10_000 / max;
        if best.is_none_or(|(s, _)| share > s) {
            best = Some((share, i));
        }
    }
    best.map(|(_, i)| i)
}

/// Jedinice usporedbe za zalijepljeno: parovi susjednih riječi (bigrami); oznaka od samo jedne riječi nema parova,
/// pa je njezina jedinica sama riječ (najmanje razumno, 🟨).
fn units(ws: &[String]) -> Vec<String> {
    if ws.len() < 2 {
        return ws.to_vec();
    }
    ws.windows(2)
        .map(|p| format!("{} {}", p[0], p[1]))
        .collect()
}

/// Kvar B13: tekst pokriva barem dvije ponuđene oznake, svaku barem `COPY_MATCH_BP` njezinih PAROVA susjednih riječi
/// (multiskup). Zalijepljen tekst čuva redoslijed riječi oznake, vlastita preformulacija ne (pregled C1).
pub fn pasted_answers(text: &str, labels: &[&str]) -> bool {
    let tw = words(text);
    let tu = units(&tw);
    let tc = counts(&tu);
    labels
        .iter()
        .filter(|label| {
            let lu = units(&words(label));
            !lu.is_empty()
                && u64::from(common(&tc, &counts(&lu))) * 10_000
                    >= u64::from(COPY_MATCH_BP) * u64::from(len(lu.len()))
        })
        .count()
        >= 2
}

/// Tekst neutralne ocjene za mentora (AI nije stigao).
pub fn neutral_fact(quality: u8) -> String {
    let grade = if quality == 3 {
        "srednju ocjenu"
    } else {
        "ocjenu"
    };
    format!(
        "AI nije stigao ocijeniti odgovor, pa je dobio {grade} {quality}/5; odgovor čeka pregled."
    )
}

/// Činjenice o ocjeni situacije za mentora; svaka se šalje samo kad vrijedi (ulaz stare igre ostaje bajt-isti).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuleFacts {
    #[serde(rename = "neutralna_ocjena", skip_serializing_if = "Option::is_none")]
    pub neutral: Option<String>,
    #[serde(rename = "ogranicena_ocjena", skip_serializing_if = "Option::is_none")]
    pub capped: Option<&'static str>,
    #[serde(
        rename = "zalijepljeni_odgovori",
        skip_serializing_if = "Option::is_none"
    )]
    pub pasted: Option<&'static str>,
}

impl RuleFacts {
    pub fn of(s: &DebriefSituation) -> RuleFacts {
        RuleFacts {
            neutral: s.neutral.then(|| neutral_fact(s.quality)),
            capped: s.capped.then_some(CAPPED_FACT),
            pasted: s.pasted.then_some(PASTED_FACT),
        }
    }
}
