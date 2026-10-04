//! Profili A/B/C (DEMO §5): isti zbroj 250, drukčije raspoređene snage.
//! Privremeno konstante; u planu jezgre prelaze u content/hotel i računaju se iz stvarnog sloja.

use crate::index::IndexSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Profile {
    A,
    B,
    C,
}

impl Profile {
    pub const ALL: [Profile; 3] = [Profile::A, Profile::B, Profile::C];

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "A" => Some(Profile::A),
            "B" => Some(Profile::B),
            "C" => Some(Profile::C),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Profile::A => "A",
            Profile::B => "B",
            Profile::C => "C",
        }
    }

    /// Ciljane početne vrijednosti u stotinkama, redoslijed G, R, Z, O, F.
    pub fn starting_indices(self) -> IndexSet {
        let values = match self {
            Profile::A => [6_000, 7_500, 3_000, 5_000, 3_500],
            Profile::B => [4_500, 3_500, 5_500, 4_000, 7_500],
            Profile::C => [5_000, 5_000, 5_000, 5_000, 5_000],
        };
        IndexSet::from_hundredths(values).expect("profili su u rasponu 0..=10_000")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Index;

    #[test]
    fn every_profile_totals_250() {
        for p in Profile::ALL {
            assert_eq!(
                p.starting_indices().total_hundredths(),
                25_000,
                "profil {}",
                p.code()
            );
        }
    }

    #[test]
    fn profile_c_is_fifty_everywhere() {
        let c = Profile::C.starting_indices();
        assert!(c.iter().all(|(_, v)| v.hundredths() == 5_000));
    }

    #[test]
    fn profiles_match_demo_section_5() {
        let a = Profile::A.starting_indices();
        assert_eq!(a.get(Index::Reviews).hundredths(), 7_500);
        assert_eq!(a.get(Index::Staff).hundredths(), 3_000);
        let b = Profile::B.starting_indices();
        assert_eq!(b.get(Index::Finance).hundredths(), 7_500);
        assert_eq!(b.get(Index::Reviews).hundredths(), 3_500);
    }

    #[test]
    fn from_code_accepts_only_known_uppercase_codes() {
        assert_eq!(Profile::from_code("A"), Some(Profile::A));
        assert_eq!(Profile::from_code("C"), Some(Profile::C));
        assert_eq!(Profile::from_code("a"), None);
        assert_eq!(Profile::from_code("D"), None);
        assert_eq!(Profile::from_code(""), None);
    }
}
