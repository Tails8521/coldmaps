use chat::format_chat_messages;
use coldmaps::*;

mod chat;
mod demo_player;
mod demostf;
mod gui_filters;
mod style;

use filters::{FilterTrait, OrderedOperator, Property, PropertyOperator};
use gui_filters::{FilterType, FiltersPane};
use heatmap::{CoordsType, HeatmapType};
use heatmap_analyser::{HeatmapAnalysis, Team};
use iced::{
    alignment, button, executor, image::Handle, pane_grid, scrollable, slider, text_input, window, Application, Button, Checkbox, Column, Command, Container, Element, Font, Image,
    Length, Point, Radio, Rectangle, Row, Scrollable, Settings, Size, Slider, Subscription, Text, TextInput,
};
use image::{io::Reader, ImageBuffer, Pixel, Rgb, RgbImage};
use pane_grid::{Axis, Pane};
use rfd::AsyncFileDialog;
use std::{mem, path::PathBuf, time::Instant};

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};

fn icon(unicode: char, color: [f32; 3]) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .color(color)
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
}

fn delete_icon() -> Text {
    icon('\u{F1F8}', [1.0, 0.0, 0.0])
}

fn chat_preview_icon() -> Text {
    icon('\u{E802}', [1.0, 1.0, 1.0])
}

pub fn main() -> Result<(), iced::Error> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(arg) = args.get(1) {
        if arg == "--demoplayer" {
            demo_player::run().unwrap();
            return Ok(());
        }
    }
    App::run(Settings {
        antialiasing: true,
        window: window::Settings {
            size: (1280, 720),
            ..Default::default()
        },
        ..Default::default()
    })
}

struct App {
    pane_grid_state: pane_grid::State<PaneState>,
    theme: style::Theme,
    busy: bool,
    // TODO visual indicator?
    dropped_files: Vec<PathBuf>,
    demos_pane: Pane,
    filters_pane: Pane,
    settings_pane: Pane,
    preview_pane: Pane,
    log_pane: Pane,
}

struct HeatmapImage {
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    image_with_heatmap_overlay: ImageBuffer<Rgb<u8>, Vec<u8>>,
    handle: Handle,
}

#[derive(Debug)]
struct DemoFile {
    _path: PathBuf,
    file_name: String,
    delete_button: button::State,
    chat_preview_button: button::State,
    heatmap_analysis: HeatmapAnalysis,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEventOccurred(iced_native::Event),
    PaneResized(pane_grid::ResizeEvent),
    DemoRemoved(usize),
    ChatPreview(usize),
    ThemeChanged(style::Theme),
    CoordsTypeChanged(CoordsType),
    HeatmapTypeChanged(HeatmapType),
    XPosInputChanged(String),
    YPosInputChanged(String),
    ScaleInputChanged(String),
    AutoIntensityCheckboxToggled(bool),
    UseSentryPositionCheckboxToggled(bool),
    IntensityChanged(f32),
    RadiusChanged(f32),
    DesaturateChanged(f32),
    ProcessDemosDone(TimedResult<Vec<DemoProcessingOutput>>),
    ExportImagePressed,
    ImageNameSelected(Option<PathBuf>),
    EndOfDemoFilesDrop(()),
    MapSet(String),
    DemosTFImageLoader(Option<(RgbImage, f32, f32, f32)>),
    LevelImageSet(RgbImage),
    AddFilter,
    FilterSelected(usize, FilterType),
    ClassIconClicked(usize, usize),
    BluTeamClicked(usize),
    RedTeamClicked(usize),
    OrderedOperatorSelected(usize, OrderedOperator),
    PropertyOperatorSelected(usize, PropertyOperator),
    PropertySelected(usize, Property),
    FilterTextInputChanged(usize, String),
    FilterRemoved(usize),
}

#[derive(Debug, Clone)]
struct TimedResult<T: std::fmt::Debug + Clone> {
    result: T,
    time_elapsed: f32,
}

enum PaneState {
    DemoList(DemoList),
    FiltersPane(FiltersPane),
    SettingsPane(SettingsPane),
    Preview(Preview),
    LogPane(LogPane),
}

impl PaneState {
    fn view(&mut self) -> Element<Message> {
        match self {
            PaneState::DemoList(pane) => pane.view(),
            PaneState::FiltersPane(pane) => pane.view(),
            PaneState::SettingsPane(pane) => pane.view(),
            PaneState::Preview(pane) => pane.view(),
            PaneState::LogPane(pane) => pane.view(),
        }
    }
}

#[derive(Default)]
struct DemoList {
    theme: style::Theme,
    busy: bool,
    scroll_state: scrollable::State,
    demo_files: Vec<DemoFile>,
}

impl DemoList {
    fn view(&mut self) -> Element<Message> {
        let (demos_list, style): (Element<_>, _) = if self.demo_files.is_empty() {
            (
                Container::new(
                    Text::new("Drag and drop demo files to add them")
                        .width(Length::Fill)
                        .size(24)
                        .horizontal_alignment(alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .into(),
                style::ResultContainer::Error,
            )
        } else {
            let theme = self.theme;
            (
                self.demo_files
                    .iter_mut()
                    .enumerate()
                    .fold(Column::new().spacing(10), |column, (index, demo)| {
                        let delete_button = Button::new(&mut demo.delete_button, delete_icon()).style(theme).on_press(Message::DemoRemoved(index));
                        let chat_preview_button = Button::new(&mut demo.chat_preview_button, chat_preview_icon())
                            .style(theme)
                            .on_press(Message::ChatPreview(index));
                        let row = Row::new()
                            .spacing(2)
                            .push(delete_button)
                            .push(chat_preview_button)
                            .push(Text::new(&demo.file_name).size(20));
                        column.push(row)
                    })
                    .into(),
                style::ResultContainer::Ok,
            )
        };
        let demos_scroll = Scrollable::new(&mut self.scroll_state).push(demos_list).width(Length::Fill).height(Length::Fill);

        let result_container = Container::new(demos_scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10)
            .style(style);

        Container::new(result_container).padding(4).width(Length::Fill).height(Length::Fill).into()
    }
}

struct SettingsPane {
    theme: style::Theme,
    busy: bool,
    scroll_state: scrollable::State,
    x_pos_input_state: text_input::State,
    x_pos_input: String,
    x_pos: Option<f32>,
    y_pos_input_state: text_input::State,
    y_pos_input: String,
    y_pos: Option<f32>,
    scale_input_state: text_input::State,
    scale_input: String,
    scale: Option<f32>,
    export_image_button: button::State,
    image_ready: bool,
    coords_type: CoordsType,
    heatmap_type: HeatmapType,
    auto_intensity: bool,
    use_sentry_position: bool,
    intensity_state: slider::State,
    intensity: f32,
    radius_state: slider::State,
    radius: f32,
    desaturate_state: slider::State,
    desaturate: f32,
}

impl Default for SettingsPane {
    fn default() -> Self {
        Self {
            theme: Default::default(),
            busy: Default::default(),
            scroll_state: Default::default(),
            x_pos_input_state: Default::default(),
            x_pos_input: "0".into(),
            x_pos: Some(0.0),
            y_pos_input_state: Default::default(),
            y_pos_input: "0".into(),
            y_pos: Some(0.0),
            scale_input_state: Default::default(),
            scale_input: Default::default(),
            scale: Default::default(),
            export_image_button: Default::default(),
            image_ready: Default::default(),
            coords_type: Default::default(),
            heatmap_type: Default::default(),
            auto_intensity: true,
            use_sentry_position: true,
            intensity_state: Default::default(),
            intensity: 50.0,
            radius_state: Default::default(),
            radius: 50.0,
            desaturate_state: Default::default(),
            desaturate: 0.0,
        }
    }
}

impl SettingsPane {
    fn view(&mut self) -> Element<Message> {
        let style = if self.x_pos.is_some() && self.y_pos.is_some() && self.scale.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let choose_theme = style::Theme::ALL.iter().fold(Column::new().spacing(10).push(Text::new("Theme:")), |column, theme| {
            column.push(Radio::new(*theme, &format!("{:?}", theme), Some(self.theme), Message::ThemeChanged).style(self.theme))
        });
        let choose_coords_type = [CoordsType::ShowPos, CoordsType::Console]
            .iter()
            .fold(Column::new().spacing(10).push(Text::new("Coordinates origin:")), |column, coords_type| {
                column.push(Radio::new(*coords_type, &format!("{}", coords_type), Some(self.coords_type), Message::CoordsTypeChanged).style(self.theme))
            });
        let choose_heatmap_type = [HeatmapType::VictimPosition, HeatmapType::KillerPosition, HeatmapType::Lines]
            .iter()
            .fold(Column::new().spacing(10).push(Text::new("Heatmap type:")), |column, heatmap_type| {
                column.push(Radio::new(*heatmap_type, &format!("{}", heatmap_type), Some(self.heatmap_type), Message::HeatmapTypeChanged).style(self.theme))
            });

        let x_pos_input = TextInput::new(&mut self.x_pos_input_state, "Camera x position", &self.x_pos_input, Message::XPosInputChanged).style(self.theme);
        let x_pos_style = if self.x_pos.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let x_pos_border = Container::new(x_pos_input).padding(3).width(Length::Fill).style(x_pos_style);

        let y_pos_input = TextInput::new(&mut self.y_pos_input_state, "Camera y position", &self.y_pos_input, Message::YPosInputChanged).style(self.theme);
        let y_pos_style = if self.y_pos.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let y_pos_border = Container::new(y_pos_input).padding(3).width(Length::Fill).style(y_pos_style);

        let scale_input = TextInput::new(&mut self.scale_input_state, "Camera scale", &self.scale_input, Message::ScaleInputChanged).style(self.theme);
        let scale_style = if self.scale.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let scale_border = Container::new(scale_input).padding(3).width(Length::Fill).style(scale_style);
        let mut export_image_button = Button::new(&mut self.export_image_button, Text::new("Export image"))
            .padding(10)
            .style(self.theme)
            .width(Length::Fill);
        if self.image_ready {
            export_image_button = export_image_button.on_press(Message::ExportImagePressed);
        }

        let coords_label = match self.coords_type {
            CoordsType::ShowPos => "Camera coordinates (use cl_showpos)",
            CoordsType::Console => "Camera coordinates (use the console)",
        };

        let mut heatmap_options = Column::new().spacing(10);
        if self.heatmap_type != HeatmapType::Lines {
            let intensity_text = if self.auto_intensity {
                "Heatmap intensity: Auto".into()
            } else {
                format!("Heatmap intensity: {:.2}", self.intensity / 100.0)
            };
            let intensity_checkbox = Container::new(Checkbox::new(self.auto_intensity, "Auto", Message::AutoIntensityCheckboxToggled).style(self.theme))
                .align_x(alignment::Horizontal::Right)
                .width(Length::Fill);
            let intensity_label = Row::new().spacing(10).push(Text::new(&intensity_text)).push(intensity_checkbox);
            heatmap_options = heatmap_options.push(intensity_label);
            if !self.auto_intensity {
                let intensity_slider = Slider::new(&mut self.intensity_state, 1.0..=100.0, self.intensity, Message::IntensityChanged).style(self.theme);
                heatmap_options = heatmap_options.push(intensity_slider);
            }
            let radius_label = Row::new().spacing(10).push(Text::new(&format!("Heatmap radius: {:.1}", self.radius / 10.0)));
            let radius_slider = Slider::new(&mut self.radius_state, 1.0..=100.0, self.radius, Message::RadiusChanged).style(self.theme);
            heatmap_options = heatmap_options.push(radius_label).push(radius_slider);
        }
        let desaturate_label = Row::new().spacing(10).push(Text::new(&format!("Desaturate level overview: {:.0}%", self.desaturate)));
        let desaturate_slider = Slider::new(&mut self.desaturate_state, 0.0..=100.0, self.desaturate, Message::DesaturateChanged).style(self.theme);
        heatmap_options = heatmap_options.push(desaturate_label).push(desaturate_slider);
        let use_sentry_position_checkbox =
            Checkbox::new(self.use_sentry_position, "Use sentry position for sentry kills", Message::UseSentryPositionCheckboxToggled).style(self.theme);
        heatmap_options = heatmap_options.push(use_sentry_position_checkbox);

        let settings_content: Element<_> = Column::new()
            .push(choose_heatmap_type)
            .push(Text::new(coords_label))
            .push(x_pos_border)
            .push(y_pos_border)
            .push(Text::new("cl_leveloverview scale"))
            .push(scale_border)
            .push(export_image_button)
            .push(heatmap_options)
            .push(choose_coords_type)
            .push(choose_theme)
            .spacing(10)
            .into();

        let scroll = Scrollable::new(&mut self.scroll_state).push(settings_content);

        let result_container = Container::new(scroll).width(Length::Fill).height(Length::Fill).padding(10).style(style);

        Container::new(result_container).padding(4).width(Length::Fill).height(Length::Fill).into()
    }
}

#[derive(Default)]
struct Preview {
    theme: style::Theme,
    heatmap_image: Option<HeatmapImage>,
}

impl Preview {
    fn view(&mut self) -> Element<Message> {
        let (image, style): (Element<_>, _) = if let Some(heatmap_image) = &self.heatmap_image {
            (Image::new(heatmap_image.handle.clone()).into(), style::ResultContainer::Ok)
        } else {
            (
                Text::new("Drag and drop the level overview screenshot to use it")
                    .width(Length::Fill)
                    .size(24)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .into(),
                style::ResultContainer::Error,
            )
        };

        let column = Column::new().push(image);

        let result_container = Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10)
            .style(style);

        Container::new(result_container).padding(4).width(Length::Fill).height(Length::Fill).into()
    }
}

#[derive(Default)]
struct LogPane {
    theme: style::Theme,
    scroll_state: scrollable::State,
    log: String,
}

impl LogPane {
    fn view(&mut self) -> Element<Message> {
        let log = Text::new(&self.log);

        let demos_scroll = Scrollable::new(&mut self.scroll_state).push(log).width(Length::Fill);

        let result_container = Container::new(demos_scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .style(style::ResultContainer::Ok);

        Container::new(result_container).padding(4).width(Length::Fill).height(Length::Fill).into()
    }

    fn log(&mut self, message: &str) {
        println!("{}", message);
        self.log.push_str(message);
        self.log.push('\n');
        // TODO replace this by a cleaner way to scroll down once possible
        self.scroll_state.scroll_to(
            1.0,
            Rectangle::new(Point::new(0.0, 0.0), Size::new(10000.0, 10000.0)),
            Rectangle::new(Point::new(0.0, 0.0), Size::new(100000.0, 100000.0)),
        );
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        let (mut pane_grid_state, demos_pane) = pane_grid::State::new(PaneState::DemoList(Default::default()));
        let (preview_pane, demos_preview_split) = pane_grid_state.split(Axis::Vertical, &demos_pane, PaneState::Preview(Default::default())).unwrap();
        let (filters_pane, demos_filter_split) = pane_grid_state.split(Axis::Horizontal, &demos_pane, PaneState::FiltersPane(Default::default())).unwrap();
        let (settings_pane, filters_settings_split) = pane_grid_state.split(Axis::Horizontal, &filters_pane, PaneState::SettingsPane(Default::default())).unwrap();
        let (log_pane, preview_log_split) = pane_grid_state.split(Axis::Horizontal, &preview_pane, PaneState::LogPane(Default::default())).unwrap();
        pane_grid_state.resize(&demos_preview_split, 0.397);
        pane_grid_state.resize(&demos_filter_split, 0.18);
        pane_grid_state.resize(&filters_settings_split, 0.294);
        pane_grid_state.resize(&preview_log_split, 0.8);
        (
            App {
                busy: false,
                dropped_files: Default::default(),
                pane_grid_state,
                theme: Default::default(),
                demos_pane,
                preview_pane,
                filters_pane,
                settings_pane,
                log_pane,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("Coldmaps {}", VERSION.unwrap_or_default())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::WindowEventOccurred(iced_native::Event::Window(iced_native::window::Event::FileDropped(path))) => {
                if !path.is_file() {
                    return Command::none();
                }
                let file_name = path.file_name().unwrap().to_string_lossy().to_string(); // The path can't be .. at that point
                let file_name_lowercase = file_name.to_lowercase();
                if file_name_lowercase.ends_with(".dem") {
                    self.dropped_files.push(path);
                    return Command::perform(async {}, Message::EndOfDemoFilesDrop);
                } else {
                    // try to load it as an image
                    if let Ok(reader) = Reader::open(&path) {
                        if let Ok(image) = reader.decode() {
                            let image = image.into_rgb8();
                            return Command::perform(async { image }, Message::LevelImageSet);
                        }
                    }
                }
            }
            Message::WindowEventOccurred(_) => {}
            Message::MapSet(map) => {
                return Command::perform(
                    async move {
                        let (x, y, scale) = match demostf::get_boundary(&map).await {
                            Ok(boundaries) => boundaries,
                            Err(e) => {
                                eprintln!("Error while fetching demostf boundaries {}", e);
                                None
                            }
                        }?;
                        let image = match demostf::get_image(&map).await {
                            Ok(image) => image,
                            Err(e) => {
                                eprintln!("Error while fetching demostf image {}", e);
                                None
                            }
                        }?;
                        Some((image, x, y, scale))
                    },
                    Message::DemosTFImageLoader,
                );
            }
            Message::LevelImageSet(image) => {
                let image_with_heatmap_overlay = image.clone();
                let handle = image_to_handle(&image);
                self.get_preview_pane_mut().heatmap_image.replace(HeatmapImage {
                    image,
                    image_with_heatmap_overlay,
                    handle,
                });
                self.get_settings_pane_mut().image_ready = true;
                self.try_generate_heatmap();
            }
            Message::DemosTFImageLoader(Some((image, x, y, scale))) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.x_pos = Some(x);
                settings_pane.x_pos_input = format!("{}", x);
                settings_pane.y_pos = Some(y);
                settings_pane.y_pos_input = format!("{}", y);
                settings_pane.scale = Some(scale);
                settings_pane.scale_input = format!("{}", scale);

                return Command::perform(async { image }, Message::LevelImageSet);
            }
            Message::DemosTFImageLoader(None) => {}
            Message::EndOfDemoFilesDrop(_) => {
                if !self.dropped_files.is_empty() {
                    self.set_busy(true);
                    let demo_count = self.dropped_files.len();
                    self.log(&format!("Processing {} demo{}...", demo_count, if demo_count > 1 { "s" } else { "" }));
                    let input_paths = mem::take(&mut self.dropped_files);
                    return Command::perform(process_demos_async(input_paths), Message::ProcessDemosDone);
                }
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.pane_grid_state.resize(&split, ratio);
            }
            Message::DemoRemoved(index) => {
                let demo_list = self.get_demo_list_pane_mut();
                let removed = demo_list.demo_files.remove(index);
                let death_count = removed.heatmap_analysis.deaths.len();
                self.log(&format!(
                    "Removing {} with {} death{}",
                    removed.file_name,
                    death_count,
                    if death_count > 1 { "s" } else { "" }
                ));
                self.show_stats();
                self.try_generate_heatmap();
            }
            Message::ChatPreview(index) => {
                let demo_list = self.get_demo_list_pane_mut();
                let demo = &demo_list.demo_files[index];
                let messages = format_chat_messages(&demo.heatmap_analysis);
                let header_message = format!("Chat log of {}:", demo.file_name);
                self.log("");
                self.log(&header_message);
                self.log("======================");
                for message in messages {
                    self.log(&message);
                }
                self.log("======================");
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                self.get_demo_list_pane_mut().theme = theme;
                self.get_filters_pane_mut().theme = theme;
                self.get_settings_pane_mut().theme = theme;
                self.get_preview_pane_mut().theme = theme;
                self.get_log_pane_mut().theme = theme;
            }
            Message::CoordsTypeChanged(coords_type) => {
                self.get_settings_pane_mut().coords_type = coords_type;
                self.try_generate_heatmap();
            }
            Message::HeatmapTypeChanged(heatmap_type) => {
                self.get_settings_pane_mut().heatmap_type = heatmap_type;
                self.try_generate_heatmap();
            }
            Message::XPosInputChanged(input) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.x_pos = input.parse().ok();
                if let Some(x_pos) = settings_pane.x_pos {
                    if !x_pos.is_normal() && x_pos != 0.0 {
                        settings_pane.x_pos = None;
                    }
                }
                settings_pane.x_pos_input = input;
                self.try_generate_heatmap();
            }
            Message::YPosInputChanged(input) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.y_pos = input.parse().ok();
                if let Some(y_pos) = settings_pane.y_pos {
                    if !y_pos.is_normal() && y_pos != 0.0 {
                        settings_pane.y_pos = None;
                    }
                }
                settings_pane.y_pos_input = input;
                self.try_generate_heatmap();
            }
            Message::ScaleInputChanged(input) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.scale = input.parse().ok();
                if let Some(scale) = settings_pane.scale {
                    if !scale.is_normal() {
                        settings_pane.scale = None;
                    }
                }
                settings_pane.scale_input = input;
                self.try_generate_heatmap();
            }
            Message::AutoIntensityCheckboxToggled(auto_intensity) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.auto_intensity = auto_intensity;
                self.try_generate_heatmap();
            }
            Message::UseSentryPositionCheckboxToggled(use_sentry_position) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.use_sentry_position = use_sentry_position;
                self.try_generate_heatmap();
            }
            Message::IntensityChanged(intensity) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.intensity = intensity;
                self.try_generate_heatmap();
            }
            Message::RadiusChanged(radius) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.radius = radius;
                self.try_generate_heatmap();
            }
            Message::DesaturateChanged(desaturate) => {
                let settings_pane = self.get_settings_pane_mut();
                settings_pane.desaturate = desaturate;
                self.try_generate_heatmap();
            }
            Message::ProcessDemosDone(mut timed_result) => {
                let mut demo_count = 0;
                let mut death_count = 0;
                let demo_list = self.get_demo_list_pane_mut();
                let mut errors = Vec::new();
                let mut map = String::new();
                for demo in timed_result.result.iter_mut() {
                    let demo = mem::take(demo);
                    demo_count += 1;
                    map = demo.map.clone();
                    if let Some(heatmap_analysis) = demo.heatmap_analysis {
                        death_count += heatmap_analysis.deaths.len();
                        let path = demo.path;
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                        let demo_file = DemoFile {
                            _path: path,
                            file_name,
                            heatmap_analysis,
                            delete_button: Default::default(),
                            chat_preview_button: Default::default(),
                        };
                        demo_list.demo_files.push(demo_file);
                    }
                    if let Some(error) = demo.error {
                        errors.push(error);
                    }
                }
                for error in errors {
                    self.log(&error);
                }
                self.log(&format!(
                    "Loaded {} death{} from {} demo{} in {:.2}s",
                    death_count,
                    if death_count > 1 { "s" } else { "" },
                    demo_count,
                    if demo_count > 1 { "s" } else { "" },
                    timed_result.time_elapsed
                ));
                self.show_stats();
                self.try_generate_heatmap();
                self.set_busy(false);
                return Command::perform(async { map }, Message::MapSet);
            }
            Message::ExportImagePressed => {
                return Command::perform(open_save_dialog(), Message::ImageNameSelected);
            }
            Message::ImageNameSelected(path) => {
                if let Some(mut path) = path {
                    if path.extension().is_none() {
                        self.log("File extension not specified, defaulting to png");
                        path.set_extension("png");
                    }
                    match &self.get_preview_pane().heatmap_image {
                        Some(heatmap_image) => {
                            if let Err(err) = heatmap_image.image_with_heatmap_overlay.save(&path) {
                                self.log(&format!("Couldn't save the image: {}", err));
                            } else {
                                self.log(&format!("Image saved: {}", path.file_name().unwrap().to_string_lossy()));
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Message::FilterSelected(index, selected) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                filter_row.selected_filter = selected;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::AddFilter => {
                self.get_filters_pane_mut().filters.push(Default::default());
            }
            Message::ClassIconClicked(index, class_index) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                let button_active = &mut filter_row.class_buttons_selected[class_index];
                *button_active = !*button_active;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::BluTeamClicked(index) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                filter_row.team_button_selected = Team::Blu;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::RedTeamClicked(index) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                filter_row.team_button_selected = Team::Red;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::OrderedOperatorSelected(index, selected) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                filter_row.selected_ordered_operator = selected;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::PropertyOperatorSelected(index, selected) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                filter_row.selected_property_operator = selected;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::PropertySelected(index, selected) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                filter_row.selected_property = selected;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::FilterTextInputChanged(index, text_input) => {
                let filter_row = &mut self.get_filters_pane_mut().filters[index];
                filter_row.text_input = text_input;
                filter_row.filter = filter_row.try_generate_filter();
                self.try_generate_heatmap();
            }
            Message::FilterRemoved(index) => {
                self.get_filters_pane_mut().filters.remove(index);
                self.try_generate_heatmap();
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::WindowEventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        let pane_grid: pane_grid::PaneGrid<Message> = pane_grid::PaneGrid::new(&mut self.pane_grid_state, |_pane, state| state.view().into()).on_resize(10, Message::PaneResized);

        let content = Column::new().align_items(iced::Alignment::Center).spacing(20).push(pane_grid);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(4)
            .style(self.theme)
            .into()
    }
}

impl App {
    fn get_demo_list_pane(&self) -> &DemoList {
        if let PaneState::DemoList(pane) = self.pane_grid_state.get(&self.demos_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_filters_pane(&self) -> &FiltersPane {
        if let PaneState::FiltersPane(pane) = self.pane_grid_state.get(&self.filters_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_settings_pane(&self) -> &SettingsPane {
        if let PaneState::SettingsPane(pane) = self.pane_grid_state.get(&self.settings_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_preview_pane(&self) -> &Preview {
        if let PaneState::Preview(pane) = self.pane_grid_state.get(&self.preview_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn _get_log_pane(&self) -> &LogPane {
        if let PaneState::LogPane(pane) = self.pane_grid_state.get(&self.log_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_demo_list_pane_mut(&mut self) -> &mut DemoList {
        if let PaneState::DemoList(pane) = self.pane_grid_state.get_mut(&self.demos_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_filters_pane_mut(&mut self) -> &mut FiltersPane {
        if let PaneState::FiltersPane(pane) = self.pane_grid_state.get_mut(&self.filters_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_settings_pane_mut(&mut self) -> &mut SettingsPane {
        if let PaneState::SettingsPane(pane) = self.pane_grid_state.get_mut(&self.settings_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_preview_pane_mut(&mut self) -> &mut Preview {
        if let PaneState::Preview(pane) = self.pane_grid_state.get_mut(&self.preview_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_log_pane_mut(&mut self) -> &mut LogPane {
        if let PaneState::LogPane(pane) = self.pane_grid_state.get_mut(&self.log_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn log(&mut self, message: &str) {
        self.get_log_pane_mut().log(message);
    }
    fn set_busy(&mut self, busy: bool) {
        self.busy = busy;
        self.get_demo_list_pane_mut().busy = busy;
        self.get_filters_pane_mut().busy = busy;
        self.get_settings_pane_mut().busy = busy;
        // self.get_preview_pane().busy = busy;
        // self.get_log_pane().busy = busy;
    }
    fn show_stats(&mut self) {
        let demo_list = self.get_demo_list_pane();
        let death_count: usize = demo_list.demo_files.iter().map(|demo_file| demo_file.heatmap_analysis.deaths.len()).sum();
        let round_count: usize = demo_list.demo_files.iter().map(|demo_file| demo_file.heatmap_analysis.rounds.len()).sum();
        let blu_wins: usize = demo_list.demo_files.iter().map(|demo_file| demo_file.heatmap_analysis.rounds.iter().filter(|round| round.winner == Team::Blu).count()).sum();
        let red_wins: usize = demo_list.demo_files.iter().map(|demo_file| demo_file.heatmap_analysis.rounds.iter().filter(|round| round.winner == Team::Red).count()).sum();
        let demo_count = demo_list.demo_files.len();
        self.log(&format!(
            "Stats: {} death{}, {} demo{}\nRound count: {}, Blu wins: {} ({:.2}%), Red wins: {} ({:.2}%)",
            death_count,
            if death_count > 1 { "s" } else { "" },
            demo_count,
            if demo_count > 1 { "s" } else { "" },
            round_count,
            blu_wins,
            blu_wins as f32 * 100.0 / round_count as f32,
            red_wins,
            red_wins as f32 * 100.0 / round_count as f32,
        ));
    }
    fn try_generate_heatmap(&mut self) {
        let preview_pane = self.get_preview_pane();
        let settings_pane = self.get_settings_pane();
        let image = match &preview_pane.heatmap_image {
            Some(image) => apply_image_transformations(&image.image, settings_pane.desaturate),
            None => return,
        };
        if let (Some(pos_x), Some(pos_y), Some(scale)) = (settings_pane.x_pos, settings_pane.y_pos, settings_pane.scale) {
            let coords_type = settings_pane.coords_type;
            let heatmap_type = settings_pane.heatmap_type;
            let radius = settings_pane.radius;
            let intensity = if settings_pane.auto_intensity { None } else { Some(settings_pane.intensity) };
            let use_sentry_position = settings_pane.use_sentry_position;
            let screen_width = image.width();
            let screen_height = image.height();
            let filters: Vec<_> = self.get_filters_pane().filters.iter().filter_map(|filter_row| filter_row.filter.as_ref()).collect();
            let demo_list = self.get_demo_list_pane();
            let deaths = demo_list
                .demo_files
                .iter()
                .map(|demo_file| demo_file.heatmap_analysis.deaths.iter())
                .flatten()
                .filter(|death| filters.iter().all(|filter| filter.apply(death)));
            let heatmap_generation_output = coldmaps::generate_heatmap(
                heatmap_type,
                deaths,
                image,
                screen_width,
                screen_height,
                pos_x,
                pos_y,
                scale,
                coords_type,
                radius,
                intensity,
                use_sentry_position,
            );
            match &mut self.get_preview_pane_mut().heatmap_image {
                Some(heatmap_image) => {
                    heatmap_image.handle = image_to_handle(&heatmap_generation_output);
                    heatmap_image.image_with_heatmap_overlay = heatmap_generation_output;
                }
                _ => unreachable!(),
            };
        } else {
            // We can't generate the heatmap yet but we should still apply the desaturation on the level overview
            match &mut self.get_preview_pane_mut().heatmap_image {
                Some(heatmap_image) => {
                    heatmap_image.handle = image_to_handle(&image);
                    heatmap_image.image_with_heatmap_overlay = image;
                }
                _ => unreachable!(),
            };
        }
    }
}

// just desaturate for now
fn apply_image_transformations(image: &ImageBuffer<Rgb<u8>, Vec<u8>>, desaturate: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let desaturate = desaturate / 100.0;
    let mut output_image = ImageBuffer::new(image.width(), image.height());

    for (x, y, pixel) in image.enumerate_pixels() {
        let Rgb(data) = *pixel;

        // Convert the pixel to grayscale
        let gray_value = (0.3 * data[0] as f32 + 0.59 * data[1] as f32 + 0.11 * data[2] as f32) as u8;

        // Linearly interpolate between the original pixel and the grayscale value
        let new_pixel = Rgb([
            ((1.0 - desaturate) * data[0] as f32 + desaturate * gray_value as f32) as u8,
            ((1.0 - desaturate) * data[1] as f32 + desaturate * gray_value as f32) as u8,
            ((1.0 - desaturate) * data[2] as f32 + desaturate * gray_value as f32) as u8,
        ]);

        output_image.put_pixel(x, y, new_pixel);
    }

    output_image
}

fn image_to_handle(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Handle {
    Handle::from_pixels(
        image.width(),
        image.height(),
        image.pixels().fold(Vec::with_capacity((image.width() * image.height() * 4) as usize), |mut acc, pixel| {
            if let [r, g, b] = pixel.channels() {
                acc.push(*b);
                acc.push(*g);
                acc.push(*r);
                acc.push(255);
                acc
            } else {
                unreachable!()
            }
        }),
    )
}

async fn process_demos_async<'a>(inputs: Vec<PathBuf>) -> TimedResult<Vec<DemoProcessingOutput>> {
    let chrono = Instant::now();
    let result = tokio::task::spawn_blocking(move || coldmaps::process_demos(inputs)).await.unwrap();
    let time_elapsed = chrono.elapsed().as_secs_f32();
    TimedResult { result, time_elapsed }
}

async fn open_save_dialog() -> Option<PathBuf> {
    AsyncFileDialog::new().add_filter("image", &["png"]).save_file().await.map(|handle| handle.path().into())
}
