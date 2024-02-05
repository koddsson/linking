use actix_web::{middleware::Logger, web, App, HttpServer};
use linking::services::{create, show, scaffold_database};
use r2d2_sqlite::SqliteConnectionManager;

extern crate rusqlite;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up some default logger for the application. I just copy/pasted this from somewhere.
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Create a SQLite pool in which we can clone connections from.
    // We do this so we can clone connections into functions such as the _app_data_
    // when we initialize the server.
    //
    // We just fail fast if we can't connect to the database.
    let manager = SqliteConnectionManager::file("urls.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file");

    // Scaffold the database with some tables we need on start.
    // I figured this was better than having some scaffolding scripts.
    scaffold_database(pool.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            // Clone a pool into the app_data of the application so that each
            // service has a reference to the pool and can get a connection to the database.
            .app_data(web::Data::new(pool.clone()))
            .service(show)
            .service(create)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
