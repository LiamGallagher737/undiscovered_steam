use anyhow::{anyhow, Context, Result};
use futures::future::join_all;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub async fn search(
    client: &reqwest::Client,
    max_price: u16,
    term: String,
) -> Result<Vec<SearchItem>> {
    let res = client
        .get("https://store.steampowered.com/search/results")
        .query(&[
            ("term", term),
            (
                "maxprice",
                if max_price > 0 {
                    max_price.to_string()
                } else {
                    "free".to_string()
                },
            ),
            ("json", "1".into()),
            ("category1", "998".into()),
            ("sort_by", "Released_DESC".into()),
        ])
        .send()
        .await?
        .json::<SearchResult>()
        .await?;

    Ok(res.items)
}

pub async fn app(client: &reqwest::Client, id: String) -> Result<App> {
    let data_req = client
        .get("https://store.steampowered.com/api/appdetails")
        .query(&[("appids", &id)])
        .send();

    let reviews_req = client
        .get(format!("https://store.steampowered.com/appreviews/{id}"))
        .query(&[("json", "1"), ("purchase_type", "all")])
        .send();

    let mut res = join_all([data_req, reviews_req]).await;
    let data_res = res.remove(0)?;
    let reviews_res = res.remove(0)?;

    if data_res.status() == StatusCode::TOO_MANY_REQUESTS
        || reviews_res.status() == StatusCode::TOO_MANY_REQUESTS
        || data_res.status() == StatusCode::FORBIDDEN
        || reviews_res.status() == StatusCode::FORBIDDEN
    {
        return Err(anyhow!("Too Many Requests"));
    }

    let data = {
        let mut res = data_res.json::<HashMap<String, Value>>().await?;
        let value = res.remove(&id).context("Invalid response")?;
        serde_json::from_value::<AppResult>(value)?.data
    };

    let reviews = {
        let res = reviews_res.json::<AppReviewsResult>().await?;
        res.query_summary
    };

    Ok(App { data, reviews })
}

// Search Result

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub desc: String,
    pub items: Vec<SearchItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchItem {
    pub name: String,
    pub logo: String,
}

// App

pub struct App {
    pub data: AppData,
    pub reviews: AppReviews,
}

// App Result

#[derive(Serialize, Deserialize, Debug)]
pub struct AppResult {
    pub success: bool,
    pub data: AppData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppData {
    #[serde(rename = "type")]
    pub app_type: String,
    pub name: String,
    pub steam_appid: u32,
    pub required_age: u32,
    pub is_free: bool,
    pub supported_languages: String,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub price_overview: PriceOverview,
    pub platforms: Platforms,
    pub categories: Vec<Category>,
    pub genres: Vec<Genre>,
    pub release_date: ReleaseDate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceOverview {
    pub currency: String,
    #[serde(rename = "initial")]
    pub initial_price: u32,
    #[serde(rename = "final")]
    pub final_price: u32,
    pub discount_percent: u32,
    pub initial_formatted: String,
    pub final_formatted: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Platforms {
    pub windows: bool,
    pub mac: bool,
    pub linux: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub id: u32,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Genre {
    pub id: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReleaseDate {
    pub coming_soon: bool,
    pub date: String,
}

// App Reviews Result

#[derive(Serialize, Deserialize, Debug)]
pub struct AppReviewsResult {
    pub success: i64,
    pub query_summary: AppReviews,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppReviews {
    pub num_reviews: usize,
    pub review_score: u8,
    pub review_score_desc: String,
    pub total_positive: usize,
    pub total_negative: usize,
    pub total_reviews: usize,
}
