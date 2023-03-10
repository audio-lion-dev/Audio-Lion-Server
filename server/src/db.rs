use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::{
    bson::{self, doc},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const MONGODB_URI: &str = "mongodb://localhost:27017";

async fn get_client() -> Client {
    let client = match Client::with_uri_str(MONGODB_URI).await {
        Ok(client) => {
            println!("Connected Database successfully.");
            client
        }
        Err(e) => panic!("Failed to initialize client: {}", e),
    };
    return client;
}

pub async fn get_database(name: Option<String>) -> mongodb::Database {
    let client = get_client().await;
    let db = match name {
        Some(name) => client.database(&name),
        None => client.database("app"),
    };
    return db;
}

pub enum ValidCollections {
    Statistics,
    Reports
}

/// Get a collection from the database
///
/// Returns a collection of type T
pub async fn get_collection<T>(collection_type: ValidCollections) -> Collection<T> {
    let db = get_database(None).await;
    let collection = match collection_type {
        ValidCollections::Statistics => db.collection::<T>("statistics"),
        ValidCollections::Reports => db.collection::<T>("reports"),
    };
    return collection;
}

#[derive(Debug, Deserialize, Serialize)]
/// Global statistics for the app
/// There will only be one document in the collection
pub struct Statistics {
    pub online_users: u32,
    pub downloads: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ReportType {
    Bug,
    Error,
    Other,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    /// The name of the Report
    pub name: ReportType,
    /// The general quick description of the report
    pub description: String,
    /// The full description of the report
    pub message: String,
    /// The date the report was created
    pub date: String,
    /// The name of the function or file that created the report
    pub caller: String,
}