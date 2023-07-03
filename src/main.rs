#[macro_use]
extern crate diesel;

mod config;
mod data;
#[macro_use]
mod error;
mod fetch;
mod greader;
mod handling;
mod models;
mod parse;
mod schema;
mod serve;

use std::env;

use tokio::runtime::Runtime;

use crate::config::Feeds;

fn main() {
    let matches = clap::Command::new("feeds")
        .subcommand_required(true)
        .subcommand(clap::Command::new("serve"))
        .subcommand(clap::Command::new("fetch"))
        .subcommand(
            clap::Command::new("subscribe")
                .arg(
                    clap::Arg::new("FEED_URL")
                        .required(true)
                )
        )
        .subcommand(clap::Command::new("prune"))
        .get_matches();

    env_logger::init();

    let feeds = env::var("DATABASE_URL")
        .map(Feeds::new)
        .expect("DATABASE_URL must be set");

    match matches.subcommand() {
        Some(("serve", _)) => {
            let port = env::var("PORT").ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000);

            let creds = env::var("FEVER_API_USERNAME").and_then(|user| {
                env::var("FEVER_API_PASSWORD").map(|pass| (user, pass))
            }).ok();

            let rt = Runtime::new()
                .expect("Error creating runtime");
            let _ = rt.block_on(feeds.serve(port, creds));
        }
        Some(("fetch", _)) => {
            let rt = Runtime::new()
                .expect("Error creating runtime");
            let _ = rt.block_on(feeds.fetch());
        }
        Some(("subscribe", subscribe_matches)) => {
            let url = subscribe_matches.get_one::<String>("FEED_URL")
                .expect("FEED_URL was not provided");
            let rt = Runtime::new()
                .expect("Error creating runtime");
            let _ = rt.block_on(feeds.subscribe(url));
        }
        Some(("prune", _)) => {
            feeds.prune();
        }
        _ => unreachable!(),
    }

}
