//! Adaptive Operator Selection (AOS) core module.
//!
//! Provides strategies for dynamically selecting operators from a portfolio
//! during a GA run, based on recent reward history.
//!
//! # Strategies
//!
//! - **Probability Matching (PM)**: Maintains selection probabilities that are
//!   updated toward observed rewards using a learning rate.
//! - **Adaptive Pursuit (AP)**: Like PM but more aggressive — pursues the
//!   current best arm with higher probability.
//! - **Multi-Armed Bandit (MAB)**: Uses UCB1 with ε-greedy exploration.
//!
//! # WASM Compatibility
//!
//! This module is pure data arithmetic — no `Instant`, no `rayon`, no
//! `std::sync::Mutex`. It compiles for `wasm32-unknown-unknown`.

use rand::Rng;
use std::f64;

/// Strategy variant for adaptive operator selection.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::aos::AosStrategy;
///
/// let pm = AosStrategy::pm_default();
/// assert!(matches!(pm, AosStrategy::ProbabilityMatching { .. }));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AosStrategy {
    /// Probability Matching: update probabilities toward observed rewards.
    ProbabilityMatching {
        /// Learning rate — how fast probabilities adapt.
        alpha: f64,
        /// Step size for probability updates.
        learning_rate: f64,
    },
    /// Adaptive Pursuit: aggressively pursue the best arm.
    AdaptivePursuit {
        /// Pursuit rate.
        beta: f64,
        /// Scaling factor.
        c: f64,
    },
    /// Multi-Armed Bandit with UCB1 and ε-greedy.
    MultiArmedBandit {
        /// Exploration constant for UCB1.
        c: f64,
        /// Probability of random exploration.
        epsilon: f64,
    },
}

impl AosStrategy {
    /// Probability Matching with literature-standard parameters:
    /// `alpha = 0.8`, `learning_rate = 0.3`.
    pub fn pm_default() -> Self {
        AosStrategy::ProbabilityMatching {
            alpha: 0.8,
            learning_rate: 0.3,
        }
    }

    /// Adaptive Pursuit with literature-standard parameters:
    /// `beta = 0.5`, `c = 1.5`.
    pub fn ap_default() -> Self {
        AosStrategy::AdaptivePursuit { beta: 0.5, c: 1.5 }
    }

    /// Multi-Armed Bandit with literature-standard parameters:
    /// `c = 1.0`, `epsilon = 0.1`.
    pub fn mab_default() -> Self {
        AosStrategy::MultiArmedBandit {
            c: 1.0,
            epsilon: 0.1,
        }
    }
}

/// Per-arm sliding window reward buffer (ring buffer).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct ArmState {
    /// Circular buffer of recent rewards.
    rewards: Vec<f64>,
    /// Write cursor into the buffer.
    cursor: usize,
    /// Total number of rewards recorded (capped at `window_size` for mean).
    count: usize,
}

impl ArmState {
    fn new(window_size: usize) -> Self {
        ArmState {
            rewards: vec![0.0; window_size],
            cursor: 0,
            count: 0,
        }
    }

    fn add_reward(&mut self, reward: f64) {
        self.rewards[self.cursor] = reward;
        self.cursor = (self.cursor + 1) % self.rewards.len();
        self.count = (self.count + 1).min(self.rewards.len());
    }

    fn mean_reward(&self) -> f64 {
        let n = self.count.max(1);
        self.rewards[..self.count].iter().sum::<f64>() / n as f64
    }
}

/// Runtime state machine for one operator portfolio.
///
/// Tracks per-arm rewards, selection probabilities (for PM/AP), and
/// selection counts (for MAB UCB). Provides methods to select operators,
/// record rewards, and update probabilities.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::aos::{AosState, AosStrategy};
///
/// let state = AosState::new(3, AosStrategy::pm_default(), 10);
/// assert_eq!(state.num_arms(), 3);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AosState {
    /// Number of operators (arms) in the portfolio.
    num_arms: usize,
    /// Per-arm sliding window reward buffers.
    arms: Vec<ArmState>,
    /// Selection probabilities for PM/AP (uniform initial, sum = 1.0).
    probabilities: Vec<f64>,
    /// Selection counts for MAB UCB.
    selection_counts: Vec<usize>,
    /// Total selections across all arms.
    total_selections: usize,
    /// The AOS strategy and parameters.
    strategy: AosStrategy,
    /// Reward window size.
    window_size: usize,
    /// Exploration phase length = window_size / 2.
    exploration_generations: usize,
}

impl AosState {
    /// Creates a new AOS state for a portfolio with `num_arms` operators.
    ///
    /// - Initializes per-arm ring buffers of capacity `window_size`.
    /// - Sets all selection probabilities to `1.0 / num_arms`.
    /// - Sets exploration phase length to `window_size / 2`.
    pub fn new(num_arms: usize, strategy: AosStrategy, window_size: usize) -> Self {
        let arms: Vec<ArmState> = (0..num_arms).map(|_| ArmState::new(window_size)).collect();
        let probabilities = vec![1.0 / num_arms.max(1) as f64; num_arms];
        let selection_counts = vec![0; num_arms];
        AosState {
            num_arms,
            arms,
            probabilities,
            selection_counts,
            total_selections: 0,
            strategy,
            window_size,
            exploration_generations: window_size / 2,
        }
    }

    /// Selects an operator index from the portfolio.
    ///
    /// During the exploration phase (`generation < exploration_generations`),
    /// selects uniformly at random. After exploration, dispatches by strategy:
    ///
    /// - **PM/AP**: Roulette-wheel selection over `self.probabilities`.
    /// - **MAB**: ε-greedy with UCB1 for the greedy arm.
    ///
    /// Increments selection tracking for the chosen arm.
    pub fn select_operator(&mut self, rng: &mut impl Rng, generation: usize) -> usize {
        if self.num_arms == 0 {
            return 0;
        }

        let op_idx = if generation < self.exploration_generations {
            // Exploration phase: uniform random
            rng.random_range(0..self.num_arms)
        } else {
            match self.strategy {
                AosStrategy::ProbabilityMatching { .. } | AosStrategy::AdaptivePursuit { .. } => {
                    self.roulette_wheel_select(rng)
                }
                AosStrategy::MultiArmedBandit { c, epsilon } => self.mab_select(rng, c, epsilon),
            }
        };

        self.selection_counts[op_idx] = self.selection_counts[op_idx].saturating_add(1);
        self.total_selections = self.total_selections.saturating_add(1);
        op_idx
    }

    /// Roulette-wheel selection over `self.probabilities`.
    fn roulette_wheel_select(&self, rng: &mut impl Rng) -> usize {
        let prob_sum: f64 = self.probabilities.iter().sum();
        if prob_sum <= 0.0 {
            // Degenerate case: uniform fallback
            return rng.random_range(0..self.num_arms);
        }
        let r = rng.random_range(0.0..prob_sum);
        let mut cumulative = 0.0;
        for (i, &p) in self.probabilities.iter().enumerate() {
            cumulative += p;
            if r < cumulative {
                return i;
            }
        }
        self.num_arms - 1 // Fallback (should not reach here in practice)
    }

    /// MAB selection with ε-greedy and UCB1.
    fn mab_select(&self, rng: &mut impl Rng, c: f64, epsilon: f64) -> usize {
        // ε-greedy: with probability epsilon, explore uniformly
        if rng.random_range(0.0..1.0) < epsilon {
            return rng.random_range(0..self.num_arms);
        }

        // UCB1: compute upper confidence bounds
        let total = self.total_selections.max(1) as f64;
        let mut best_arm = 0;
        let mut best_ucb = f64::NEG_INFINITY;

        for i in 0..self.num_arms {
            let n = self.selection_counts[i];
            if n == 0 {
                // Forced exploration: any unselected arm gets priority
                return i;
            }
            let mean = self.arms[i].mean_reward();
            let ucb = mean + c * (2.0 * total.ln() / n as f64).sqrt();
            if ucb > best_ucb {
                best_ucb = ucb;
                best_arm = i;
            }
        }

        best_arm
    }

    /// Records rewards for one or more operators.
    ///
    /// Each entry `(op_idx, reward)` appends `reward` to that arm's ring buffer.
    /// Out-of-range operator indices are silently dropped (bounds check per T-43-01).
    pub fn record_rewards(&mut self, rewards: &[(usize, f64)]) {
        for &(op_idx, reward) in rewards {
            if op_idx < self.num_arms {
                self.arms[op_idx].add_reward(reward);
            }
        }
    }

    /// Recomputes selection probabilities for PM/AP strategies.
    ///
    /// For **PM**: each arm's probability is updated toward its mean reward
    /// with the configured learning rate. Probabilities are clamped to
    /// `[P_MIN, 1.0]` where `P_MIN = 1.0 / (num_arms * 1.5)`, then normalized.
    ///
    /// For **AP**: the best arm's probability is pursued toward 0.9; all other
    /// arms are driven toward `0.1 / (num_arms - 1)`. Clamped to `[0.01, 0.99]`
    /// then normalized.
    ///
    /// **MAB**: no probability update needed (UCB handles exploration).
    pub fn update(&mut self) {
        match self.strategy {
            AosStrategy::ProbabilityMatching {
                alpha,
                learning_rate: _,
            } => {
                self.update_pm(alpha);
            }
            AosStrategy::AdaptivePursuit { beta, c: _ } => {
                self.update_ap(beta);
            }
            AosStrategy::MultiArmedBandit { .. } => {
                // MAB: UCB handles exploration naturally — no probability update needed
            }
        }
    }

    /// PM probability update.
    fn update_pm(&mut self, alpha: f64) {
        let p_min = 1.0 / (self.num_arms.max(1) as f64 * 1.5);
        let credits: Vec<f64> = (0..self.num_arms)
            .map(|i| self.arms[i].mean_reward())
            .collect();

        for (prob, credit) in self.probabilities.iter_mut().zip(credits.iter()) {
            let new_prob = *prob + alpha * (credit - *prob);
            *prob = new_prob.clamp(p_min, 1.0);
        }

        self.normalize_probabilities();
    }

    /// AP probability update.
    fn update_ap(&mut self, beta: f64) {
        // Find the arm with the highest mean reward
        let best_idx = (0..self.num_arms)
            .max_by(|&a, &b| {
                self.arms[a]
                    .mean_reward()
                    .partial_cmp(&self.arms[b].mean_reward())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(0);

        let n = self.num_arms.max(2) as f64;
        for i in 0..self.num_arms {
            if i == best_idx {
                let target = 0.9;
                self.probabilities[i] =
                    self.probabilities[i] + beta * (target - self.probabilities[i]);
            } else {
                let target = 0.1 / (n - 1.0);
                self.probabilities[i] =
                    self.probabilities[i] + beta * (target - self.probabilities[i]);
            }
            self.probabilities[i] = self.probabilities[i].clamp(0.01, 0.99);
        }

        self.normalize_probabilities();
    }

    /// Normalize probabilities to sum to 1.0.
    fn normalize_probabilities(&mut self) {
        let sum: f64 = self.probabilities.iter().sum();
        if sum > 0.0 {
            for p in self.probabilities.iter_mut() {
                *p /= sum;
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Accessors for testing
    // ---------------------------------------------------------------------------

    /// Returns the current selection probabilities (for PM/AP).
    pub fn probabilities(&self) -> &[f64] {
        &self.probabilities
    }

    /// Returns the number of arms in this state.
    pub fn num_arms(&self) -> usize {
        self.num_arms
    }

    /// Returns the exploration phase length.
    pub fn exploration_generations(&self) -> usize {
        self.exploration_generations
    }

    /// Returns the window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Returns the AOS strategy.
    pub fn strategy(&self) -> &AosStrategy {
        &self.strategy
    }

    /// Returns per-arm selection counts.
    pub fn selection_counts(&self) -> &[usize] {
        &self.selection_counts
    }

    /// Returns total selections across all arms.
    pub fn total_selections(&self) -> usize {
        self.total_selections
    }

    /// Returns per-arm mean rewards (for inspection).
    pub fn arm_mean_reward(&self, idx: usize) -> f64 {
        if idx < self.num_arms {
            self.arms[idx].mean_reward()
        } else {
            0.0
        }
    }
}

/// Computes a normalized reward from parent and offspring fitness values.
///
/// Formula: `(parent_fitness - offspring_fitness) / max(|best_fitness|, EPSILON)`
///
/// A positive result means the offspring is better (lower fitness for minimization).
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::aos::compute_normalized_reward;
///
/// let reward = compute_normalized_reward(1.0, 0.8, 1.0);
/// assert!((reward - 0.2).abs() < 1e-9);
/// ```
/// A negative result means the offspring is worse.
/// The denominator is clamped to `f64::EPSILON` to prevent division by zero (T-43-03).
///
/// This function assumes **lower fitness is better** for the comparison.
/// The GA loop is responsible for passing (parent, offspring, best) with the
/// correct sign depending on the optimization direction.
pub fn compute_normalized_reward(
    parent_fitness: f64,
    offspring_fitness: f64,
    best_fitness: f64,
) -> f64 {
    let delta = parent_fitness - offspring_fitness;
    let denom = best_fitness.abs().max(f64::EPSILON);
    delta / denom
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn make_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

    // ---------------------------------------------------------------------------
    // AosStrategy default constructors
    // ---------------------------------------------------------------------------

    #[test]
    fn test_pm_default() {
        let s = AosStrategy::pm_default();
        match s {
            AosStrategy::ProbabilityMatching {
                alpha,
                learning_rate,
            } => {
                assert!((alpha - 0.8).abs() < 1e-10);
                assert!((learning_rate - 0.3).abs() < 1e-10);
            }
            _ => panic!("Expected ProbabilityMatching"),
        }
    }

    #[test]
    fn test_ap_default() {
        let s = AosStrategy::ap_default();
        match s {
            AosStrategy::AdaptivePursuit { beta, c } => {
                assert!((beta - 0.5).abs() < 1e-10);
                assert!((c - 1.5).abs() < 1e-10);
            }
            _ => panic!("Expected AdaptivePursuit"),
        }
    }

    #[test]
    fn test_mab_default() {
        let s = AosStrategy::mab_default();
        match s {
            AosStrategy::MultiArmedBandit { c, epsilon } => {
                assert!((c - 1.0).abs() < 1e-10);
                assert!((epsilon - 0.1).abs() < 1e-10);
            }
            _ => panic!("Expected MultiArmedBandit"),
        }
    }

    // ---------------------------------------------------------------------------
    // AosState construction
    // ---------------------------------------------------------------------------

    #[test]
    fn test_new_creates_correct_number_of_arms() {
        let state = AosState::new(3, AosStrategy::pm_default(), 50);
        assert_eq!(state.num_arms(), 3);
        assert_eq!(state.probabilities().len(), 3);
        assert_eq!(state.selection_counts().len(), 3);
        assert_eq!(state.arms.len(), 3);
    }

    #[test]
    fn test_new_uniform_probabilities() {
        let state = AosState::new(4, AosStrategy::pm_default(), 50);
        for &p in state.probabilities() {
            assert!((p - 0.25).abs() < 1e-10);
        }
    }

    #[test]
    fn test_new_exploration_generations() {
        let state = AosState::new(2, AosStrategy::pm_default(), 50);
        assert_eq!(state.exploration_generations(), 25);
    }

    #[test]
    fn test_new_zero_arms() {
        let state = AosState::new(0, AosStrategy::pm_default(), 50);
        assert_eq!(state.num_arms(), 0);
        assert!(state.probabilities().is_empty());
    }

    // ---------------------------------------------------------------------------
    // select_operator — exploration phase
    // ---------------------------------------------------------------------------

    #[test]
    fn test_select_exploration_distribution() {
        let mut state = AosState::new(5, AosStrategy::pm_default(), 100);
        let mut rng = make_rng();
        let mut counts = [0usize; 5];
        for _ in 0..500 {
            let op = state.select_operator(&mut rng, 10); // exploration (10 < 50)
            assert!(op < 5);
            counts[op] += 1;
        }
        // All 5 arms should have been selected at least once
        assert!(
            counts.iter().all(|&c| c > 0),
            "All arms should be selected during exploration"
        );
    }

    #[test]
    fn test_select_exploration_tracks_counts() {
        let mut state = AosState::new(3, AosStrategy::pm_default(), 50);
        let mut rng = make_rng();
        let _ = state.select_operator(&mut rng, 0);
        assert_eq!(state.total_selections(), 1);
        let total_counts: usize = state.selection_counts().iter().sum();
        assert_eq!(total_counts, 1);
    }

    // ---------------------------------------------------------------------------
    // select_operator — post-exploration PM/AP
    // ---------------------------------------------------------------------------

    #[test]
    fn test_select_pm_post_exploration() {
        let mut state = AosState::new(2, AosStrategy::pm_default(), 50);
        let mut rng = make_rng();
        // Post-exploration (gen > 25)
        let _ = state.select_operator(&mut rng, 30);
        // Should not panic — selection works
    }

    #[test]
    fn test_select_ap_post_exploration() {
        let mut state = AosState::new(2, AosStrategy::ap_default(), 50);
        let mut rng = make_rng();
        let _ = state.select_operator(&mut rng, 30);
    }

    // ---------------------------------------------------------------------------
    // select_operator — MAB
    // ---------------------------------------------------------------------------

    #[test]
    fn test_select_mab_forced_exploration() {
        let mut state = AosState::new(3, AosStrategy::mab_default(), 50);
        let mut rng = make_rng();
        // All arms have selection_count == 0 — first MAB selection should pick
        // the first arm with count 0 (forced exploration)
        let op = state.select_operator(&mut rng, 30);
        assert!(op < 3);
    }

    #[test]
    fn test_select_mab_ucb1_after_rewards() {
        let mut state = AosState::new(3, AosStrategy::mab_default(), 50);
        let mut rng = make_rng();

        // Seed some selections first to get past forced exploration
        state.record_rewards(&[(0, 0.5), (0, 0.6), (0, 0.7)]);
        state.record_rewards(&[(1, 0.1), (1, 0.0), (1, -0.1)]);
        state.record_rewards(&[(2, 0.3), (2, 0.2), (2, 0.4)]);

        // Seed enough selections so no forced exploration remains
        for i in 0..5 {
            // Record selects at exploration phases and post-exploration
            let _ = state.select_operator(&mut rng, i);
        }
        // Post-exploration
        let _ = state.select_operator(&mut rng, 30);
    }

    // ---------------------------------------------------------------------------
    // record_rewards
    // ---------------------------------------------------------------------------

    #[test]
    fn test_record_rewards_single() {
        let mut state = AosState::new(2, AosStrategy::pm_default(), 50);
        state.record_rewards(&[(0, 0.5), (1, -0.2)]);
        assert!((state.arm_mean_reward(0) - 0.5).abs() < 1e-10);
        assert!((state.arm_mean_reward(1) - (-0.2)).abs() < 1e-10);
    }

    #[test]
    fn test_record_rewards_out_of_bounds_dropped() {
        let mut state = AosState::new(2, AosStrategy::pm_default(), 50);
        // Should not panic
        state.record_rewards(&[(5, 0.5)]);
        // Arm 0 should still have default mean (no reward recorded)
        assert!((state.arm_mean_reward(0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_record_rewards_batch() {
        let mut state = AosState::new(3, AosStrategy::pm_default(), 10);
        let rewards: Vec<(usize, f64)> = vec![(0, 0.8), (1, 0.1), (0, 0.6)];
        state.record_rewards(&rewards);
        // Arm 0: two rewards, mean should be 0.7
        assert!((state.arm_mean_reward(0) - 0.7).abs() < 1e-10);
        // Arm 1: one reward, mean = 0.1
        assert!((state.arm_mean_reward(1) - 0.1).abs() < 1e-10);
        // Arm 2: no rewards, mean = 0.0
        assert!((state.arm_mean_reward(2) - 0.0).abs() < 1e-10);
    }

    // ---------------------------------------------------------------------------
    // update — PM
    // ---------------------------------------------------------------------------

    #[test]
    fn test_update_pm_changes_probabilities() {
        let mut state = AosState::new(
            3,
            AosStrategy::ProbabilityMatching {
                alpha: 0.8,
                learning_rate: 0.3,
            },
            10,
        );
        // Give arm 0 high rewards, arms 1-2 low
        for _ in 0..3 {
            state.record_rewards(&[(0, 1.0), (1, -0.5), (2, -0.3)]);
        }
        state.update();
        // After update, arm 0 should have higher probability
        let probs = state.probabilities().to_vec();
        assert!(
            probs[0] > probs[1],
            "Arm 0 should have higher prob, got {:?}",
            probs
        );
        // Probabilities should sum to ~1.0
        let sum: f64 = probs.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Probabilities should sum to ~1.0, got {}",
            sum
        );
    }

    // ---------------------------------------------------------------------------
    // update — AP
    // ---------------------------------------------------------------------------

    #[test]
    fn test_update_ap() {
        let mut state = AosState::new(2, AosStrategy::AdaptivePursuit { beta: 0.5, c: 1.5 }, 10);
        state.record_rewards(&[(0, 1.0), (1, -1.0)]);
        state.update();
        let probs = state.probabilities().to_vec();
        let sum: f64 = probs.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Probabilities should sum to ~1.0, got {}",
            sum
        );
    }

    // ---------------------------------------------------------------------------
    // update — MAB (no-op, verifies no panic)
    // ---------------------------------------------------------------------------

    #[test]
    fn test_update_mab_noop() {
        let mut state = AosState::new(2, AosStrategy::mab_default(), 10);
        state.record_rewards(&[(0, 1.0), (1, -1.0)]);
        state.update();
        // MAB does not update probabilities; they should remain uniform
        for &p in state.probabilities() {
            assert!((p - 0.5).abs() < 1e-10);
        }
    }

    // ---------------------------------------------------------------------------
    // compute_normalized_reward
    // ---------------------------------------------------------------------------

    #[test]
    fn test_compute_normalized_reward_positive() {
        // Offspring better (lower fitness) than parent: positive reward
        let reward = compute_normalized_reward(10.0, 5.0, 20.0);
        assert!(
            reward > 0.0,
            "Better offspring should give positive reward, got {}",
            reward
        );
        assert!(
            (reward - 0.25).abs() < 1e-10,
            "Expected ~0.25, got {}",
            reward
        );
    }

    #[test]
    fn test_compute_normalized_reward_negative() {
        // Offspring worse (higher fitness) than parent: negative reward
        let reward = compute_normalized_reward(5.0, 10.0, 20.0);
        assert!(
            reward < 0.0,
            "Worse offspring should give negative reward, got {}",
            reward
        );
    }

    #[test]
    fn test_compute_normalized_reward_zero_delta() {
        let reward = compute_normalized_reward(10.0, 10.0, 20.0);
        assert!(
            (reward).abs() < 1e-10,
            "Equal fitness should give ~0 reward, got {}",
            reward
        );
    }

    #[test]
    fn test_compute_normalized_reward_zero_best() {
        // Best fitness of 0 should not cause NaN (EPSILON clamp)
        let reward = compute_normalized_reward(10.0, 5.0, 0.0);
        assert!(
            !reward.is_nan(),
            "Reward should not be NaN when best_fitness=0"
        );
        assert!(reward > 0.0, "Positive reward expected");
    }

    #[test]
    fn test_compute_normalized_reward_best_nonzero() {
        let reward = compute_normalized_reward(100.0, 95.0, 50.0);
        assert!(
            (reward - 0.1).abs() < 1e-10,
            "Expected ~0.1, got {}",
            reward
        );
    }
}
