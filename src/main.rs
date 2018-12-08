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
extern crate tokio;
extern crate url;
extern crate warp;

mod config;
mod data;
#[macro_use]
mod error;
mod fetch;
mod handling;
mod models;
mod schema;
mod serve;

use std::env;

use crate::config::Feeds;

fn main() {
    let matches = clap::App::new("feeds")
        .setting(clap::AppSettings::DisableVersion)
        .setting(clap::AppSettings::VersionlessSubcommands)
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(clap::SubCommand::with_name("serve"))
        .subcommand(clap::SubCommand::with_name("fetch"))
        .subcommand(clap::SubCommand::with_name("prune"))
        .get_matches();

    env_logger::init();

    let feeds = env::var("DATABASE_URL")
        .map(Feeds::new)
        .expect("DATABASE_URL must be set");

    match matches.subcommand_name() {
        Some("serve") => {
            let port = env::var("PORT").ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000);

            let creds = env::var("FEVER_API_USERNAME").and_then(|user| {
                env::var("FEVER_API_PASSWORD").map(|pass| (user, pass))
            }).ok();

            feeds.serve(port, creds);
        },
        Some("fetch") => {
            feeds.fetch();
        }
        Some("prune") => {
            feeds.prune();
        }
        _ => unreachable!(),
    }

}
