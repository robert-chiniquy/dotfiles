#![cfg(test)]

use super::SplitMix64;

#[test]
fn same_seed_gives_same_stream() {
    let mut a = SplitMix64::new(42);
    let mut b = SplitMix64::new(42);
    for _ in 0..100 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn stream_matches_reference_splitmix64() {
    // First output of SplitMix64 seeded with 0, from the published reference
    // implementation; a change here changes every recorded run sheet.
    assert_eq!(SplitMix64::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
}

#[test]
fn sample_takes_everything_when_k_exceeds_length() {
    let mut r = SplitMix64::new(7);
    let mut got = r.sample(vec![1, 2, 3], 14);
    got.sort_unstable();
    assert_eq!(got, vec![1, 2, 3]);
}

#[test]
fn shuffle_keeps_every_item() {
    let mut r = SplitMix64::new(9);
    let mut v: Vec<u32> = (0..50).collect();
    r.shuffle(&mut v);
    let mut sorted = v.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..50).collect::<Vec<_>>());
    assert_ne!(v, sorted);
}
