#![allow(clippy::new_without_default)]

pub mod api;
pub mod config;
pub mod database;
pub mod error;
pub mod model;
pub mod parser;

#[macro_use]
extern crate diesel;
