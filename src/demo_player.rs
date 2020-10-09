mod weapons;

use io::{BufRead, BufReader, BufWriter, LineWriter, StdoutLock};
use std::{num::NonZeroU32, str::FromStr};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    convert::TryFrom,
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use coldmaps::heatmap_analyser::{Class, HeatmapAnalyser, HeatmapAnalysis, PlayerState, Spawn, Team, UserId, UserInfo};
use serde::Serialize;
use tf_demo_parser::{
    demo::gamevent::GameEvent, demo::header::Header, demo::message::packetentities::EntityId, demo::message::packetentities::PacketEntity, demo::message::packetentities::PVS,
    demo::message::Message, demo::packet::datatable::ParseSendTable, demo::packet::datatable::ServerClass, demo::packet::datatable::ServerClassName,
    demo::packet::stringtable::StringTableEntry, demo::parser::handler::BorrowMessageHandler, demo::parser::DemoTicker, demo::parser::MessageHandler, demo::vector::Vector,
    demo::vector::VectorXY, Demo, DemoParser, MessageType, ParseError, ParserState, ReadResult, Stream,
};
use weapons::Weapon;

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

#[derive(Debug, Clone, Serialize, PartialEq)]
struct PlayerEntity {
    entity: EntityId,
    position: Vector,
    health: u16,
    max_health: u16,
    class: Class,
    team: Team,
    view_angle_horizontal: f32,
    view_angle_vertical: f32,
    state: PlayerState,
    active_weapon: Option<NonZeroU32>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq)]
struct ProjectileProperties {
    crit: bool,
    team: Team,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "type")]
enum EntityContent {
    Unknown,
    Other {
        class_name: String,
    },
    Pipe(ProjectileProperties),
    Sticky(ProjectileProperties),
    Rocket(ProjectileProperties),
    TeamTrainWatcher {
        total_progress: f32,
        train_speed_level: i32,
        num_cappers: i32,
        recede_time: f32,
    },
    Cart,
    Weapon { name: Weapon, id: i32, owner: Option<NonZeroU32> }
}

impl Default for EntityContent {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Default, Clone, Debug, Serialize, PartialEq)]
struct OtherEntity {
    entity_content: EntityContent,
    position: Vector,
    rotation: Vector,
}

#[derive(Default, Clone, Debug, Serialize, PartialEq)]
struct DemoAnalysis {
    current_tick: u32,
    users: BTreeMap<UserId, UserInfo>,
    player_entities: Vec<PlayerEntity>,
    other_entities: BTreeMap<EntityId, OtherEntity>,
}

impl DemoAnalysis {
    fn get_or_create_player_entity(&mut self, entity_id: EntityId) -> &mut PlayerEntity {
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
                    active_weapon: None,
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
    tick_offset: u32,
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
        if self.tick_offset == 0 && tick != 0 {
            self.tick_offset = tick - 1;
        }
        self.state.current_tick = tick - self.tick_offset; // first tick = start of the demo rather than map change
        match message {
            Message::GameEvent(message) => self.handle_event(&message.event, tick),
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    if entity.pvs == PVS::Delete {
                        let removed_entity = entity.entity_index;
                        self.state.player_entities.retain(|player_entity| player_entity.entity != removed_entity);
                        let _removed = self.state.other_entities.remove(&removed_entity);
                    } else {
                        self.handle_entity(entity);
                    }
                }
                for removed_entity in &message.removed_entities {
                    self.state.player_entities.retain(|player_entity| player_entity.entity != *removed_entity);
                    let _removed = self.state.other_entities.remove(removed_entity);
                }
            }
            _ => {}
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
            "CTFGrenadePipebombProjectile" => self.handle_demo_projectile(entity),
            "CTFProjectile_Rocket" => self.handle_rocket(entity),
            "CTeamTrainWatcher" => self.handle_team_train_watcher(entity),
            "CFuncTrackTrain" => self.handle_func_track_train(entity),
            "CTFBat"
            | "CTFBat_Fish"
            | "CTFBat_Giftwrap"
            | "CTFBat_Wood"
            | "CTFBonesaw"
            | "CTFBottle"
            | "CTFBreakableMelee"
            | "CTFBreakableSign"
            | "CTFBuffItem"
            | "CTFCannon"
            | "CTFChargedSMG"
            | "CTFCleaver"
            | "CTFClub"
            | "CTFCompoundBow"
            | "CTFCrossbow"
            | "CTFDRGPomson"
            | "CTFFireAxe"
            | "CTFFists"
            | "CTFFlameThrower"
            | "CTFFlareGun"
            | "CTFFlareGun_Revenge"
            | "CTFGrenadeLauncher"
            | "CTFJar"
            | "CTFJarGas"
            | "CTFJarMilk"
            | "CTFKatana"
            | "CTFKnife"
            | "CTFLaserPointer"
            | "CTFLunchBox"
            | "CTFLunchBox_Drink"
            | "CTFMechanicalArm"
            | "CTFMinigun"
            | "CTFParachute"
            | "CTFParachute_Primary"
            | "CTFParachute_Secondary"
            | "CTFParticleCannon"
            | "CTFPEPBrawlerBlaster"
            | "CTFPipebombLauncher"
            | "CTFPistol"
            | "CTFPistol_Scout"
            | "CTFPistol_ScoutPrimary"
            | "CTFPistol_ScoutSecondary"
            | "CTFRevolver"
            | "CTFRobotArm"
            | "CTFRocketLauncher"
            | "CTFRocketLauncher_AirStrike"
            | "CTFRocketLauncher_DirectHit"
            | "CTFRocketLauncher_Mortar"
            | "CTFRocketPack"
            | "CTFScatterGun"
            | "CTFShotgun"
            | "CTFShotgun_HWG"
            | "CTFShotgun_Pyro"
            | "CTFShotgun_Revenge"
            | "CTFShotgun_Soldier"
            | "CTFShotgunBuildingRescue"
            | "CTFShovel"
            | "CTFSlap"
            | "CTFSMG"
            | "CTFSniperRifle"
            | "CTFSniperRifleClassic"
            | "CTFSniperRifleDecap"
            | "CTFSodaPopper"
            | "CTFStickBomb"
            | "CTFSword"
            | "CTFSyringeGun"
            | "CTFWeaponBuilder"
            | "CTFWeaponPDA"
            | "CTFWeaponPDA_Engineer_Build"
            | "CTFWeaponPDA_Engineer_Destroy"
            | "CTFWeaponPDA_Spy"
            | "CTFWeaponSapper"
            | "CTFWearableDemoShield"
            | "CTFWearableRazorback"
            | "CTFWearableRobotArm"
            | "CTFWrench" => self.handle_weapon(entity),

            _ => {
                let class_name = class_name.into();
                self.handle_unknown_entity(entity, class_name);
            }
        }
    }

    fn handle_player_resource(&mut self, entity: &PacketEntity) {
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

    fn handle_player_entity(&mut self, entity: &PacketEntity) {
        let player = self.state.get_or_create_player_entity(entity.entity_index);

        for prop in &entity.props {
            match prop.definition.owner_table.as_str() {
                "DT_BasePlayer" => match prop.definition.name.as_str() {
                    "m_iHealth" => player.health = i64::try_from(&prop.value).unwrap_or_default() as u16,
                    "m_iMaxHealth" => player.max_health = i64::try_from(&prop.value).unwrap_or_default() as u16,
                    "m_lifeState" => player.state = PlayerState::new(i64::try_from(&prop.value).unwrap_or_default()),
                    // "m_fFlags" => {
                    //     match &prop.value {
                    //         tf_demo_parser::demo::sendprop::SendPropValue::Integer(x) => {
                    //             // TODO investigate, 1 = on ground, 2 = ducking, etc.
                    //             eprintln!("{}", x);
                    //         },
                    //         _ => {}
                    //     }
                    // }
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
                "DT_BaseCombatCharacter" => match prop.definition.name.as_str() {
                    "m_hActiveWeapon" => player.active_weapon = handle_to_entity_index(i64::try_from(&prop.value).unwrap_or_default()),
                    _ => {}
                }
                _ => {}
            }
        }
    }

    fn handle_unknown_entity(&mut self, entity: &PacketEntity, class_name: String) {
        let entry = self.state.other_entities.entry(entity.entity_index).or_insert_with(|| OtherEntity {
            ..Default::default()
        });
        entry.entity_content = EntityContent::Other { class_name };
        for prop in &entity.props {
            match prop.definition.name.as_str() {
                "m_vecOrigin" => entry.position = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_angRotation" => entry.rotation = Vector::try_from(&prop.value).unwrap_or_default(),
                _ => {}
            }
        }
    }

    fn handle_demo_projectile(&mut self, entity: &PacketEntity) {
        let entry = self.state.other_entities.entry(entity.entity_index).or_insert_with(|| OtherEntity {
            ..Default::default()
        });
        let (mut itype, mut projectile_properties) = match entry.entity_content {
            EntityContent::Pipe(projectile_properties) => (0, projectile_properties),
            EntityContent::Sticky(projectile_properties) => (1, projectile_properties),
            _ => (-1, Default::default()),
        };
        for prop in &entity.props {
            match prop.definition.name.as_str() {
                "m_vecOrigin" => entry.position = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_angRotation" => entry.rotation = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_iType" => itype = i64::try_from(&prop.value).unwrap_or(-1),
                "m_bCritical" => projectile_properties.crit = i64::try_from(&prop.value).unwrap_or_default() != 0,
                "m_iTeamNum" => projectile_properties.team = Team::new(i64::try_from(&prop.value).unwrap_or_default()),
                // "m_hThrower" => eprintln!("Demo {}: {}", i64::try_from(&prop.value).unwrap_or_default() & 0b111_1111_1111, entity.entity_index),
                _ => {}
            }
        }
        entry.entity_content = match itype {
            0 => EntityContent::Pipe(projectile_properties),
            1 => EntityContent::Sticky(projectile_properties),
            _ => EntityContent::Unknown, // TODO check for quickiebomb, scotres (DT_TFProjectile_Pipebomb::m_bDefensiveBomb?) etc.
        }
    }

    fn handle_rocket(&mut self, entity: &PacketEntity) {
        let entry = self.state.other_entities.entry(entity.entity_index).or_insert_with(|| OtherEntity {
            ..Default::default()
        });
        let mut projectile_properties = if let EntityContent::Rocket(projectile_properties) = entry.entity_content {
            projectile_properties
        } else {
            Default::default()
        };
        for prop in &entity.props {
            match prop.definition.name.as_str() {
                "m_vecOrigin" => entry.position = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_angRotation" => entry.rotation = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_bCritical" => projectile_properties.crit = i64::try_from(&prop.value).unwrap_or_default() != 0,
                "m_iTeamNum" => projectile_properties.team = Team::new(i64::try_from(&prop.value).unwrap_or_default()),
                // "m_hOwnerEntity" => eprintln!("Soldier {}: {}", i64::try_from(&prop.value).unwrap_or_default() & 0b111_1111_1111, entity.entity_index),
                _ => {}
            }
        }
        entry.entity_content = EntityContent::Rocket(projectile_properties);
    }

    fn handle_team_train_watcher(&mut self, entity: &PacketEntity) {
        let entry = self.state.other_entities.entry(entity.entity_index).or_insert_with(|| OtherEntity {
            ..Default::default()
        });
        let (mut total_progress, mut train_speed_level, mut num_cappers, mut recede_time) = if let EntityContent::TeamTrainWatcher {
            total_progress,
            train_speed_level,
            num_cappers,
            recede_time,
        } = entry.entity_content
        {
            (total_progress, train_speed_level, num_cappers, recede_time)
        } else {
            Default::default()
        };
        for prop in &entity.props {
            match prop.definition.name.as_str() {
                "m_vecOrigin" => entry.position = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_angRotation" => entry.rotation = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_flTotalProgress" => total_progress = f32::try_from(&prop.value).unwrap_or_default(),
                "m_iTrainSpeedLevel" => train_speed_level = i64::try_from(&prop.value).unwrap_or_default() as i32,
                "m_nNumCappers" => num_cappers = i64::try_from(&prop.value).unwrap_or_default() as i32,
                "m_flRecedeTime" => recede_time = f32::try_from(&prop.value).unwrap_or_default(),
                _ => {}
            }
        }
        entry.entity_content = EntityContent::TeamTrainWatcher {
            total_progress,
            train_speed_level,
            num_cappers,
            recede_time,
        };
    }

    fn handle_func_track_train(&mut self, entity: &PacketEntity) {
        let entry = self.state.other_entities.entry(entity.entity_index).or_insert_with(|| OtherEntity {
            ..Default::default()
        });
        for prop in &entity.props {
            match prop.definition.name.as_str() {
                "m_vecOrigin" => entry.position = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_angRotation" => entry.rotation = Vector::try_from(&prop.value).unwrap_or_default(),
                _ => {}
            }
        }
        entry.entity_content = EntityContent::Cart;
    }

    fn handle_weapon(&mut self, entity: &PacketEntity) {
        let entry = self.state.other_entities.entry(entity.entity_index).or_insert_with(|| OtherEntity {
            ..Default::default()
        });
        let (mut id, mut name, mut owner) = match entry.entity_content {
            EntityContent::Weapon { name, id, owner } => (id, name, owner),
            _ => (-1, Weapon::Unknown, None)
        };
        for prop in &entity.props {
            match prop.definition.name.as_str() {
                "m_vecOrigin" => entry.position = Vector::try_from(&prop.value).unwrap_or_default(),
                "m_angRotation" => entry.rotation = Vector::try_from(&prop.value).unwrap_or_default(),
                "moveparent" => owner = handle_to_entity_index(i64::try_from(&prop.value).unwrap_or_default()),
                "m_iItemDefinitionIndex" => id = i64::try_from(&prop.value).unwrap_or(-1) as i32, // TODO: for some reason this is not always filled :(
                _ => {}
            }
        }
        if name == Weapon::Unknown {
            name = weapons::index_to_weapon(id);
        }
        if name == Weapon::Unknown {
            // eprintln!("Unknown weapon: {}, {}, owner: {:?}, {}", id, self.class_names.get(usize::from(entity.server_class)).map(|class_name| class_name.as_str()).unwrap_or(""), owner, entity.entity_index);
            entry.entity_content = EntityContent::Other { class_name: self.class_names.get(usize::from(entity.server_class)).map(|class_name| class_name.as_str()).unwrap_or("").into() };
            return;
        }
        entry.entity_content = EntityContent::Weapon { name, id, owner };
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

fn handle_to_entity_index(handle: i64) -> Option<NonZeroU32> {
    let ret = handle as u32 & 0b111_1111_1111; // The rest of the bits is probably some kind of generational index
    if ret == 2047 {
        return None
    }
    NonZeroU32::new(ret)
}