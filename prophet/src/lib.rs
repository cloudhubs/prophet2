//! A library for cloning microservice git repositories and analyzing them

pub(crate) mod repositories;
pub use repositories::*;

pub(crate) mod error;
pub use error::*;

pub(crate) mod app_data;
pub use app_data::*;

pub(crate) mod adapter;
