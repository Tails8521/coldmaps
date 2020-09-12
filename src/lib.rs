pub mod filters;
pub mod heatmap;
pub mod heatmap_analyser;

use heatmap_analyser::{Death, HeatmapAnalyser, HeatmapAnalysis};
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use std::{fs, path::PathBuf, rc::Rc};

use heatmap::{CoordsType, HeatmapType};
use tf_demo_parser::{Demo, DemoParser};

#[derive(Debug, Clone, Default)]
pub struct DemoProcessingOutput {
    pub path: PathBuf,
    pub heatmap_analysis: Option<HeatmapAnalysis>,
    pub error: Option<String>,
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
                    }
                }
            };
            let demo = Demo::new(file);
            let heatmap_analysis = Default::default();
            let error = DemoParser::new_with_analyser(demo.get_stream(), HeatmapAnalyser::new(Rc::clone(&heatmap_analysis)))
                .parse()
                .map_err(|_err| {
                    format!(
                        "{}: Demo is corrupted, could only analyse up to tick {}",
                        path.to_string_lossy(),
                        heatmap_analysis.borrow().end_tick
                    )
                })
                .err();
            DemoProcessingOutput {
                path: path.clone(),
                heatmap_analysis: Some(Rc::try_unwrap(heatmap_analysis).unwrap().into_inner()),
                error,
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
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let heatmap_generator = heatmap::HeatMapGenerator::new(pos_x, pos_y, screen_width, screen_height, scale, coords_type);
    heatmap_generator.generate_heatmap(heatmap_type, deaths, &mut image);
    image
}
