#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unreachable_pub)]
#![allow(clippy::missing_errors_doc)]

pub mod apns;
pub(crate) mod auth;
pub(crate) mod config;
pub(crate) mod handlers;

pub mod db;
pub mod error;
pub mod models;
pub mod router;
