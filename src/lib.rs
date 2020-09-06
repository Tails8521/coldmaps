pub mod heatmap;
pub mod heatmap_analyser;

use heatmap_analyser::{Death, HeatmapAnalyser};
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use std::{fs, mem, path::PathBuf, rc::Rc};

use heatmap::{CoordsType, HeatmapType};
use tf_demo_parser::{Demo, DemoParser};

pub fn process_demos(input_paths: Vec<PathBuf>) -> (Vec<Death>, Vec<String>) {
    input_paths
        .par_iter()
        .map(|path| {
            let mut errors = Vec::new();
            let file = match fs::read(path) {
                Ok(file) => file,
                Err(err) => {
                    errors.push(err.to_string());
                    return (Vec::new(), errors);
                }
            };
            let demo = Demo::new(file);
            let state = Default::default();
            if let Err(_err) = DemoParser::new_with_analyser(
                demo.get_stream(),
                HeatmapAnalyser::new(Rc::clone(&state)),
            )
            .parse()
            {
                errors.push(format!(
                    "{}: Demo is corrupted, could only analyse up to tick {}",
                    path.to_string_lossy(),
                    state.borrow().end_tick
                ));
            }
            let deaths = mem::take(&mut state.borrow_mut().deaths);
            (deaths, errors)
        })
        .reduce(
            || (Vec::new(), Vec::new()),
            |mut a, mut b| {
                a.0.append(&mut b.0);
                a.1.append(&mut b.1);
                a
            },
        )
}

pub fn generate_heatmap(
    heatmap_type: HeatmapType,
    deaths: Vec<Death>,
    mut image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    screen_width: u32,
    screen_height: u32,
    pos_x: f32,
    pos_y: f32,
    scale: f32,
    coords_type: CoordsType,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let heatmap_generator = heatmap::HeatMapGenerator::new(
        pos_x,
        pos_y,
        screen_width,
        screen_height,
        scale,
        coords_type,
    );
    heatmap_generator.generate_heatmap(heatmap_type, deaths.iter(), &mut image);
    image
}
