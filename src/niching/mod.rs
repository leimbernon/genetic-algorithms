//! Fitness sharing (niching) utilities for promoting population diversity.
//!
//! This module provides post-processing functions that adjust fitness values
//! based on inter-individual distances, encouraging the GA to maintain diverse
//! solutions across multiple niches.
//!
//! # Modules
//!
//! - [`configuration`] — `NichingConfiguration` with builder pattern.
//! - [`distance`] — Hamming, Euclidean, and custom distance metrics.
//! - [`sharing`] — Sharing function, fitness adjustment, and distance matrix computation.

pub mod configuration;
pub mod distance;
pub mod sharing;
