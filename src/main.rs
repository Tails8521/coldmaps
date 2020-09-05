use coldmaps::*;
mod style;

use heatmap::CoordsType;
use heatmap_analyser::Death;
use iced::{
    button, executor, image::Handle, pane_grid, scrollable, text_input, window, Align, Application,
    Button, Column, Command, Container, Element, Font, HorizontalAlignment, Image, Length, Point,
    Radio, Rectangle, Row, Scrollable, Settings, Size, Subscription, Text, TextInput,
};
use image::{io::Reader, ImageBuffer, Pixel, Rgb};
use pane_grid::{Axis, Pane};
use std::{path::PathBuf, time::Instant};

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .color([1.0, 0.0, 0.0])
        .horizontal_alignment(HorizontalAlignment::Center)
        .size(20)
}

fn delete_icon() -> Text {
    icon('\u{F1F8}')
}

pub fn main() {
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
    _path: PathBuf,
}

#[derive(Debug)]
struct DemoFile {
    path: PathBuf,
    file_name: String,
    delete_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    WindowEventOccurred(iced_native::Event),
    PaneResized(pane_grid::ResizeEvent),
    DemoRemoved(usize),
    ThemeChanged(style::Theme),
    CoordsTypeChanged(CoordsType),
    XPosInputChanged(String),
    YPosInputChanged(String),
    ScaleInputChanged(String),
    ProcessDemosPressed,
    ProcessDemosDone(DemoProcessingOutput),
    GenerateHeatmapPressed,
    HeatmapGenerationDone(HeatmapGenerationOutput),
    ExportImagePressed,
    ImageNameSelected(Option<PathBuf>),
}

#[derive(Debug, Clone)]
struct DemoProcessingOutput {
    result: (Vec<Death>, Vec<String>),
    time_elapsed: f32,
}

#[derive(Debug, Clone)]
struct HeatmapGenerationOutput {
    result: Result<ImageBuffer<Rgb<u8>, Vec<u8>>, String>,
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
                        .horizontal_alignment(HorizontalAlignment::Center),
                )
                .width(Length::Fill)
                .into(),
                style::ResultContainer::Error,
            )
        } else {
            let theme = self.theme;
            let busy = self.busy;
            (
                self.demo_files
                    .iter_mut()
                    .enumerate()
                    .fold(Column::new().spacing(10), |column, (index, demo)| {
                        let mut delete_button =
                            Button::new(&mut demo.delete_button, delete_icon()).style(theme);
                        if !busy {
                            delete_button = delete_button.on_press(Message::DemoRemoved(index));
                        }
                        let row = Row::new()
                            .push(delete_button)
                            .push(Text::new(&demo.file_name).size(20));
                        column.push(row)
                    })
                    .into(),
                style::ResultContainer::Ok,
            )
        };
        let demos_scroll = Scrollable::new(&mut self.scroll_state)
            .push(demos_list)
            .width(Length::Fill)
            .height(Length::Fill);

        let result_container = Container::new(demos_scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10)
            .style(style);

        Container::new(result_container)
            .padding(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Default)]
struct FiltersPane {
    theme: style::Theme,
    busy: bool,
    filters: Vec<()>, // TODO
}

impl FiltersPane {
    fn view(&mut self) -> Element<Message> {
        let (filters, style): (Element<_>, _) = if self.filters.is_empty() {
            (
                Container::new(
                    Text::new("Filters will go here")
                        .width(Length::Fill)
                        .size(20)
                        .horizontal_alignment(HorizontalAlignment::Center),
                )
                .width(Length::Fill)
                .center_y()
                .into(),
                style::ResultContainer::Ok,
            )
        } else {
            todo!()
        };

        let result_container = Container::new(filters)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10)
            .style(style);

        Container::new(result_container)
            .padding(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Default)]
struct SettingsPane {
    theme: style::Theme,
    busy: bool,
    deaths: Vec<Death>,
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
    process_demos_button: button::State,
    generate_heatmap_button: button::State,
    export_image_button: button::State,
    demo_list_ready: bool,       // is there at least 1 demo in the list?
    demos_need_processing: bool, // has the current list not been processed?
    image_ready: bool,
    coords_type: CoordsType,
}

impl SettingsPane {
    fn view(&mut self) -> Element<Message> {
        let style = if self.x_pos.is_some() && self.y_pos.is_some() && self.scale.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Theme:")),
            |column, theme| {
                column.push(
                    Radio::new(
                        *theme,
                        &format!("{:?}", theme),
                        Some(self.theme),
                        Message::ThemeChanged,
                    )
                    .style(self.theme),
                )
            },
        );
        let choose_coords_type = [CoordsType::ShowPos, CoordsType::Console].iter().fold(
            Column::new()
                .spacing(10)
                .push(Text::new("Coordinates origin:")),
            |column, coords_type| {
                column.push(
                    Radio::new(
                        *coords_type,
                        &format!("{}", coords_type),
                        Some(self.coords_type),
                        Message::CoordsTypeChanged,
                    )
                    .style(self.theme),
                )
            },
        );

        let x_pos_input = TextInput::new(
            &mut self.x_pos_input_state,
            "Camera x position",
            &self.x_pos_input,
            Message::XPosInputChanged,
        )
        .style(self.theme);
        let x_pos_style = if self.x_pos.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let x_pos_border = Container::new(x_pos_input)
            .padding(3)
            .width(Length::Fill)
            .style(x_pos_style);

        let y_pos_input = TextInput::new(
            &mut self.y_pos_input_state,
            "Camera y position",
            &self.y_pos_input,
            Message::YPosInputChanged,
        )
        .style(self.theme);
        let y_pos_style = if self.y_pos.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let y_pos_border = Container::new(y_pos_input)
            .padding(3)
            .width(Length::Fill)
            .style(y_pos_style);

        let scale_input = TextInput::new(
            &mut self.scale_input_state,
            "Camera scale",
            &self.scale_input,
            Message::ScaleInputChanged,
        )
        .style(self.theme);
        let scale_style = if self.scale.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let scale_border = Container::new(scale_input)
            .padding(3)
            .width(Length::Fill)
            .style(scale_style);

        let mut process_demos_button =
            Button::new(&mut self.process_demos_button, Text::new("Process demos"))
                .padding(10)
                .style(self.theme)
                .width(Length::Fill);
        if self.demo_list_ready && self.demos_need_processing && !self.busy {
            process_demos_button = process_demos_button.on_press(Message::ProcessDemosPressed);
        }
        let mut generate_heatmap_button = Button::new(
            &mut self.generate_heatmap_button,
            Text::new("Generate heatmap"),
        )
        .padding(10)
        .style(self.theme)
        .width(Length::Fill);
        if self.x_pos.is_some()
            && self.y_pos.is_some()
            && self.scale.is_some()
            && self.image_ready
            && !self.deaths.is_empty()
            && !self.busy
        {
            generate_heatmap_button =
                generate_heatmap_button.on_press(Message::GenerateHeatmapPressed);
        }
        let mut export_image_button =
            Button::new(&mut self.export_image_button, Text::new("Export image"))
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
        let settings_content: Element<_> = Column::new()
            .push(Text::new(coords_label))
            .push(x_pos_border)
            .push(y_pos_border)
            .push(Text::new("cl_leveloverview scale"))
            .push(scale_border)
            .push(process_demos_button)
            .push(generate_heatmap_button)
            .push(export_image_button)
            .push(choose_coords_type)
            .push(choose_theme)
            .spacing(10)
            .into();

        let scroll = Scrollable::new(&mut self.scroll_state).push(settings_content);

        let result_container = Container::new(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .style(style);

        Container::new(result_container)
            .padding(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
            (
                Image::new(heatmap_image.handle.clone()).into(),
                style::ResultContainer::Ok,
            )
        } else {
            (
                Text::new("Drag and drop the level overview screenshot to use it")
                    .width(Length::Fill)
                    .size(24)
                    .horizontal_alignment(HorizontalAlignment::Center)
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

        Container::new(result_container)
            .padding(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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

        let demos_scroll = Scrollable::new(&mut self.scroll_state)
            .push(log)
            .width(Length::Fill);

        let result_container = Container::new(demos_scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .style(style::ResultContainer::Ok);

        Container::new(result_container)
            .padding(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn log(&mut self, message: &str) {
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
        let (mut pane_grid_state, demos_pane) =
            pane_grid::State::new(PaneState::DemoList(Default::default()));
        let (preview_pane, demos_preview_split) = pane_grid_state
            .split(
                Axis::Vertical,
                &demos_pane,
                PaneState::Preview(Default::default()),
            )
            .unwrap();
        let (filters_pane, demos_filter_split) = pane_grid_state
            .split(
                Axis::Horizontal,
                &demos_pane,
                PaneState::FiltersPane(Default::default()),
            )
            .unwrap();
        let (settings_pane, filters_settings_split) = pane_grid_state
            .split(
                Axis::Horizontal,
                &filters_pane,
                PaneState::SettingsPane(Default::default()),
            )
            .unwrap();
        let (log_pane, preview_log_split) = pane_grid_state
            .split(
                Axis::Horizontal,
                &preview_pane,
                PaneState::LogPane(Default::default()),
            )
            .unwrap();
        pane_grid_state.resize(&demos_preview_split, 0.15);
        pane_grid_state.resize(&demos_filter_split, 0.3);
        pane_grid_state.resize(&filters_settings_split, 0.1);
        pane_grid_state.resize(&preview_log_split, 0.8);
        (
            App {
                busy: false,
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
            Message::WindowEventOccurred(iced_native::Event::Window(
                iced_native::window::Event::FileDropped(path),
            )) if !self.busy => {
                if !path.is_file() {
                    return Command::none();
                }
                let file_name = path.file_name().unwrap().to_string_lossy().to_string(); // The path can't be .. at that point
                let file_name_lowercase = file_name.to_lowercase();
                if file_name_lowercase.ends_with(".dem") {
                    self.get_demo_list_pane().demo_files.push(DemoFile {
                        path,
                        file_name,
                        delete_button: Default::default(),
                    });
                    let setting_pane = self.get_settings_pane();
                    setting_pane.demo_list_ready = true;
                    setting_pane.demos_need_processing = true;
                } else {
                    // try to load it as an image
                    if let Ok(reader) = Reader::open(&path) {
                        if let Ok(image) = reader.decode() {
                            let image = image.into_rgb();
                            let image_with_heatmap_overlay = image.clone();
                            let handle = image_to_handle(&image);
                            self.get_preview_pane().heatmap_image.replace(HeatmapImage {
                                image,
                                image_with_heatmap_overlay,
                                handle,
                                _path: path,
                            });
                            self.get_settings_pane().image_ready = true;
                        }
                    }
                }
            }
            Message::WindowEventOccurred(_) => {}
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.pane_grid_state.resize(&split, ratio);
            }
            Message::DemoRemoved(index) => {
                let demo_list = self.get_demo_list_pane();
                demo_list.demo_files.remove(index);
                if demo_list.demo_files.is_empty() {
                    self.get_settings_pane().demo_list_ready = false;
                }
                self.get_settings_pane().demos_need_processing = true;
            }
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                self.get_demo_list_pane().theme = theme;
                self.get_filters_pane().theme = theme;
                self.get_settings_pane().theme = theme;
                self.get_preview_pane().theme = theme;
                self.get_log_pane().theme = theme;
            }
            Message::CoordsTypeChanged(coords_type) => {
                self.get_settings_pane().coords_type = coords_type;
            }
            Message::XPosInputChanged(input) => {
                let settings_pane = self.get_settings_pane();
                settings_pane.x_pos = input.parse().ok();
                settings_pane.x_pos_input = input;
            }
            Message::YPosInputChanged(input) => {
                let settings_pane = self.get_settings_pane();
                settings_pane.y_pos = input.parse().ok();
                settings_pane.y_pos_input = input;
            }
            Message::ScaleInputChanged(input) => {
                let settings_pane = self.get_settings_pane();
                settings_pane.scale = input.parse().ok();
                settings_pane.scale_input = input;
            }
            Message::ProcessDemosPressed => {
                self.get_settings_pane().demos_need_processing = false;
                self.set_busy(true);
                let demo_count = self.get_demo_list_pane().demo_files.len();
                self.log(&format!(
                    "Processing {} demo{}...",
                    demo_count,
                    if demo_count > 1 { "s" } else { "" }
                ));
                return Command::perform(
                    process_demos_async(
                        self.get_demo_list_pane()
                            .demo_files
                            .iter()
                            .map(|demo| demo.path.clone())
                            .collect::<Vec<_>>(),
                    ),
                    Message::ProcessDemosDone,
                );
            }
            Message::ProcessDemosDone(demo_processing_output) => {
                let demo_list = self.get_demo_list_pane();
                let demo_count = demo_list.demo_files.len();
                let (deaths, errors) = demo_processing_output.result;
                for error in errors {
                    self.log(&error);
                }
                let death_count = deaths.len();
                self.log(&format!(
                    "Loaded {} death{} from {} demo{} in {:.2}s",
                    death_count,
                    if death_count > 1 { "s" } else { "" },
                    demo_count,
                    if demo_count > 1 { "s" } else { "" },
                    demo_processing_output.time_elapsed
                ));
                self.get_settings_pane().deaths = deaths;
                self.set_busy(false);
            }
            Message::GenerateHeatmapPressed => {
                self.set_busy(true);
                let settings_pane = self.get_settings_pane();
                let deaths = settings_pane.deaths.clone();
                let pos_x = settings_pane.x_pos.unwrap();
                let pos_y = settings_pane.y_pos.unwrap();
                let scale = settings_pane.scale.unwrap();
                let coords_type = settings_pane.coords_type;
                let preview_pane = self.get_preview_pane();
                let image = match &preview_pane.heatmap_image {
                    Some(image) => image.image.clone(),
                    _ => unreachable!(),
                };
                let screen_width = image.width();
                let screen_height = image.height();
                return Command::perform(
                    generate_heatmap_async(
                        deaths,
                        image,
                        screen_width,
                        screen_height,
                        pos_x,
                        pos_y,
                        scale,
                        coords_type,
                    ),
                    Message::HeatmapGenerationDone,
                );
            }
            Message::HeatmapGenerationDone(heatmap_generation_output) => {
                match heatmap_generation_output.result {
                    Ok(image) => {
                        self.log(&format!(
                            "Heatmap generated in {:.2}s",
                            heatmap_generation_output.time_elapsed
                        ));
                        match &mut self.get_preview_pane().heatmap_image {
                            Some(heatmap_image) => {
                                heatmap_image.handle = image_to_handle(&image);
                                heatmap_image.image_with_heatmap_overlay = image;
                            }
                            _ => unreachable!(),
                        };
                    }
                    Err(err) => {
                        self.log(&err);
                    }
                }
                self.set_busy(false);
            }
            Message::ExportImagePressed => {
                return Command::perform(open_save_dialog(), Message::ImageNameSelected);
            }
            Message::ImageNameSelected(path) => {
                if let Some(path) = path {
                    match &self.get_preview_pane().heatmap_image {
                        Some(heatmap_image) => {
                            if let Err(err) = heatmap_image.image_with_heatmap_overlay.save(&path) {
                                self.log(&format!("Couldn't save the image: {}", err));
                            } else {
                                self.log(&format!(
                                    "Image saved: {}",
                                    path.file_name().unwrap().to_string_lossy()
                                ));
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::WindowEventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        let pane_grid: pane_grid::PaneGrid<Message> =
            pane_grid::PaneGrid::new(&mut self.pane_grid_state, |_pane, state, _focus| {
                state.view().into()
            })
            .on_resize(10, Message::PaneResized);

        let content = Column::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(pane_grid);

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
    fn get_demo_list_pane(&mut self) -> &mut DemoList {
        if let PaneState::DemoList(pane) = self.pane_grid_state.get_mut(&self.demos_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_filters_pane(&mut self) -> &mut FiltersPane {
        if let PaneState::FiltersPane(pane) =
            self.pane_grid_state.get_mut(&self.filters_pane).unwrap()
        {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_settings_pane(&mut self) -> &mut SettingsPane {
        if let PaneState::SettingsPane(pane) =
            self.pane_grid_state.get_mut(&self.settings_pane).unwrap()
        {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_preview_pane(&mut self) -> &mut Preview {
        if let PaneState::Preview(pane) = self.pane_grid_state.get_mut(&self.preview_pane).unwrap()
        {
            pane
        } else {
            unreachable!()
        }
    }
    fn get_log_pane(&mut self) -> &mut LogPane {
        if let PaneState::LogPane(pane) = self.pane_grid_state.get_mut(&self.log_pane).unwrap() {
            pane
        } else {
            unreachable!()
        }
    }
    fn log(&mut self, message: &str) {
        self.get_log_pane().log(message);
    }
    fn set_busy(&mut self, busy: bool) {
        self.busy = busy;
        self.get_demo_list_pane().busy = busy;
        self.get_filters_pane().busy = busy;
        self.get_settings_pane().busy = busy;
        // self.get_preview_pane().busy = busy;
        // self.get_log_pane().busy = busy;
    }
}

fn image_to_handle(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Handle {
    Handle::from_pixels(
        image.width(),
        image.height(),
        image.pixels().fold(
            Vec::with_capacity((image.width() * image.height() * 4) as usize),
            |mut acc, pixel| {
                if let [r, g, b] = pixel.channels() {
                    acc.push(*b);
                    acc.push(*g);
                    acc.push(*r);
                    acc.push(255);
                    acc
                } else {
                    unreachable!()
                }
            },
        ),
    )
}

async fn process_demos_async<'a>(input_paths: Vec<PathBuf>) -> DemoProcessingOutput {
    let chrono = Instant::now();
    let output = tokio::task::spawn_blocking(move || coldmaps::process_demos(input_paths)).await;
    let time_elapsed = chrono.elapsed().as_secs_f32();
    match output {
        Ok(result) => DemoProcessingOutput {
            result,
            time_elapsed,
        },
        Err(err) => DemoProcessingOutput {
            result: (Vec::new(), vec![err.to_string()]),
            time_elapsed,
        },
    }
}

async fn generate_heatmap_async(
    deaths: Vec<Death>,
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    screen_width: u32,
    screen_height: u32,
    pos_x: f32,
    pos_y: f32,
    scale: f32,
    coords_type: CoordsType,
) -> HeatmapGenerationOutput {
    let chrono = Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        coldmaps::generate_heatmap(
            deaths,
            image,
            screen_width,
            screen_height,
            pos_x,
            pos_y,
            scale,
            coords_type,
        )
    })
    .await;
    let time_elapsed = chrono.elapsed().as_secs_f32();
    HeatmapGenerationOutput {
        result: result.map_err(|err| return err.to_string()),
        time_elapsed,
    }
}

async fn open_save_dialog() -> Option<PathBuf> {
    if let Ok(Ok(nfd2::Response::Okay(path))) =
        tokio::task::spawn_blocking(move || nfd2::open_save_dialog(None, None)).await
    {
        return Some(path);
    }
    None
}
