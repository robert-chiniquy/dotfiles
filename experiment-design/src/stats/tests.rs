#![cfg(test)]

use super::{binomial, counting, frontier_sample_size, rejects, separation_p, welch, Test};

#[test]
fn three_against_three_separation_is_one_in_twenty() {
    assert_eq!(separation_p(3, 3), 1.0 / 20.0);
    let p = counting(&[6.3, 6.0, 6.6], &[5.1, 5.4, 4.9], 0.5, Test::BeatsByDelta);
    assert_eq!(p, Some(1.0 / 20.0));
}

#[test]
fn four_against_four_separation_is_one_in_seventy() {
    assert_eq!(separation_p(4, 4), 1.0 / 70.0);
    assert_eq!(separation_p(3, 4), 1.0 / 35.0);
    assert_eq!(separation_p(5, 5), 1.0 / 252.0);
}

#[test]
fn p_equal_to_alpha_rejects() {
    // 1/20 and the literal 0.05 are the same double; a strict < would fail.
    assert!(rejects(separation_p(3, 3), 0.05));
    assert!(!rejects(0.0500001, 0.05));
}

#[test]
fn counting_beats_requires_shift_by_delta() {
    // Without the shift these separate; after taking δ = 0.5 off, 5.3 < 5.4.
    let n = [5.8, 5.5, 6.1];
    let i = [5.1, 5.4, 4.9];
    assert!(counting(&n, &i, 0.0, Test::BeatsByDelta).is_some());
    assert_eq!(counting(&n, &i, 0.5, Test::BeatsByDelta), None);
}

#[test]
fn counting_not_better_adds_delta_to_incumbent() {
    // N's best 6.4 is above I's worst 6.1, but below 6.1 + 0.5.
    let n = [6.4, 6.0, 6.3];
    let i = [6.2, 6.5, 6.1];
    assert_eq!(counting(&n, &i, 0.5, Test::NotBetterByDelta), Some(0.05));
    assert_eq!(counting(&n, &i, 0.2, Test::NotBetterByDelta), None);
}

#[test]
fn counting_worse_ignores_delta() {
    let n = [5.5, 5.9, 5.6];
    let i = [6.2, 6.5, 6.1];
    assert_eq!(counting(&n, &i, 0.5, Test::Worse), Some(0.05));
    assert_eq!(counting(&n, &i, 10.0, Test::Worse), Some(0.05));
    assert_eq!(counting(&[6.4, 6.0, 6.3], &i, 0.5, Test::Worse), None);
}

#[test]
fn counting_ties_do_not_separate() {
    assert_eq!(
        counting(&[5.4, 6.0, 6.1], &[5.4, 5.0, 5.1], 0.0, Test::BeatsByDelta),
        None
    );
}

#[test]
fn welch_beats_shifts_neighbor_down_by_delta() {
    let i = [10.0, 10.2, 9.8, 10.1, 9.9, 10.0];
    let n = [11.0, 11.2, 10.8, 11.1, 10.9, 11.0];
    // Difference is 1.0: clearly beats by 0.5, clearly does not beat by 1.5.
    assert!(welch(&n, &i, 0.5, Test::BeatsByDelta).unwrap() < 0.001);
    assert!(welch(&n, &i, 1.5, Test::BeatsByDelta).unwrap() > 0.999);
}

#[test]
fn welch_not_better_and_worse_use_lower_tail() {
    let i = [10.0, 10.2, 9.8, 10.1, 9.9, 10.0];
    let n = [10.1, 10.3, 9.9, 10.2, 10.0, 10.1];
    // Difference 0.1: not better by 0.5 (rejects), not shown worse.
    assert!(welch(&n, &i, 0.5, Test::NotBetterByDelta).unwrap() < 0.001);
    assert!(welch(&n, &i, 0.5, Test::Worse).unwrap() > 0.5);
}

#[test]
fn welch_matches_reference_value() {
    // Two runs per arm with equal variance give Welch df = 2, where Student t
    // has the closed-form CDF 1/2 + t / (2 sqrt(2 + t^2)). Means 2 and 4,
    // variance 2 each, so t = -2 / sqrt(2).
    let t = -2.0 / 2f64.sqrt();
    let expected = 0.5 + t / (2.0 * (2.0 + t * t).sqrt());
    let p = welch(&[1.0, 3.0], &[3.0, 5.0], 0.0, Test::Worse).unwrap();
    assert!((p - expected).abs() < 1e-9, "p = {p}, expected {expected}");
    // The same data shifted: N must be 1.5 better, so the not-better null
    // sits 1.5 above the observed difference of -2.
    let t = (-2.0 - 1.5) / 2f64.sqrt();
    let expected = 0.5 + t / (2.0 * (2.0 + t * t).sqrt());
    let p = welch(&[1.0, 3.0], &[3.0, 5.0], 1.5, Test::NotBetterByDelta).unwrap();
    assert!((p - expected).abs() < 1e-9, "p = {p}, expected {expected}");
}

#[test]
fn welch_refuses_single_runs() {
    assert!(welch(&[1.0], &[1.0, 2.0], 0.0, Test::Worse).is_err());
}

#[test]
fn binomial_beats_tests_against_half_plus_delta() {
    // 10 of 10 wins: P(X ≥ 10 | 0.6) = 0.6^10 ≈ 0.00605.
    let p = binomial(10, 10, 0.1, Test::BeatsByDelta).unwrap();
    assert!((p - 0.6f64.powi(10)).abs() < 1e-12);
    // 8 of 10 beats 0.5 but not 0.6 + margin at α = 0.05.
    assert!(binomial(8, 10, 0.0, Test::BeatsByDelta).unwrap() < 0.06);
    assert!(binomial(8, 10, 0.1, Test::BeatsByDelta).unwrap() > 0.1);
}

#[test]
fn binomial_worse_ignores_delta_and_not_better_uses_it() {
    // 0 of 5: P(X ≤ 0 | 0.5) = 1/32; P(X ≤ 0 | 0.6) = 0.4^5.
    assert!((binomial(0, 5, 0.1, Test::Worse).unwrap() - 1.0 / 32.0).abs() < 1e-12);
    assert!((binomial(0, 5, 0.1, Test::NotBetterByDelta).unwrap() - 0.4f64.powi(5)).abs() < 1e-12);
}

#[test]
fn frontier_sample_is_14_for_q_02_and_29_for_q_01() {
    assert_eq!(frontier_sample_size(0.2, 0.05), 14);
    assert_eq!(frontier_sample_size(0.1, 0.05), 29);
}
