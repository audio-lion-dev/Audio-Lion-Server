use axum::{routing::get, Router};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::{
    crud::{create_api_report, get_current_date, get_global_statistics, update_global_statistics},
    db::ReportType,
};

// modules
mod crud;
mod db;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u8,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/api/v1/global/stats/:admin_code",
        get(get_global_statistics).post(update_global_statistics),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 7983));
    println!("listening on {}", addr);
    match axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            eprintln!("server error: {}", err);
            create_api_report(
                ReportType::Error,
                "Error starting the server.".to_string(),
                err.to_string(),
                get_current_date(),
                "main".to_string(),
            )
            .await;
            std::process::exit(1);
        }
    }
}
