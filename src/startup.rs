use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::middleware::Logger;
use std::net::TcpListener;
use sqlx::PgPool;
use crate::routes::{health_check, subscribe};


pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {

  let db_pool = Data::new(db_pool);

  let server = HttpServer::new(move || {    
    App::new()
      .wrap(Logger::default())
      .route("/health_check", web::get().to(health_check))
      .route("/subscriptions", web::post().to(subscribe))
      // Register the connection as part of the application state
      .app_data(db_pool.clone())
  })
  .listen(listener)?
  .run();

  Ok(server)
}