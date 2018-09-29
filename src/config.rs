use std::env;

use serve;

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
}
