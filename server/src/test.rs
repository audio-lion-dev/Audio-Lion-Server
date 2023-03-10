#![allow(unused_imports)]

use crate::crud::{
    get_current_date, get_global_statistics, update_global_statistics, OnlineUsersOptions,
    UpdateGlobalStatsPayload,
};
use axum::{extract::Path, http::StatusCode, Json};

#[tokio::test]
async fn test_get_current_date() {
    let date = get_current_date();
    assert!(date.contains("2023"));
}

#[tokio::test]
async fn test_get_global_statistics_unauthorized() {
    let admin_code = "user";
    let result = get_global_statistics(Path(admin_code.to_string())).await;

    assert_eq!(result.unwrap_err().status_code.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_global_statistics_success() {
    let admin_code = "admin";
    let result = get_global_statistics(Path(admin_code.to_string())).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_global_statistics_unauthorized() {
    let admin_code = "user";
    let payload = UpdateGlobalStatsPayload {
        online_users: OnlineUsersOptions {
            inc: Some(true),
            dec: None,
        },
        downloads: None,
    };
    let result = update_global_statistics(Path(admin_code.to_string()), Json(payload)).await;

    assert_eq!(result.unwrap_err().status_code.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_global_statistics_success() {
    let admin_code = "admin";
    let payload = UpdateGlobalStatsPayload {
        online_users: OnlineUsersOptions {
            inc: Some(true),
            dec: None,
        },
        downloads: None,
    };
    let result = update_global_statistics(Path(admin_code.to_string()), Json(payload)).await;

    assert_eq!(result.unwrap(), StatusCode::OK);
}
