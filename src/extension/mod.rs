//! Extension strategies for population diversity control.
//!
//! Extensions are optional diversity-rescue mechanisms that trigger when
//! population diversity (measured by fitness standard deviation) drops below
//! a configurable threshold. They apply a corrective action to restore
//! genetic diversity and prevent premature convergence.
//!
//! # Available strategies
//!
//! - [`Extension::MassExtinction`](crate::operations::Extension::MassExtinction) — Random cull protecting elite.
//! - [`Extension::MassGenesis`](crate::operations::Extension::MassGenesis) — Trim to 2 best, regrow population.
//! - [`Extension::MassDegeneration`](crate::operations::Extension::MassDegeneration) — Multiple mutation rounds.
//! - [`Extension::MassDeduplication`](crate::operations::Extension::MassDeduplication) — Remove duplicate chromosomes.

pub mod configuration;
