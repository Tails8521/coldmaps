use io::{BufRead, BufReader, BufWriter, LineWriter, StdoutLock};
use std::str::FromStr;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    convert::TryFrom,
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use coldmaps::heatmap_analyser::{Class, HeatmapAnalyser, HeatmapAnalysis, PlayerEntity, PlayerState, Spawn, Team, UserId, UserInfo};
use serde::Serialize;
use tf_demo_parser::{
    demo::gamevent::GameEvent, demo::header::Header, demo::message::packetentities::EntityId, demo::message::packetentities::PacketEntity, demo::message::Message,
    demo::packet::datatable::ParseSendTable, demo::packet::datatable::ServerClass, demo::packet::datatable::ServerClassName, demo::packet::stringtable::StringTableEntry,
    demo::parser::handler::BorrowMessageHandler, demo::parser::DemoTicker, demo::parser::MessageHandler, demo::vector::Vector, demo::vector::VectorXY, Demo, DemoParser,
    MessageType, ParseError, ParserState, ReadResult, Stream,
};

const SECTION_SIZE: usize = 1024;

#[derive(Debug, Serialize)]
enum Command {
    Load(PathBuf),
    Tick(usize),
    Frame(usize),
    TickToFrame,
    FrameToTick,
    Analysis,
    Prefetch(bool),
}

impl Command {
    fn from_str(str: &str) -> Option<Self> {
        if str == "analysis" {
            return Some(Self::Analysis);
        }
        if str == "frametotick" {
            return Some(Self::FrameToTick);
        }
        if str == "ticktoframe" {
            return Some(Self::TickToFrame);
        }
        if let [command, arg] = str.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
            if *command == "frame" {
                if let Ok(frame) = arg.parse() {
                    return Some(Self::Frame(frame));
                }
            }
            if *command == "tick" {
                if let Ok(tick) = arg.parse() {
                    return Some(Self::Tick(tick));
                }
            }
            if *command == "load" {
                return Some(Self::Load(PathBuf::from(arg)));
            }
            if *command == "prefetch" {
                if let Ok(prefetch) = arg.parse() {
                    return Some(Self::Prefetch(prefetch));
                }
            }
        }
        None
    }
}

#[derive(Debug, Serialize)]
struct LoadOutput<'a> {
    server: &'a str,
    nick: &'a str,
    map: &'a str,
    duration: f32,
    ticks: u32,
    frames: u32,
    is_corrupted: bool,
}

#[derive(Debug, Serialize)]
struct TickOutput<'a> {
    state: &'a HeatmapAnalysis,
    ticks_left: bool,
}

#[derive(Debug, Serialize)]
struct Output<T> {
    result: Option<T>,
    error: Option<Cow<'static, str>>,
}

struct OutputWriter<'a>(LineWriter<BufWriter<StdoutLock<'a>>>);

impl OutputWriter<'_> {
    fn write_result<T: serde::Serialize>(&mut self, result: T) -> Result<(), Box<dyn Error>> {
        let output = Output {
            result: Some(result),
            error: None,
        };
        serde_json::ser::to_writer(&mut self.0, &output)?;
        self.0.write_all(&[b'\n'])?;
        Ok(())
    }

    fn write_error(&mut self, error: Cow<'static, str>) -> Result<(), Box<dyn Error>> {
        let output: Output<()> = Output { result: None, error: Some(error) };
        serde_json::ser::to_writer(&mut self.0, &output)?;
        self.0.write_all(&[b'\n'])?;
        Ok(())
    }

    fn write_text(&mut self, input: &str) -> Result<(), Box<dyn Error>> {
        self.0.write_all(input.as_bytes())?;
        self.0.write_all(&[b'\n'])?;
        Ok(())
    }
}

fn serialize<T: serde::Serialize>(input: T) -> String {
    let output = Output { result: Some(input), error: None };
    serde_json::to_string(&output).unwrap()
}

struct BufferSection {
    ticker: DemoTicker<DemoAnalyzer>,
    playback_ticker: Option<DemoTicker<DemoAnalyzer>>,
    cached_frames: Vec<String>,
    first_frame: String,
}

struct BufferedPlayer {
    sections: Vec<BufferSection>,
    playhead_position: usize,
    last_frame: usize,
}

impl BufferedPlayer {
    fn get_frame(&mut self, playhead_position: usize) -> &str {
        self.playhead_position = playhead_position;
        let section_idx = playhead_position / SECTION_SIZE;
        let frame_idx = playhead_position % SECTION_SIZE;
        let section = &mut self.sections[section_idx];
        if frame_idx == 0 {
            return &section.first_frame;
        }
        if section.playback_ticker.is_none() {
            section.playback_ticker = Some(section.ticker.clone());
            section.cached_frames.reserve_exact(SECTION_SIZE - 1);
        }
        let playback_ticker = section.playback_ticker.as_mut().unwrap();
        while section.cached_frames.len() < frame_idx {
            playback_ticker.tick().unwrap_or_default();
            section.cached_frames.push(serialize(playback_ticker.state()));
        }
        &section.cached_frames[frame_idx - 1]
    }

    fn prefetch(&mut self) {
        // For continuous playback to be smooth, we need to buffer at least 1 frame forward and SECTION_SIZE frames backwards for backwards playback.
        // This is because by the time we reach frame 0 in the current section, we need the previous section to have its last frame cached
        // so we can play it immediately afterwards.
        let section_idx = self.playhead_position / SECTION_SIZE;
        let frame_idx = self.playhead_position % SECTION_SIZE;
        let section = &mut self.sections[section_idx];
        // expand forward
        if self.playhead_position != self.last_frame
        // if the next frame is in the next section, do nothing because the 1st frame of each section is always there
        && frame_idx + 1 < SECTION_SIZE
        // if the next frame is already cached we do nothing
        && section.cached_frames.len() < frame_idx + 1
        {
            if section.playback_ticker.is_none() {
                section.playback_ticker = Some(section.ticker.clone());
                section.cached_frames.reserve_exact(SECTION_SIZE - 1);
            }
            let playback_ticker = section.playback_ticker.as_mut().unwrap();
            playback_ticker.tick().unwrap_or_default();
            section.cached_frames.push(serialize(playback_ticker.state()));
        }
        // expand backward
        if section_idx > 0 {
            let previous_section = &mut self.sections[section_idx - 1];
            while previous_section.cached_frames.len() < SECTION_SIZE - frame_idx - 1 {
                if previous_section.playback_ticker.is_none() {
                    previous_section.playback_ticker = Some(previous_section.ticker.clone());
                    previous_section.cached_frames.reserve_exact(SECTION_SIZE - 1);
                }
                let previous_playback_ticker = previous_section.playback_ticker.as_mut().unwrap();
                previous_playback_ticker.tick().unwrap_or_default();
                previous_section.cached_frames.push(serialize(previous_playback_ticker.state()));
            }
        }
    }

    fn evict(&mut self) {
        let section_idx = self.playhead_position / SECTION_SIZE;
        // discard cached sections that are far away
        self.sections
            .iter_mut()
            .enumerate()
            .filter(|(idx, section)| ((*idx as isize) < section_idx as isize - 2 || (*idx as isize) > section_idx as isize + 2) && !section.cached_frames.is_empty())
            .for_each(|(_idx, far_section)| {
                far_section.cached_frames.clear();
                far_section.cached_frames.shrink_to_fit();
                far_section.playback_ticker = None;
            });
    }
}

struct DemoPlayerState {
    is_corrupted: bool,
    frame_to_tick: Vec<u32>,
    tick_to_frame: Vec<usize>,
    final_state: String,
    demo_header: Header,
    player: BufferedPlayer,
}

impl DemoPlayerState {
    fn new(demo: Demo) -> Result<Self, ParseError> {
        let (demo_header, mut ticker) = DemoParser::new_with_analyser(demo.get_stream(), DemoAnalyzer::default()).ticker()?;
        let mut frame_to_tick = Vec::with_capacity(demo_header.frames as usize + 6);
        let mut tick_to_frame = Vec::new();
        let mut player = BufferedPlayer {
            sections: Vec::with_capacity((demo_header.frames as usize + 6) / SECTION_SIZE + 1),
            playhead_position: 0,
            last_frame: 0,
        };
        let is_corrupted = loop {
            match ticker.tick() {
                Ok(true) => {
                    let current_tick = ticker.state().current_tick;
                    if current_tick == 0 {
                        // This seems to happen for 6 frames at the start of the demo
                        // If we don't do this, demo_header.frames != frames.len()
                        continue;
                    }
                    let current_frame_index = frame_to_tick.len();
                    frame_to_tick.push(current_tick);
                    while tick_to_frame.len() <= current_tick as usize {
                        tick_to_frame.push(current_frame_index);
                    }
                    if current_frame_index % SECTION_SIZE == 0 {
                        player.sections.push(BufferSection {
                            ticker: ticker.clone(),
                            playback_ticker: None,
                            cached_frames: Vec::new(),
                            first_frame: serialize(ticker.state()),
                        });
                    }
                }
                Ok(false) => {
                    break false;
                }
                Err(_err) => {
                    break true;
                }
            };
        };
        if demo_header.frames != 0 && demo_header.frames as usize != frame_to_tick.len() {
            eprintln!("Expected {} frames in the demo, got {}", demo_header.frames, frame_to_tick.len());
        }
        player.last_frame = frame_to_tick.len() - 1;

        // Our final state should be from a HeatmapAnalyser as it contains more useful data (deaths, chat etc.)
        let (_demo_header, mut heatmap_ticker) = DemoParser::new_with_analyser(demo.get_stream(), HeatmapAnalyser::default()).ticker()?;
        loop {
            match heatmap_ticker.tick() {
                Ok(true) => (),
                Ok(false) => break,
                Err(_err) => break,
            }
        }
        let final_state = serialize(heatmap_ticker.state());
        Ok(Self {
            is_corrupted,
            frame_to_tick,
            tick_to_frame,
            final_state,
            demo_header,
            player,
        })
    }
}

pub(crate) fn run() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let stdin_handle = stdin.lock();
    let input = BufReader::new(stdin_handle);
    let mut lines = input.lines();
    let stdout = io::stdout();
    let stdout_handle = stdout.lock();
    let output = BufWriter::new(stdout_handle);
    let mut output_writer = OutputWriter(LineWriter::new(output));

    let mut demo_player_state = None;
    let mut prefetch = true;

    while let Some(line) = lines.next() {
        let line = line?;
        if let Some(command) = Command::from_str(&line) {
            match command {
                Command::Load(path) => match fs::read(path) {
                    Ok(file) => {
                        let demo = Demo::new(file);
                        match DemoPlayerState::new(demo) {
                            Ok(new_state) => {
                                let load_output = LoadOutput {
                                    server: &new_state.demo_header.server,
                                    nick: &new_state.demo_header.nick,
                                    map: &new_state.demo_header.map,
                                    duration: new_state.demo_header.duration,
                                    ticks: new_state.demo_header.ticks,
                                    frames: new_state.demo_header.frames,
                                    is_corrupted: new_state.is_corrupted,
                                };
                                output_writer.write_result(&load_output)?;
                                demo_player_state = Some(new_state);
                            }
                            Err(err) => {
                                output_writer.write_error(err.to_string().into())?;
                            }
                        }
                    }
                    Err(err) => {
                        output_writer.write_error(err.to_string().into())?;
                    }
                },
                Command::Frame(frame) => {
                    if let Some(DemoPlayerState { player, frame_to_tick, .. }) = demo_player_state.as_mut() {
                        if frame < frame_to_tick.len() {
                            let state = player.get_frame(frame);
                            output_writer.write_text(state)?;
                            if prefetch {
                                player.prefetch();
                            }
                            player.evict();
                        } else {
                            output_writer.write_error("Seeking to a frame out of bound".into())?;
                        }
                    } else {
                        output_writer.write_error("No demo loaded".into())?;
                    }
                }
                Command::Tick(tick) => {
                    if let Some(DemoPlayerState { player, tick_to_frame, .. }) = demo_player_state.as_mut() {
                        if let Some(&frame) = tick_to_frame.get(tick) {
                            let state = player.get_frame(frame);
                            output_writer.write_text(state)?;
                            if prefetch {
                                player.prefetch();
                            }
                            player.evict();
                        } else {
                            output_writer.write_error("Seeking to a tick out of bound".into())?;
                        }
                    } else {
                        output_writer.write_error("No demo loaded".into())?;
                    }
                }
                Command::FrameToTick => {
                    if let Some(DemoPlayerState { frame_to_tick, .. }) = demo_player_state.as_ref() {
                        output_writer.write_result(frame_to_tick)?;
                    } else {
                        output_writer.write_error("No demo loaded".into())?;
                    }
                }
                Command::TickToFrame => {
                    if let Some(DemoPlayerState { tick_to_frame, .. }) = demo_player_state.as_ref() {
                        output_writer.write_result(tick_to_frame)?;
                    } else {
                        output_writer.write_error("No demo loaded".into())?;
                    }
                }
                Command::Analysis => {
                    if let Some(DemoPlayerState { final_state, .. }) = demo_player_state.as_ref() {
                        output_writer.write_text(final_state)?;
                    } else {
                        output_writer.write_error("No demo loaded".into())?;
                    }
                }
                Command::Prefetch(new_prefetch) => {
                    prefetch = new_prefetch;
                    if prefetch {
                        output_writer.write_result("Prefetching enabled")?;
                    } else {
                        output_writer.write_result("Prefetching disabled")?;
                    }
                }
            }
        } else {
            output_writer.write_error(format!("Can't parse command: \"{}\"", &line).into())?;
        }
    }
    Ok(())
}

#[derive(Default, Clone, Debug, Serialize, PartialEq)]
struct DemoAnalysis {
    current_tick: u32,
    users: BTreeMap<UserId, UserInfo>,
    player_entities: Vec<PlayerEntity>,
}

impl DemoAnalysis {
    pub fn get_or_create_player_entity(&mut self, entity_id: EntityId) -> &mut PlayerEntity {
        let index = match self
            .player_entities
            .iter_mut()
            .enumerate()
            .find(|(_index, player)| player.entity == entity_id)
            .map(|(index, _)| index)
        {
            Some(index) => index,
            None => {
                let player = PlayerEntity {
                    entity: entity_id,
                    position: Vector::default(),
                    health: 0,
                    max_health: 0,
                    class: Class::Other,
                    team: Team::Other,
                    view_angle_horizontal: 0.0,
                    view_angle_vertical: 0.0,
                    state: PlayerState::Alive,
                };

                let index = self.player_entities.len();
                self.player_entities.push(player);
                index
            }
        };
        &mut self.player_entities[index]
    }
}

#[derive(Default, Clone, Debug, Serialize, PartialEq)]
struct DemoAnalyzer {
    state: DemoAnalysis,
    class_names: Vec<ServerClassName>,
}

impl MessageHandler for DemoAnalyzer {
    type Output = DemoAnalysis;

    fn does_handle(message_type: MessageType) -> bool {
        match message_type {
            MessageType::GameEvent | MessageType::PacketEntities => true,
            _ => false,
        }
    }

    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.state
    }

    fn handle_message(&mut self, message: &Message, tick: u32) {
        self.state.current_tick = tick;
        match message {
            Message::GameEvent(message) => self.handle_event(&message.event, tick),
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    self.handle_entity(entity);
                }
            }
            _ => unreachable!(),
        }
    }

    fn handle_string_entry(&mut self, table: &str, _index: usize, entry: &StringTableEntry) {
        match table {
            "userinfo" => {
                let _ = self.parse_user_info(entry.text.as_ref().map(|s| s.as_str()), entry.extra_data.as_ref().map(|data| data.data.clone()));
            }
            _ => {}
        }
    }

    fn handle_data_tables(&mut self, _tables: &[ParseSendTable], server_classes: &[ServerClass]) {
        self.class_names = server_classes.iter().map(|class| &class.name).cloned().collect();
    }
}

impl BorrowMessageHandler for DemoAnalyzer {
    fn borrow_output(&self, _state: &ParserState) -> &Self::Output {
        &self.state
    }
}

impl DemoAnalyzer {
    fn handle_entity(&mut self, entity: &PacketEntity) {
        let class_name: &str = self.class_names.get(usize::from(entity.server_class)).map(|class_name| class_name.as_str()).unwrap_or("");
        match class_name {
            "CTFPlayer" => self.handle_player_entity(entity),
            "CTFPlayerResource" => self.handle_player_resource(entity),
            _ => {}
        }
    }

    pub fn handle_player_resource(&mut self, entity: &PacketEntity) {
        for prop in &entity.props {
            if let Ok(player_id) = u32::from_str(prop.definition.name.as_str()) {
                let entity_id = EntityId::from(player_id);
                if let Some(player) = self.state.player_entities.iter_mut().find(|player| player.entity == entity_id) {
                    match prop.definition.owner_table.as_str() {
                        "m_iTeam" => player.team = Team::new(i64::try_from(&prop.value).unwrap_or_default()),
                        "m_iMaxHealth" => player.max_health = i64::try_from(&prop.value).unwrap_or_default() as u16,
                        "m_iPlayerClass" => player.class = Class::new(i64::try_from(&prop.value).unwrap_or_default()),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn handle_player_entity(&mut self, entity: &PacketEntity) {
        let player = self.state.get_or_create_player_entity(entity.entity_index);

        for prop in &entity.props {
            match prop.definition.owner_table.as_str() {
                "DT_BasePlayer" => match prop.definition.name.as_str() {
                    "m_iHealth" => player.health = i64::try_from(&prop.value).unwrap_or_default() as u16,
                    "m_iMaxHealth" => player.max_health = i64::try_from(&prop.value).unwrap_or_default() as u16,
                    "m_lifeState" => player.state = PlayerState::new(i64::try_from(&prop.value).unwrap_or_default()),
                    _ => {}
                },
                "DT_TFLocalPlayerExclusive" | "DT_TFNonLocalPlayerExclusive" => match prop.definition.name.as_str() {
                    "m_vecOrigin" => {
                        let pos_xy = VectorXY::try_from(&prop.value).unwrap_or_default();
                        player.position.x = pos_xy.x;
                        player.position.y = pos_xy.y;
                    }
                    "m_vecOrigin[2]" => player.position.z = f32::try_from(&prop.value).unwrap_or_default(),
                    "m_angEyeAngles[0]" => player.view_angle_vertical = f32::try_from(&prop.value).unwrap_or_default(),
                    "m_angEyeAngles[1]" => player.view_angle_horizontal = f32::try_from(&prop.value).unwrap_or_default(),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn handle_event(&mut self, event: &GameEvent, tick: u32) {
        match event {
            GameEvent::PlayerSpawn(event) => {
                let spawn = Spawn::from_event(event, tick);
                if let Some(user_state) = self.state.users.get_mut(&spawn.user) {
                    user_state.team = spawn.team;
                }
            }
            _ => {}
        }
    }

    fn parse_user_info(&mut self, text: Option<&str>, data: Option<Stream>) -> ReadResult<()> {
        if let Some(mut data) = data {
            let name: String = data.read_sized(32).unwrap_or_else(|_| "Malformed Name".into());
            let user_id: UserId = data.read::<u32>()?.into();
            let steam_id: String = data.read()?;

            let entity_id = if let Some(slot_id) = text {
                Some((slot_id.parse::<u32>().expect("can't parse player slot") + 1).into())
            } else {
                None
            };

            if !steam_id.is_empty() {
                self.state
                    .users
                    .entry(user_id)
                    .and_modify(|info| {
                        if entity_id != None {
                            info.entity_id = entity_id;
                        }
                    })
                    .or_insert_with(|| UserInfo {
                        team: Team::Other,
                        steam_id,
                        user_id,
                        name,
                        entity_id: entity_id,
                    });
            }
        }
        Ok(())
    }
}
