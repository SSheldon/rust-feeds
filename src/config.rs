use diesel::r2d2;
use diesel::Connection;
use diesel::pg::PgConnection;

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

    pub async fn serve(self, port: u16, creds: Option<(String, String)>) {
        let pool = self.establish_connection_pool();
        serve::serve(port, creds, pool).await;
    }

    pub async fn fetch(self) {
        let mut conn = self.establish_connection();
        fetch::fetch_items(&mut conn).await
            .expect("Error fetching feeds");
    }

    pub async fn subscribe(self, url: &str) {
        let mut conn = self.establish_connection();
        fetch::subscribe(url, &mut conn).await
            .expect("Error subscribing to feed");
    }

    pub fn prune(self) {
        let mut conn = self.establish_connection();
        let count = data::prune_read_items(&mut conn)
            .expect("Error deleting read items");
        println!("Pruned {} read items", count);
    }
}
