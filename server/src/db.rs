use axum::http::StatusCode;
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    Reports,
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

/// Global statistics for the app
/// There will only be one document in the collection
#[derive(Debug, Deserialize, Serialize)]
pub struct Statistics {
    pub online_users: u32,
    pub downloads: u32,
}

/// The different types of reports that can be created on the server
#[derive(Debug, Deserialize, Serialize)]
pub enum ReportType {
    Bug,
    Error,
    Other,
}

/// A report that can be created on the server to help with debugging and tracking issues
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

#[derive(Debug, Deserialize, Serialize)]
pub struct IError {
    pub status_code: StatusCodeWrapper,
    pub error_message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusCodeWrapper(pub StatusCode);

impl Serialize for StatusCodeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_u16().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StatusCodeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = u16::deserialize(deserializer)?;
        Ok(StatusCodeWrapper(StatusCode::from_u16(code).map_err(
            |_| serde::de::Error::custom(format!("Invalid status code: {}", code)),
        )?))
    }
}