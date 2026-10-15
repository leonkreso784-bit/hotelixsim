//! Oblik AI mentora kruga (uputa `mentor.v4`, rezerva `fallback_debrief`): četiri dijela, podnaslov u svom retku.
const PARTS: [&str; 3] = ["Što je išlo dobro", "Što nije valjalo", "Zašto"];
/// Četvrti podnaslov nosi krug („Što probati u kolovozu", „… sljedeći tjedan", „… sljedeći put").
const LAST: &str = "Što probati";

/// Ima li tekst sva četiri dijela; radnik inače sprema rezervu, da nepotpun tekst ne ostane zapisan kao AI-jev.
pub fn debrief_has_parts(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    PARTS.iter().all(|p| lines.contains(p))
        && lines
            .iter()
            .any(|l| l.starts_with(LAST) && l.split_whitespace().count() <= 5)
}
