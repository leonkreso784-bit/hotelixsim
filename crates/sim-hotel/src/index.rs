//! Pet indeksa 0–100 (DEMO §4). U jezgri se drže u stotinkama: 0..=10_000.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Index {
    Guests,
    Reviews,
    Staff,
    Operations,
    Finance,
}

impl Index {
    pub const ALL: [Index; 5] = [
        Index::Guests,
        Index::Reviews,
        Index::Staff,
        Index::Operations,
        Index::Finance,
    ];

    /// Stabilan kod za API i prijevode; nikad se ne mijenja.
    pub fn code(self) -> &'static str {
        match self {
            Index::Guests => "G",
            Index::Reviews => "R",
            Index::Staff => "Z",
            Index::Operations => "O",
            Index::Finance => "F",
        }
    }

    fn position(self) -> usize {
        match self {
            Index::Guests => 0,
            Index::Reviews => 1,
            Index::Staff => 2,
            Index::Operations => 3,
            Index::Finance => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexValue(u16);

impl IndexValue {
    pub const MAX_HUNDREDTHS: u16 = 10_000;

    pub fn new(hundredths: u16) -> Option<Self> {
        (hundredths <= Self::MAX_HUNDREDTHS).then_some(Self(hundredths))
    }

    pub fn hundredths(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexSet([IndexValue; 5]);

impl IndexSet {
    /// Vrijednosti u redoslijedu `Index::ALL` (G, R, Z, O, F).
    pub fn from_hundredths(values: [u16; 5]) -> Option<Self> {
        let mut out = [IndexValue(0); 5];
        for (slot, v) in out.iter_mut().zip(values) {
            *slot = IndexValue::new(v)?;
        }
        Some(Self(out))
    }

    pub fn get(&self, index: Index) -> IndexValue {
        self.0[index.position()]
    }

    pub fn iter(&self) -> impl Iterator<Item = (Index, IndexValue)> + '_ {
        Index::ALL.into_iter().map(|i| (i, self.get(i)))
    }

    pub fn total_hundredths(&self) -> u32 {
        self.0.iter().map(|v| u32::from(v.0)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_value_rejects_over_100() {
        assert!(IndexValue::new(10_000).is_some());
        assert!(IndexValue::new(10_001).is_none());
    }

    #[test]
    fn index_codes_are_stable_and_ordered() {
        let codes: Vec<&str> = Index::ALL.iter().map(|i| i.code()).collect();
        assert_eq!(codes, ["G", "R", "Z", "O", "F"]);
    }

    #[test]
    fn index_set_rejects_invalid_value() {
        assert!(IndexSet::from_hundredths([5_000, 5_000, 5_000, 5_000, 10_001]).is_none());
    }
}
