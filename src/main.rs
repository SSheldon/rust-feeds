// Workaround for diesl#1785
#![allow(proc_macro_derive_resolution_fallback)]

extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate feed_stream;
extern crate fever_api;
extern crate futures;
extern crate iter_read;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate warp;

mod data;
mod handling;
mod models;
mod schema;
mod serve;

use std::env;

fn main() {
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let port = env::var("PORT").ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    serve::serve(port, database_url);
}
