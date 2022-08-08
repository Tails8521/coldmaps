pub mod filters;
pub mod heatmap;
pub mod heatmap_analyser;

use heatmap_analyser::{Death, HeatmapAnalyser, HeatmapAnalysis};
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use std::{fs, path::PathBuf};

use heatmap::{CoordsType, HeatmapType};
use tf_demo_parser::{Demo, DemoParser};

#[derive(Debug, Clone, Default)]
pub struct DemoProcessingOutput {
    pub path: PathBuf,
    pub heatmap_analysis: Option<HeatmapAnalysis>,
    pub error: Option<String>,
    pub map: String,
}

pub fn process_demos(inputs: Vec<PathBuf>) -> Vec<DemoProcessingOutput> {
    inputs
        .par_iter()
        .map(|path| {
            let file = match fs::read(&path) {
                Ok(file) => file,
                Err(err) => {
                    return DemoProcessingOutput {
                        path: path.clone(),
                        heatmap_analysis: None,
                        error: Some(err.to_string()),
                        map: String::new(),
                    }
                }
            };
            let demo = Demo::owned(file);
            let (header, mut ticker) = DemoParser::new_with_analyser(demo.get_stream(), HeatmapAnalyser::default()).ticker().unwrap();
            loop {
                match ticker.tick() {
                    Ok(true) => continue,
                    Ok(false) => {
                        break DemoProcessingOutput {
                            path: path.clone(),
                            heatmap_analysis: Some(ticker.into_state()),
                            error: None,
                            map: header.map,
                        }
                    }
                    Err(_err) => {
                        let heatmap_analysis = ticker.into_state();
                        let error = Some(format!(
                            "{}: Demo is corrupted, could only analyse up to tick {}",
                            path.to_string_lossy(),
                            heatmap_analysis.current_tick
                        ));
                        break DemoProcessingOutput {
                            path: path.clone(),
                            heatmap_analysis: Some(heatmap_analysis),
                            map: header.map,
                            error,
                        };
                    }
                };
            }
        })
        .collect()
}

pub fn generate_heatmap<'a>(
    heatmap_type: HeatmapType,
    deaths: impl IntoIterator<Item = &'a Death>,
    mut image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    screen_width: u32,
    screen_height: u32,
    pos_x: f32,
    pos_y: f32,
    scale: f32,
    coords_type: CoordsType,
    radius: f32,
    intensity: Option<f32>,
    use_sentry_position: bool,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let heatmap_generator = heatmap::HeatMapGenerator::new(pos_x, pos_y, screen_width, screen_height, scale, coords_type, radius, intensity, use_sentry_position);
    heatmap_generator.generate_heatmap(heatmap_type, deaths, &mut image);
    image
}
