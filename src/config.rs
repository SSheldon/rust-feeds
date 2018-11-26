use diesel::r2d2;
use diesel::Connection;
use diesel::pg::PgConnection;
use futures::Future;
use tokio::runtime;

use data;
use fetch;
use serve;

pub type PgConnectionManager = r2d2::ConnectionManager<PgConnection>;
pub type PgConnectionPool = r2d2::Pool<PgConnectionManager>;
pub type PooledPgConnection = r2d2::PooledConnection<PgConnectionManager>;

pub struct Feeds {
    database_url: String,
}

impl Feeds {
    pub fn new(database_url: String) -> Self {
        Feeds { database_url }
    }

    fn establish_connection_pool(&self) -> PgConnectionPool {
        PgConnectionPool::new(PgConnectionManager::new(&*self.database_url))
            .expect("Failed to create pool.")
    }

    fn establish_connection(&self) -> PgConnection {
        PgConnection::establish(&self.database_url)
            .expect(&format!("Error connecting to {}", self.database_url))
    }

    pub fn serve(self, port: u16, creds: Option<(String, String)>) {
        let pool = self.establish_connection_pool();
        serve::serve(port, creds, pool)
    }

    pub fn fetch(self) {
        let pool = self.establish_connection_pool();
        runtime::run(fetch::fetch_items_task(pool).map_err(|_| ()));
    }

    pub fn prune(self) {
        let conn = self.establish_connection();
        let count = data::prune_read_items(&conn)
            .expect("Error deleting read items");
        println!("Pruned {} read items", count);
    }
}
