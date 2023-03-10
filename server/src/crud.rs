use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::{
    bson::{self, doc},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::db::{get_collection, Report, ReportType, Statistics, ValidCollections};

pub fn get_current_date() -> String {
    let now = chrono::Local::now();
    let date = now.format("%Y-%m-%d %H:%M:%S").to_string();
    return date;
}

pub async fn get_global_statistics() -> Result<Json<Value>, StatusCode> {
    let filter = doc! {};
    let options = None;
    let col = get_collection::<Statistics>(ValidCollections::Statistics).await;

    match col.find_one(filter, options).await {
        Ok(doc) => match doc {
            Some(doc) => {
                return Ok(Json(json!(doc)));
            }
            None => {
                // If nothing exist we create a new document
                let new_doc = Statistics {
                    online_users: 0,
                    downloads: 0,
                };
                match col.insert_one(new_doc, None).await {
                    Ok(_) => {
                        println!("Inserted new global stats");

                        let default = Statistics {
                            online_users: 0,
                            downloads: 0,
                        };

                        return Ok(Json(json!(default)));
                    }
                    Err(e) => {
                        create_api_report(
                            ReportType::Error,
                            "Error getting global stats on the server.".to_string(),
                            e.to_string(),
                            get_current_date(),
                            "get_global_statistics".to_string(),
                        )
                        .await;
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
        },
        Err(e) => {
            create_api_report(
                ReportType::Error,
                "Error getting global stats on the server.".to_string(),
                e.to_string(),
                get_current_date(),
                "get_global_statistics".to_string(),
            )
            .await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
}

pub async fn update_global_statistics(
    col: Collection<Statistics>,
    online_users: u32,
    downloads: u32,
) -> Result<StatusCode, String> {
    let filter = doc! {};

    let current_data = match col.find_one(filter, None).await {
        Ok(doc) => match doc {
            Some(doc) => doc,
            None => {
                // If nothing exist we create a new document
                let new_doc = Statistics {
                    online_users: 0,
                    downloads: 0,
                };
                match col.insert_one(new_doc, None).await {
                    Ok(_) => {
                        println!("Inserted new global stats");
                        Statistics {
                            online_users: 0,
                            downloads: 0,
                        }
                    }
                    Err(e) => {
                        create_api_report(
                            ReportType::Error,
                            "Error creating global stats on the server.".to_string(),
                            e.to_string(),
                            get_current_date(),
                            "update_global_statistics".to_string(),
                        )
                        .await;
                        return Err(e.to_string());
                    }
                }
            }
        },
        Err(e) => {
            create_api_report(
                ReportType::Error,
                "Error getting global stats on the server.".to_string(),
                e.to_string(),
                get_current_date(),
                "update_global_statistics".to_string(),
            )
            .await;
            return Err(e.to_string());
        }
    };

    let update = doc! {
        "$set": {
            "online_users": current_data.online_users + online_users,
            "downloads": current_data.downloads + downloads,
        }
    };

    let options = None;
    match col.update_one(doc! {}, update, options).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            create_api_report(
                ReportType::Error,
                "Error updating global stats on the server.".to_string(),
                e.to_string(),
                get_current_date(),
                "update_global_statistics".to_string(),
            )
            .await;
            Err(e.to_string())
        }
    }
}

/// Create a new report in the database
///
/// Useful for tracking errors and bugs from the client
pub async fn create_api_report(
    name: ReportType,
    description: String,
    message: String,
    date: String,
    caller: String,
) {
    let col = get_collection::<Report>(ValidCollections::Reports).await;

    let report = Report {
        name,
        description,
        message,
        date,
        caller,
    };
    match col.insert_one(report, None).await {
        Ok(_) => {
            println!("Inserted report");
        }
        Err(e) => {
            eprintln!("Failed to insert report: {}", e);
        }
    }
}
