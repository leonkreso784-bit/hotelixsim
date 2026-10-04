//! Kocka: SplitMix64 (Vigna), neovisni tokovi iz sjemena. Ne ovisi o vanjskoj biblioteci, jer `rand`
//! dopušta da nova manja verzija promijeni niz brojeva (istrazivanje/RUST_ALATI_2026-10.md).
//! Sva `wrapping_*` aritmetika ovdje je namjerna: dio je definicije algoritma.

const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn finalize(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn mix64(x: u64) -> u64 {
    finalize(x.wrapping_add(GOLDEN_GAMMA))
}

/// FNV-1a 64 nad UTF-8 bajtovima; služi za stabilne oznake tokova (npr. `label("trigger:T1")`).
pub fn label(s: &str) -> u64 {
    s.bytes().fold(FNV_OFFSET, |h, b| (h ^ u64::from(b)).wrapping_mul(FNV_PRIME))
}

/// Ponovljiva kocka. Isto sjeme i iste oznake daju isti niz na svakom računalu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Neovisan tok izveden iz sjemena igre i oznaka, npr. `(runda, hotel, label("trigger:T1"))`.
    pub fn derive(seed: u64, labels: &[u64]) -> Self {
        let state = labels.iter().fold(mix64(seed), |s, l| mix64(s ^ l));
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        finalize(self.state)
    }

    /// Jednoliko iz `[lo, hi)`, bez pristranosti (odbacivanje ispod praga).
    pub fn range(&mut self, lo: u64, hi: u64) -> u64 {
        assert!(lo < hi, "range: potrebno lo < hi (lo={lo}, hi={hi})");
        let span = hi.checked_sub(lo).expect("lo < hi");
        let threshold = span.wrapping_neg().checked_rem(span).expect("span > 0");
        loop {
            let r = self.next_u64();
            if r >= threshold {
                let offset = r.checked_rem(span).expect("span > 0");
                return lo.checked_add(offset).expect("lo + offset < hi");
            }
        }
    }

    /// Vjerojatnost u baznim bodovima: 0 = nikad, 10_000 = uvijek.
    pub fn chance_bp(&mut self, bp: u32) -> bool {
        assert!(bp <= 10_000, "chance_bp: potrebno bp <= 10000 (bp={bp})");
        self.range(0, 10_000) < u64::from(bp)
    }
}
