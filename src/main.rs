// Workaround for diesl#1785
#![allow(proc_macro_derive_resolution_fallback)]

extern crate chrono;
extern crate clap;
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

mod config;
mod data;
mod handling;
mod models;
mod schema;
mod serve;

use std::env;

use config::Feeds;

fn main() {
    let matches = clap::App::new("feeds")
        .setting(clap::AppSettings::DisableVersion)
        .setting(clap::AppSettings::VersionlessSubcommands)
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(clap::SubCommand::with_name("serve"))
        .get_matches();

    env_logger::init();

    let feeds = Feeds::new()
        .expect("DATABASE_URL must be set");

    match matches.subcommand_name() {
        Some("serve") => {
            let port = env::var("PORT").ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000);

            feeds.serve(port);
        },
        _ => unreachable!(),
    }

}
