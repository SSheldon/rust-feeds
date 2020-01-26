use std::ops::Deref;

use diesel::r2d2;
use diesel::Connection;
use diesel::pg::PgConnection;

use crate::data;
use crate::fetch;
use crate::serve;

pub type PgConnectionManager = r2d2::ConnectionManager<PgConnection>;
pub type PgConnectionPool = r2d2::Pool<PgConnectionManager>;
pub type PooledPgConnection = r2d2::PooledConnection<PgConnectionManager>;

pub enum MaybePooled<T: 'static + Connection> {
    Pooled(r2d2::PooledConnection<r2d2::ConnectionManager<T>>),
    Owned(T),
}

impl<T: 'static + Connection> Deref for MaybePooled<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            MaybePooled::Pooled(conn) => conn,
            MaybePooled::Owned(conn) => conn,
        }
    }
}

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
        let conn = self.establish_connection();
        let conn = MaybePooled::Owned(conn);
        fetch::fetch_items_task(conn).await
            .expect("Error fetching feeds");
    }

    pub fn prune(self) {
        let conn = self.establish_connection();
        let count = data::prune_read_items(&conn)
            .expect("Error deleting read items");
        println!("Pruned {} read items", count);
    }
}
