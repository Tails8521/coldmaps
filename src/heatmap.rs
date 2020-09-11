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
    Lines,
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
            HeatmapType::Lines => write!(f, "Killer -> victim lines"),
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
        // lines
        if heatmap_type == HeatmapType::Lines {
            let line_gradient = Gradient::new(vec![
                LinSrgba::new(0.0, 0.6, 1.0, 1.0),
                LinSrgba::new(0.067, 0.8, 1.0, 1.0),
                LinSrgba::new(0.33, 1.0, 0.33, 1.0),
                LinSrgba::new(1.0, 0.0, 0.0, 1.0),
                LinSrgba::new(1.0, 0.8, 0.0, 1.0),
                // LinSrgba::new(1.0, 0.0, 0.0, 1.0),
                // LinSrgba::new(1.0, 1.0, 0.0, 1.0),
                // LinSrgba::new(0.0, 1.0, 0.0, 1.0),
                // LinSrgba::new(0.0, 1.0, 1.0, 1.0),
                // LinSrgba::new(0.0, 0.0, 1.0, 1.0),
            ]);
            for death in deaths {
                if let (Some(killer_entity), Some(victim_entity)) =
                    (&death.killer_entity_state, &death.victim_entity_state)
                {
                    let killer_coords = self.game_coords_to_screen_coords(
                        killer_entity.position.x,
                        killer_entity.position.y,
                    );
                    let victim_coords = self.game_coords_to_screen_coords(
                        victim_entity.position.x,
                        victim_entity.position.y,
                    );
                    let points: Vec<((i32, i32), f32)> =
                        line_drawing::XiaolinWu::<f32, i32>::new(killer_coords, victim_coords)
                            .collect();

                    // this is needed because the line drawing algorithm doesn't always go in the start-end order, we need to check what order was used and invert the gradient as needed
                    let (first_point_x, first_point_y) = match points.get(0) {
                        Some(((first_point_x, first_point_y), _)) => {
                            (*first_point_x as f32, *first_point_y as f32)
                        }
                        None => continue,
                    };
                    let dist_killer_x = killer_coords.0 - first_point_x;
                    let dist_killer_y = killer_coords.1 - first_point_y;
                    let dist_victim_x = victim_coords.0 - first_point_x;
                    let dist_victim_y = victim_coords.1 - first_point_y;
                    let invert_gradient = dist_killer_x * dist_killer_x
                        + dist_killer_y * dist_killer_y
                        > dist_victim_x * dist_victim_x + dist_victim_y * dist_victim_y;

                    let len = points.len() as f32;
                    for (index, ((x, y), alpha)) in points.iter().enumerate() {
                        let (x, y) = (*x, *y);
                        if y < 0 || y >= image.height() as i32 || x < 0 || x >= image.width() as i32
                        {
                            continue;
                        }
                        let color = if invert_gradient {
                            line_gradient.get(1.0 - ((index + 1) as f32 / len))
                        } else {
                            line_gradient.get((index + 1) as f32 / len)
                        };
                        let pixel = image.get_pixel_mut(x as u32, y as u32);
                        if let [r, g, b] = pixel.channels() {
                            *pixel = Rgb::from([
                                ((alpha * color.red + (1.0 - alpha) * (*r as f32 / 255.0)) * 255.0)
                                    as u8,
                                ((alpha * color.green + (1.0 - alpha) * (*g as f32 / 255.0))
                                    * 255.0) as u8,
                                ((alpha * color.blue + (1.0 - alpha) * (*b as f32 / 255.0)) * 255.0)
                                    as u8,
                            ]);
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
            return;
        }

        // heatmap
        let heatmap_gradient = Gradient::new(vec![
            // LinSrgba::new(0.0, 0.0, 0.0, 0.0),
            LinSrgba::new(0.0, 0.0, 1.0, 0.0),
            LinSrgba::new(0.0, 1.0, 1.0, 0.25),
            LinSrgba::new(0.0, 1.0, 0.0, 0.5),
            LinSrgba::new(1.0, 1.0, 0.0, 0.75),
            LinSrgba::new(1.0, 0.0, 0.0, 1.0),
            // LinSrgba::new(1.0, 1.0, 1.0, 1.0),
        ]);
        let nb_pixels = (image.width() * image.height()) as usize;
        let mut intensities = Vec::with_capacity(nb_pixels);
        intensities.resize_with(nb_pixels, || 0.0);
        let mut max_intensity = f32::NEG_INFINITY;
        for death in deaths {
            let entity_state = match heatmap_type {
                HeatmapType::VictimPosition => &death.victim_entity_state,
                HeatmapType::KillerPosition => &death.killer_entity_state,
                HeatmapType::Lines => unreachable!(),
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
            let heat_color = heatmap_gradient.get(intensity);
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
