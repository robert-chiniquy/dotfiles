//! Seeded randomness for run order and frontier samples. The seed is written
//! into the round header, and the same seed must give the same run sheet on
//! every build of this tool, so the generator is defined here rather than
//! taken from a library whose stream may change between versions.
//!
//! Generator: SplitMix64. Steele, G. L., Lea, D., Flood, C. H., "Fast
//! Splittable Pseudorandom Number Generators", OOPSLA 2014, 453-472.
//! Shuffle: Durstenfeld's form of the Fisher-Yates shuffle (Knuth, The Art of
//! Computer Programming, vol. 2, 3rd ed., 1997, section 3.4.2, Algorithm P).

mod tests;

pub(crate) struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub(crate) fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n`, `n` > 0. Draws at or above the largest multiple of
    /// `n` are rejected so that no residue is favored.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        let n = n as u64;
        let limit = u64::MAX - u64::MAX % n;
        loop {
            let x = self.next_u64();
            if x < limit {
                return (x % n) as usize;
            }
        }
    }

    pub(crate) fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }

    /// `k` items drawn without replacement, in draw order: the first `k`
    /// steps of the same shuffle. All of them when `k` ≥ the length.
    pub(crate) fn sample<T>(&mut self, mut items: Vec<T>, k: usize) -> Vec<T> {
        let k = k.min(items.len());
        for i in 0..k {
            let j = i + self.below(items.len() - i);
            items.swap(i, j);
        }
        items.truncate(k);
        items
    }
}
