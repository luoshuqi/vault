pub use server::start as start_server;

use crate::error::*;

#[macro_use]
mod error;
#[cfg(target_os = "android")]
mod android;
mod crypto;
mod db;
mod server;
mod service;
