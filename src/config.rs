use std::env;

use diesel::Connection;
use diesel::pg::PgConnection;
use tokio::runtime;

use handling;
use serve;

fn refresh(conn: PgConnection) {
    use std::ops::Deref;

    struct ConnectionWrapper(PgConnection);
    impl Deref for ConnectionWrapper {
        type Target = PgConnection;
        fn deref(&self) -> &PgConnection { &self.0 }
    }

    let task = handling::fetch_items_task(ConnectionWrapper(conn));
    runtime::run(task);
}

pub struct Feeds {
    database_url: String,
}

impl Feeds {
    pub fn new() -> Result<Self, env::VarError> {
        env::var("DATABASE_URL")
            .map(|database_url| Feeds { database_url })
    }

    pub fn serve(self, port: u16) {
        serve::serve(port, self.database_url)
    }

    pub fn refresh(self) {
        let conn = PgConnection::establish(&self.database_url)
            .expect(&format!("Error connecting to {}", self.database_url));
        refresh(conn)
    }
}
