use std::env;

use diesel::r2d2;
use diesel::Connection;
use diesel::pg::PgConnection;
use tokio::runtime;

use data;
use handling;
use serve;

pub type PgConnectionManager = r2d2::ConnectionManager<PgConnection>;
pub type PgConnectionPool = r2d2::Pool<PgConnectionManager>;
pub type PooledPgConnection = r2d2::PooledConnection<PgConnectionManager>;

fn fetch(conn: PgConnection) {
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

    fn establish_connection_pool(&self) -> PgConnectionPool {
        PgConnectionPool::new(PgConnectionManager::new(&*self.database_url))
            .expect("Failed to create pool.")
    }

    fn establish_connection(&self) -> PgConnection {
        PgConnection::establish(&self.database_url)
            .expect(&format!("Error connecting to {}", self.database_url))
    }

    pub fn serve(self, port: u16) {
        let pool = self.establish_connection_pool();
        serve::serve(port, pool)
    }

    pub fn fetch(self) {
        let conn = self.establish_connection();
        fetch(conn)
    }

    pub fn prune(self) {
        let conn = self.establish_connection();
        let count = data::prune_read_items(&conn)
            .expect("Error deleting read items");
        println!("Pruned {} read items", count);
    }
}
