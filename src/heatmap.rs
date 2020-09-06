use crate::heatmap_analyser::Death;
use image::{ImageBuffer, Pixel, Rgb};
use palette::{Gradient, LinSrgba};
use std::fmt::Display;

const LEVELOVERVIEW_SCALE_MULTIPLIER: f32 = 512.0;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CoordsType {
    ShowPos,
    Console,
}

impl Default for CoordsType {
    fn default() -> Self {
        Self::ShowPos
    }
}

impl Display for CoordsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordsType::ShowPos => write!(f, "cl_showpos"),
            CoordsType::Console => write!(f, "Console"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HeatmapType {
    VictimPosition,
    KillerPosition,
}

impl Default for HeatmapType {
    fn default() -> Self {
        Self::VictimPosition
    }
}

impl Display for HeatmapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeatmapType::VictimPosition => write!(f, "Victim position"),
            HeatmapType::KillerPosition => write!(f, "Killer position"),
        }
    }
}

#[derive(Debug)]
struct HeatMapParameters {
    screen_width: f32,
    screen_height: f32,
    left_x: f32,
    right_x: f32,
    top_y: f32,
    bottom_y: f32,
}

#[derive(Debug)]
pub struct HeatMapGenerator {
    params: HeatMapParameters,
}

impl HeatMapGenerator {
    pub fn new(
        pos_x: f32,
        pos_y: f32,
        screen_width: u32,
        screen_height: u32,
        scale: f32,
        coords_type: CoordsType,
    ) -> Self {
        let screen_width = screen_width as f32;
        let screen_height = screen_height as f32;
        let aspect_ratio = screen_width / screen_height;
        match coords_type {
            CoordsType::ShowPos => Self {
                params: HeatMapParameters {
                    screen_width,
                    screen_height,
                    left_x: pos_x - scale * LEVELOVERVIEW_SCALE_MULTIPLIER * aspect_ratio,
                    right_x: pos_x + scale * LEVELOVERVIEW_SCALE_MULTIPLIER * aspect_ratio,
                    top_y: pos_y + scale * LEVELOVERVIEW_SCALE_MULTIPLIER,
                    bottom_y: pos_y - scale * LEVELOVERVIEW_SCALE_MULTIPLIER,
                },
            },
            CoordsType::Console => Self {
                params: HeatMapParameters {
                    screen_width,
                    screen_height,
                    left_x: pos_x,
                    right_x: pos_x + scale * LEVELOVERVIEW_SCALE_MULTIPLIER * aspect_ratio * 2.0,
                    top_y: pos_y,
                    bottom_y: pos_y - scale * LEVELOVERVIEW_SCALE_MULTIPLIER * 2.0,
                },
            },
        }
    }

    pub fn generate_heatmap<'a>(
        &self,
        heatmap_type: HeatmapType,
        deaths: impl IntoIterator<Item = &'a Death>,
        image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) {
        let nb_pixels = (image.width() * image.height()) as usize;
        let mut intensities = Vec::with_capacity(nb_pixels);
        intensities.resize_with(nb_pixels, || 0.0);
        let mut max_intensity = 0.0;
        let gradient = Gradient::new(vec![
            // LinSrgba::new(0.0, 0.0, 0.0, 0.0),
            LinSrgba::new(0.0, 0.0, 1.0, 0.0),
            LinSrgba::new(0.0, 1.0, 1.0, 0.25),
            LinSrgba::new(0.0, 1.0, 0.0, 0.5),
            LinSrgba::new(1.0, 1.0, 0.0, 0.75),
            LinSrgba::new(1.0, 0.0, 0.0, 1.0),
            // LinSrgba::new(1.0, 1.0, 1.0, 1.0),
        ]);
        for death in deaths {
            let entity_state = match heatmap_type {
                HeatmapType::VictimPosition => &death.victim_entity_state,
                HeatmapType::KillerPosition => &death.killer_entity_state,
            };
            if let Some(entity_state) = entity_state {
                let game_coords = entity_state.position;
                let (x_f, y_f) = self.game_coords_to_screen_coords(game_coords.x, game_coords.y);
                let x_i = x_f.round() as i32;
                let y_i = y_f.round() as i32;
                for y_offset in -10..10 {
                    let y = y_i + y_offset;
                    if y < 0 || y >= image.height() as i32 {
                        continue;
                    }
                    for x_offset in -10..10 {
                        let x = x_i + x_offset;
                        if x < 0 || x >= image.width() as i32 {
                            continue;
                        }
                        let x_dist = x_f - x as f32;
                        let y_dist = y_f - y as f32;
                        let dist = (x_dist * x_dist + y_dist * y_dist).sqrt();
                        let intensity = gaussian(dist, 5.0);
                        let intensity_index = (y * image.width() as i32 + x) as usize;
                        intensities[intensity_index] += intensity;
                        if intensities[intensity_index] > max_intensity {
                            max_intensity = intensities[intensity_index];
                        }
                    }
                }
            }
        }
        for (pixel, base_intensity) in image.pixels_mut().zip(intensities) {
            let intensity = base_intensity / max_intensity;
            let heat_color = gradient.get(intensity);
            if let [r, g, b] = pixel.channels() {
                *pixel = Rgb::from([
                    ((heat_color.alpha * heat_color.red
                        + (1.0 - heat_color.alpha) * (*r as f32 / 255.0))
                        * 255.0) as u8,
                    ((heat_color.alpha * heat_color.green
                        + (1.0 - heat_color.alpha) * (*g as f32 / 255.0))
                        * 255.0) as u8,
                    ((heat_color.alpha * heat_color.blue
                        + (1.0 - heat_color.alpha) * (*b as f32 / 255.0))
                        * 255.0) as u8,
                ]);
            } else {
                unreachable!();
            }
        }
    }

    fn game_coords_to_screen_coords(&self, x: f32, y: f32) -> (f32, f32) {
        let p = &self.params;
        (
            (x - p.left_x) / (p.right_x - p.left_x) * p.screen_width,
            (1.0 - (y - p.top_y)) / (p.top_y - p.bottom_y) * p.screen_height,
        )
    }
}

fn gaussian(x: f32, std_dev: f32) -> f32 {
    (-((x * x) / (2.0 * std_dev * std_dev))).exp()
}
