//! Feature-gated GPU backend implementations.

#[cfg(feature = "software")]
pub mod softbuffer;