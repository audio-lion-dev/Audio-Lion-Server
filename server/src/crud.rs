use axum::{extract::Path, http::StatusCode, Json};
use mongodb::bson::doc;
use serde::Deserialize;

use crate::db::{
    get_collection, IError, Report, ReportType, Statistics, StatusCodeWrapper, ValidCollections,
};

pub fn get_current_date() -> String {
    let now = chrono::Local::now();
    let date = now.format("%Y-%m-%d %H:%M:%S").to_string();
    return date;
}

pub async fn get_global_statistics(
    Path(admin_code): Path<String>,
) -> Result<Json<Statistics>, Json<IError>> {
    // todo - implement a real auth system
    if admin_code != "admin" {
        return Err(Json(IError {
            status_code: StatusCodeWrapper(StatusCode::UNAUTHORIZED),
            error_message: "You shall not pass!".to_string(),
        }));
    }

    let filter = doc! {};
    let options = None;
    let col = get_collection::<Statistics>(ValidCollections::Statistics).await;

    match col.find_one(filter, options).await {
        Ok(doc) => match doc {
            Some(doc) => {
                return Ok(Json(doc));
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

                        return Ok(Json(default));
                    }
                    Err(e) => {
                        match create_api_report(
                            ReportType::Error,
                            "Error getting global stats on the server.".to_string(),
                            e.to_string(),
                            get_current_date(),
                            "get_global_statistics".to_string(),
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(Json(IError {
                                    status_code: StatusCodeWrapper(
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                    ),
                                    error_message: e.to_string(),
                                }));
                            }
                        }
                        return Err(Json(IError {
                            status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                            error_message: e.to_string(),
                        }));
                    }
                }
            }
        },
        Err(e) => {
            match create_api_report(
                ReportType::Error,
                "Error getting global stats on the server.".to_string(),
                e.to_string(),
                get_current_date(),
                "get_global_statistics".to_string(),
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(Json(IError {
                        status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                        error_message: e.to_string(),
                    }));
                }
            }
            return Err(Json(IError {
                status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                error_message: e.to_string(),
            }));
        }
    };
}

#[derive(Deserialize, Debug)]
pub struct OnlineUsersOptions {
    pub inc: Option<bool>,
    pub dec: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateGlobalStatsPayload {
    pub online_users: OnlineUsersOptions,
    pub downloads: Option<bool>,
}

pub async fn update_global_statistics(
    Path(admin_code): Path<String>,
    Json(payload): Json<UpdateGlobalStatsPayload>,
) -> Result<StatusCode, Json<IError>> {
    // todo - implement a real auth system
    if admin_code != "admin" {
        return Err(Json(IError {
            status_code: StatusCodeWrapper(StatusCode::UNAUTHORIZED),
            error_message: "You shall not pass!".to_string(),
        }));
    }

    let filter = doc! {};
    let col = get_collection::<Statistics>(ValidCollections::Statistics).await;

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
                        match create_api_report(
                            ReportType::Error,
                            "Error creating global stats on the server.".to_string(),
                            e.to_string(),
                            get_current_date(),
                            "update_global_statistics".to_string(),
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(Json(IError {
                                    status_code: StatusCodeWrapper(
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                    ),
                                    error_message: e.to_string(),
                                }));
                            }
                        }
                        return Err(Json(IError {
                            status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                            error_message: e.to_string(),
                        }));
                    }
                }
            }
        },
        Err(e) => {
            match create_api_report(
                ReportType::Error,
                "Error getting global stats on the server.".to_string(),
                e.to_string(),
                get_current_date(),
                "update_global_statistics".to_string(),
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(Json(IError {
                        status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                        error_message: e.to_string(),
                    }));
                }
            }
            return Err(Json(IError {
                status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                error_message: e.to_string(),
            }));
        }
    };

    println!("{:?}", current_data);
    println!("{:?}", payload);

    if payload.downloads.is_none()
        && payload.online_users.inc.is_none()
        && payload.online_users.dec.is_none()
    {
        return Err(Json(IError {
            status_code: StatusCodeWrapper(StatusCode::BAD_REQUEST),
            error_message: "None or invalid request data".to_string(),
        }));
    }

    let mut update = doc! {};

    // Make sure inc and dec are not both true
    if let Some(inc) = payload.online_users.inc {
        if let Some(dec) = payload.online_users.dec {
            if inc && dec {
                return Err(Json(IError {
                    status_code: StatusCodeWrapper(StatusCode::BAD_REQUEST),
                    error_message: "inc & dec TRUE in same request".to_string(),
                }));
            }
        }
    }

    if let Some(inc) = payload.online_users.inc {
        if inc {
            update = doc! {
                "$set": {
                    "last_updated": get_current_date()
                },
                "$inc": {
                    "online_users": 1
                }
            };
        }
    }

    if let Some(dec) = payload.online_users.dec {
        if current_data.online_users > 0 && dec {
            update = doc! {
                "$set": {
                    "last_updated": get_current_date()
                },
                "$inc": {
                    "online_users": -1
                }
            };
        } else if current_data.online_users == 0 && dec {
            update = doc! {
                "$set": {
                    "last_updated": get_current_date()
                },
                "$inc": {
                    "online_users": 0
                }
            };
        }
    }

    if let Some(downloads) = payload.downloads {
        if downloads {
            update = doc! {
                "$set": {
                    "last_updated": get_current_date()
                },
                "$inc": {
                    "downloads": 1
                }
            };
        }
    }

    if update.is_empty() {
        return Err(Json(IError {
            status_code: StatusCodeWrapper(StatusCode::BAD_REQUEST),
            error_message: "None or invalid request data".to_string(),
        }));
    }

    let options = None;
    match col.update_one(doc! {}, update, options).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            match create_api_report(
                ReportType::Error,
                "Updating global stats on the server.".to_string(),
                e.to_string(),
                get_current_date(),
                "update_global_statistics".to_string(),
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(Json(IError {
                        status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                        error_message: e.to_string(),
                    }));
                }
            }
            Err(Json(IError {
                status_code: StatusCodeWrapper(StatusCode::INTERNAL_SERVER_ERROR),
                error_message: e.to_string(),
            }))
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
) -> Result<(), String> {
    let col = get_collection::<Report>(ValidCollections::Reports).await;

    let report = Report {
        name,
        description,
        message,
        date,
        caller,
    };
    match col.insert_one(report, None).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
