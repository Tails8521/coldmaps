use coldmaps::heatmap::LEVELOVERVIEW_SCALE_MULTIPLIER;
use image::io::Reader;
use image::{ImageFormat, RgbImage};
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Cursor;

pub async fn get_boundary(map: &str) -> Result<Option<(f32, f32, f32)>, reqwest::Error> {
    let cache: HashMap<String, Boundary> = reqwest::get("https://github.com/demostf/demos.tf/raw/master/src/Analyse/mapboundries.json")
        .await?
        .json()
        .await?;
    Ok(cache.get(map).map(|boundary| {
        let x = (boundary.max.x + boundary.min.x) / 2.0;
        let y = (boundary.max.y + boundary.min.y) / 2.0;
        let scale = (boundary.max.y - boundary.min.y) / LEVELOVERVIEW_SCALE_MULTIPLIER / 2.0;
        (x, y, scale)
    }))
}

pub async fn get_image(map: &str) -> Result<Option<RgbImage>, reqwest::Error> {
    let result = reqwest::get(&format!("https://github.com/demostf/demos.tf/raw/master/src/images/leveloverview/dist/{}.png", map)).await?;
    if result.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let body = result.bytes().await?;
    let reader = Cursor::new(body);
    let mut reader = Reader::new(reader);
    reader.set_format(ImageFormat::Png);

    Ok(reader.decode().ok().map(|image| image.into_rgb8()))
}

#[derive(Debug, Deserialize)]
struct Boundary {
    #[serde(rename = "boundaryMin")]
    min: Point,
    #[serde(rename = "boundaryMax")]
    max: Point,
}

#[derive(Debug, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
