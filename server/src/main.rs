#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

// imports
use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    response::Json,
    response::{Html, IntoResponse, Response},
    routing::get,
    RequestPartsExt, Router,
};
use db::get_collection;
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr};

use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};

use mongodb::{
    bson::{self, doc},
    Client, Collection,
};

use crate::{
    crud::{create_api_report, get_current_date, get_global_statistics},
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
    let app = Router::new().route("/api/v1/global/stats", get(get_global_statistics));

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
