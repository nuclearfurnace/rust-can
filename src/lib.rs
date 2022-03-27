//! A general purpose library of common CAN types.
//! 
//! Provides various logical types that are required to meaningfully interact with CAN, such as
//! identifiers, frames, and filters.
//! 
//! ## Feature flags
//! 
//! While the types are ostensibly meant to be foundational and thus shared amongst the ecosystem,
//! the crate does provide conversion implementations for popular CAN-related crates to allow for interoperation:
//! 
//! - **embedded-can-compat**: supports converting identifiers into [`embedded-can`][embedded-can] identifiers
//! - **socketcan-compat**: supports converting filters into [socketcan][socketcan] filters
//! 
//! All feature flags are enabled by default.
//! 
//! [embedded-can]: https://docs.rs/embedded-can/latest/embedded_can/
//! [socketcan]: https://docs.rs/socketcan/latest/socketcan/
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]

pub mod constants;
pub mod frame;
pub mod identifier;
