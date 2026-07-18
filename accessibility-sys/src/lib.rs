//! Raw macOS Accessibility bindings and constants.
//!
//! This crate supplements `objc2_application_services` with AX string constants,
//! which are `CFSTR` macros and therefore absent from generated bindings.

mod action_constants;
mod attribute_constants;
mod error;
mod notification_constants;
mod role_constants;
mod value_constants;

pub use objc2_application_services::*;

pub use action_constants::*;
pub use attribute_constants::*;
pub use error::*;
pub use notification_constants::*;
pub use role_constants::*;
pub use value_constants::*;

pub use libc::pid_t;
