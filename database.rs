extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use postgres::Connection;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use postgres::tls::openssl::openssl::ssl::{SslConnectorBuilder, SslMethod};
use std::{ops::Deref, env};
use ::rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome, http::Status};

// An alias to the type for a pool r2d2 Postgresql connections
type Pool = r2d2::Pool<PostgresConnectionManager>;

// Creates the database connection pool
pub fn init_pg_pool() -> Pool {
    let config = r2d2::Config::default();
    let conn_string =  env::var("DB_URL").unwrap_or(format!("postgres://nicolasb@localhost/nicolasb"));
    let dbssl = env::var("DB_SSL").unwrap_or(format!("require"));
    // ssl connection
    let mut connbuilder = SslConnectorBuilder::new(SslMethod::tls()).unwrap();
    match dbssl.to_lowercase().as_ref() {
        "require" | "prefer" | "allow" => connbuilder.set_verify(postgres::tls::openssl::openssl::ssl::SSL_VERIFY_NONE),
        _ => (), // by default we verify certs: it's like either verify-ca or verify-full, TBD
    }
    
    let negotiator = Box::new(::postgres::tls::openssl::OpenSsl::from(connbuilder.build()));
    let db_ssl_mode = match dbssl.to_lowercase().as_ref() {
        "require" | "verify-ca" | "verify-full" => TlsMode::Require(negotiator),
        // `prefer` and `allow` fall into here and will not try TLS. 
        // Not totally correct: please use at least `require` for real use.
        _ => TlsMode::None, 
    };

    // r2d2 connection pool
    let manager = PostgresConnectionManager::new(conn_string.as_ref(), db_ssl_mode).expect("Could not connect to database using specified connection string.");
    r2d2::Pool::new(config, manager).expect("Could not create database pool")
}

// Connection request guard type: a wrapper around an r2d2 pooled connection. 
pub struct Dbconn(
    pub r2d2::PooledConnection<PostgresConnectionManager>
);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for Dbconn {
    type Error = ();
 
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Dbconn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Dbconn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}
 
// For the convenience of using &Dbconn
impl Deref for Dbconn {
    type Target = Connection;
 
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}