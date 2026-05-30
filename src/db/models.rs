use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Option<i64>,
    pub name: String,
    pub file_path: String,
    pub category_id: Option<i64>,
    pub volume: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bind {
    pub id: Option<i64>,
    pub track_id: i64,
    pub keyval: u32,
    pub modifiers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
    pub icon_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopSound {
    pub name: String,
    pub url: String,
    pub category: Option<String>,
}
