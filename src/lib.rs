
#![cfg_attr(feature = "benchmark", allow(unstable_features))]
#![cfg_attr(feature = "benchmark", feature(test))]

#[cfg(feature = "benchmark")]
extern crate test;

mod apng;

pub use apng::*;
pub use apng::encoder::*;
pub use apng::errors::*;
