#[macro_use]
extern crate diesel;

mod config;
mod data;
#[macro_use]
mod error;
mod fetch;
mod handling;
mod models;
mod parse;
mod schema;
mod serve;

use std::env;

use tokio::runtime::Runtime;

use crate::config::Feeds;

fn main() {
    let matches = clap::App::new("feeds")
        .setting(clap::AppSettings::DisableVersion)
        .setting(clap::AppSettings::VersionlessSubcommands)
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(clap::SubCommand::with_name("serve"))
        .subcommand(clap::SubCommand::with_name("fetch"))
        .subcommand(
            clap::SubCommand::with_name("subscribe")
                .arg(
                    clap::Arg::with_name("FEED_URL")
                        .required(true)
                        .takes_value(true)
                )
        )
        .subcommand(clap::SubCommand::with_name("prune"))
        .get_matches();

    env_logger::init();

    let feeds = env::var("DATABASE_URL")
        .map(Feeds::new)
        .expect("DATABASE_URL must be set");

    match matches.subcommand() {
        ("serve", _) => {
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
        ("fetch", _) => {
            let rt = Runtime::new()
                .expect("Error creating runtime");
            let _ = rt.block_on(feeds.fetch());
        }
        ("subscribe", Some(subscribe_matches)) => {
            let url = subscribe_matches.value_of("FEED_URL")
                .expect("FEED_URL was not provided");
            let rt = Runtime::new()
                .expect("Error creating runtime");
            let _ = rt.block_on(feeds.subscribe(url));
        }
        ("prune", _) => {
            feeds.prune();
        }
        _ => unreachable!(),
    }

}
