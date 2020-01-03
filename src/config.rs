use diesel::r2d2;
use diesel::Connection;
use diesel::pg::PgConnection;
use tokio::runtime::Runtime;

use crate::data;
use crate::fetch;
use crate::serve;

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
        let mut rt = Runtime::new()
            .expect("Error creating runtime");
        let _ = rt.block_on(serve::serve(port, creds, pool));
    }

    pub fn fetch(self) {
        let pool = self.establish_connection_pool();
        let conn = pool.get()
            .expect("Error getting connection from pool");
        let mut rt = Runtime::new()
            .expect("Error creating runtime");
        let _ = rt.block_on(fetch::fetch_items_task(conn));
    }

    pub fn prune(self) {
        let conn = self.establish_connection();
        let count = data::prune_read_items(&conn)
            .expect("Error deleting read items");
        println!("Pruned {} read items", count);
    }
}
