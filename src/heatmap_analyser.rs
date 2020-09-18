use num_enum::TryFromPrimitive;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use tf_demo_parser::demo::gameevent_gen::{GameEvent, PlayerDeathEvent, PlayerSpawnEvent, TeamPlayRoundWinEvent};
use tf_demo_parser::demo::message::packetentities::{EntityId, PacketEntity};
use tf_demo_parser::demo::message::usermessage::{ChatMessageKind, SayText2Message, UserMessage};
use tf_demo_parser::demo::message::{Message, MessageType};
use tf_demo_parser::demo::packet::{
    datatable::{ParseSendTable, ServerClass, ServerClassName},
    stringtable::StringTableEntry,
};
use tf_demo_parser::demo::{
    parser::MessageHandler,
    sendprop::{SendProp, SendPropValue},
    vector::{Vector, VectorXY},
};
use tf_demo_parser::{ParserState, ReadResult, Stream};

const MAX_PLAYER_ENTITY: u32 = 34;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMassage {
    pub kind: ChatMessageKind,
    pub from: String,
    pub text: String,
    pub tick: u32,
}

impl ChatMassage {
    pub fn from_message(message: &SayText2Message, tick: u32) -> Self {
        ChatMassage {
            kind: message.kind,
            from: message.from.clone().unwrap_or_default(),
            text: message.text.clone(),
            tick,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Team {
    Other = 0,
    Spectator = 1,
    Red = 2,
    Blue = 3,
}

impl Team {
    pub fn new<U>(number: U) -> Self
    where
        u8: TryFrom<U>,
    {
        Team::try_from(u8::try_from(number).unwrap_or_default()).unwrap_or_default()
    }
}

impl Default for Team {
    fn default() -> Self {
        Team::Other
    }
}

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum Class {
    Other = 0,
    Scout = 1,
    Sniper = 2,
    Soldier = 3,
    Demoman = 4,
    Medic = 5,
    Heavy = 6,
    Pyro = 7,
    Spy = 8,
    Engineer = 9,
}

impl Class {
    pub fn new<U>(number: U) -> Self
    where
        u8: TryFrom<U>,
    {
        Class::try_from(u8::try_from(number).unwrap_or_default()).unwrap_or_default()
    }
}

impl Default for Class {
    fn default() -> Self {
        Class::Other
    }
}

#[derive(Default, Debug, Eq, PartialEq, Deserialize, Clone)]
#[serde(from = "HashMap<Class, u8>")]
pub struct ClassList([u8; 10]);

impl Index<Class> for ClassList {
    type Output = u8;

    #[cfg_attr(feature = "no-panic", no_panic::no_panic)]
    fn index(&self, class: Class) -> &Self::Output {
        &self.0[class as u8 as usize]
    }
}

impl IndexMut<Class> for ClassList {
    #[cfg_attr(feature = "no-panic", no_panic::no_panic)]
    fn index_mut(&mut self, class: Class) -> &mut Self::Output {
        &mut self.0[class as u8 as usize]
    }
}

impl Serialize for ClassList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let count = self.0.iter().filter(|c| **c > 0).count();
        let mut classes = serializer.serialize_map(Some(count))?;
        for (class, count) in self.0.iter().copied().enumerate() {
            if count > 0 {
                classes.serialize_entry(&class, &count)?;
            }
        }

        classes.end()
    }
}

impl From<HashMap<Class, u8>> for ClassList {
    fn from(map: HashMap<Class, u8>) -> Self {
        let mut classes = ClassList::default();

        for (class, count) in map.into_iter() {
            classes[class] = count;
        }

        classes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct UserId(u32);

impl From<u32> for UserId {
    fn from(int: u32) -> Self {
        UserId(int)
    }
}

impl From<u16> for UserId {
    fn from(int: u16) -> Self {
        UserId(int as u32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Spawn {
    pub user: UserId,
    pub class: Class,
    pub team: Team,
    pub tick: u32,
}

impl Spawn {
    pub fn from_event(event: &PlayerSpawnEvent, tick: u32) -> Self {
        Spawn {
            user: UserId::from(event.user_id),
            class: Class::new(event.class),
            team: Team::new(event.team),
            tick,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub classes: ClassList,
    pub name: String,
    pub user_id: UserId,
    pub steam_id: String,
    #[serde(skip)]
    pub entity_id: Option<EntityId>,
    pub team: Team,
}

impl PartialEq for UserInfo {
    fn eq(&self, other: &UserInfo) -> bool {
        self.classes == other.classes && self.name == other.name && self.user_id == other.user_id && self.steam_id == other.steam_id && self.team == other.team
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Death {
    pub weapon: String,
    pub victim: UserId,
    pub victim_steamid: String,
    pub victim_entity: u32,
    pub victim_entity_state: Option<PlayerEntity>,
    pub assister: Option<UserId>,
    pub assister_steamid: Option<String>,
    pub killer: UserId,
    pub killer_steamid: String,
    pub killer_entity: u32, // probably the projectile entity rather than the killer'sm unless it's hitscan?
    pub killer_entity_state: Option<PlayerEntity>,
    pub tick: u32,
    pub round: u32,
}

impl Death {
    pub fn from_event(event: &PlayerDeathEvent, tick: u32, users: &BTreeMap<UserId, UserInfo>, round: u32) -> Self {
        let (assister, assister_steamid) = if event.assister < (16 * 1024) {
            let assister = UserId::from(event.assister);
            (Some(assister), Some(users.get(&assister).expect("Can't get assister").steam_id.clone()))
        } else {
            (None, None)
        };
        let killer = UserId::from(if event.attacker == 0 {
            event.user_id // if world killed the player, count it as a suicide
        } else {
            event.attacker
        });
        let victim = UserId::from(event.user_id);
        Death {
            assister,
            assister_steamid,
            tick,
            round,
            killer,
            killer_steamid: users.get(&killer).expect("Can't get killer").steam_id.clone(),
            killer_entity: if event.attacker == 0 {
                event.victim_ent_index // if world killed the player, count it as a suicide
            } else {
                event.inflictor_ent_index
            },
            killer_entity_state: None,
            weapon: event.weapon.clone(),
            victim,
            victim_steamid: users.get(&victim).expect("Can't get victim").steam_id.clone(),
            victim_entity: event.victim_ent_index,
            victim_entity_state: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Round {
    winner: Team,
    length: f32,
    end_tick: u32,
}

impl Round {
    pub fn from_event(event: &TeamPlayRoundWinEvent, tick: u32) -> Self {
        Round {
            winner: Team::new(event.team),
            length: event.round_time,
            end_tick: tick,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct World {
    boundary_min: Vector,
    boundary_max: Vector,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct HeatmapAnalyser {
    state: HeatmapAnalysis,
    user_id_map: HashMap<EntityId, UserId>,
    class_names: Vec<ServerClassName>, // indexed by ClassId
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PlayerState {
    Alive = 0,
    Dying = 1,
    Death = 2,
    Respawnable = 3,
}

impl PlayerState {
    pub fn new(number: i64) -> Self {
        match number {
            1 => PlayerState::Dying,
            2 => PlayerState::Death,
            3 => PlayerState::Respawnable,
            _ => PlayerState::Alive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerEntity {
    entity: EntityId,
    pub position: Vector,
    pub health: u16,
    pub max_health: u16,
    pub class: Class,
    pub team: Team,
    pub view_angle_horizontal: f32,
    pub view_angle_vertical: f32,
    pub state: PlayerState,
}

impl MessageHandler for HeatmapAnalyser {
    type Output = HeatmapAnalysis;

    fn does_handle(message_type: MessageType) -> bool {
        match message_type {
            MessageType::GameEvent | MessageType::UserMessage | MessageType::ServerInfo | MessageType::PacketEntities => true,
            _ => false,
        }
    }

    fn handle_message(&mut self, message: &Message, tick: u32) {
        if self.state.start_tick == 0 {
            self.state.start_tick = tick;
        }
        self.state.end_tick = tick;
        match message {
            Message::ServerInfo(message) => self.state.interval_per_tick = message.interval_per_tick,
            Message::GameEvent(message) => self.handle_event(&message.event, tick),
            Message::UserMessage(message) => self.handle_user_message(&message, tick),
            Message::PacketEntities(message) => {
                for entity in &message.entities {
                    self.handle_entity(entity);
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

    fn into_output(self, _state: &ParserState) -> Self::Output {
        self.state
    }
}

impl HeatmapAnalyser {
    pub fn handle_entity(&mut self, entity: &PacketEntity) {
        let class_name: &str = self.class_names.get(usize::from(entity.server_class)).map(|class_name| class_name.as_str()).unwrap_or("");
        match class_name {
            "CTFPlayer" => self.handle_player_entity(entity),
            "CTFPlayerResource" => self.handle_player_resource(entity),
            "CWorld" => self.handle_world_entity(entity),
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

    pub fn handle_world_entity(&mut self, entity: &PacketEntity) {
        if let (
            Some(SendProp {
                value: SendPropValue::Vector(boundary_min),
                ..
            }),
            Some(SendProp {
                value: SendPropValue::Vector(boundary_max),
                ..
            }),
        ) = (entity.get_prop_by_name("DT_WORLD", "m_WorldMins"), entity.get_prop_by_name("DT_WORLD", "m_WorldMaxs"))
        {
            self.state.world = Some(World {
                boundary_min: boundary_min.clone(),
                boundary_max: boundary_max.clone(),
            })
        }
    }

    fn handle_user_message(&mut self, message: &UserMessage, tick: u32) {
        if let UserMessage::SayText2(text_message) = message {
            if text_message.kind == ChatMessageKind::NameChange {
                if let Some(from) = text_message.from.clone() {
                    self.change_name(from, text_message.text.clone());
                }
            } else {
                self.state.chat.push(ChatMassage::from_message(text_message, tick));
            }
        }
    }

    fn change_name(&mut self, from: String, to: String) {
        if let Some(user) = self.state.users.values_mut().find(|user| user.name == from) {
            user.name = to;
        }
    }

    fn handle_event(&mut self, event: &GameEvent, tick: u32) {
        const WIN_REASON_TIME_LIMIT: u8 = 6;

        match event {
            GameEvent::PlayerDeath(event) => {
                let round = self.state.rounds.len() as u32 + 1;
                let mut death = Death::from_event(event, tick, &self.state.users, round);
                let killer = self.state.users.get_mut(&death.killer).expect("got a kill from unknown user");
                if death.killer_entity < MAX_PLAYER_ENTITY {
                    killer.entity_id = Some(EntityId::from(death.killer_entity));
                }
                if let Some(killer_entity) = killer.entity_id {
                    death.killer_entity_state = Some(self.state.get_or_create_player_entity(killer_entity).clone());
                }
                let victim = self.state.users.get_mut(&death.victim).expect("got a kill on unknown user");
                if death.victim_entity < MAX_PLAYER_ENTITY {
                    victim.entity_id = Some(EntityId::from(death.victim_entity));
                }
                if let Some(victim_entity) = victim.entity_id {
                    death.victim_entity_state = Some(self.state.get_or_create_player_entity(victim_entity).clone());
                }
                self.state.deaths.push(death);
            }
            GameEvent::PlayerSpawn(event) => {
                let spawn = Spawn::from_event(event, tick);
                if let Some(user_state) = self.state.users.get_mut(&spawn.user) {
                    user_state.classes[spawn.class] += 1;
                    user_state.team = spawn.team;
                }
            }
            GameEvent::TeamPlayRoundWin(event) => {
                if event.win_reason != WIN_REASON_TIME_LIMIT {
                    self.state.rounds.push(Round::from_event(event, tick))
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
                        classes: ClassList::default(),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapAnalysis {
    pub chat: Vec<ChatMassage>,
    pub users: BTreeMap<UserId, UserInfo>,
    pub deaths: Vec<Death>,
    pub rounds: Vec<Round>,
    pub start_tick: u32,
    pub end_tick: u32,
    pub interval_per_tick: f32,

    pub player_entities: Vec<PlayerEntity>,
    pub world: Option<World>,
}

impl Default for HeatmapAnalysis {
    fn default() -> Self {
        Self {
            chat: Default::default(),
            users: {
                let mut users = BTreeMap::new();
                let world = UserInfo {
                    classes: ClassList::default(),
                    entity_id: Some(EntityId::from(0)),
                    name: "world".into(),
                    user_id: UserId::from(0u32),
                    steam_id: "".into(),
                    team: Team::default(),
                };
                users.insert(UserId::from(0u32), world);
                users
            },
            deaths: Default::default(),
            rounds: Default::default(),
            start_tick: Default::default(),
            end_tick: Default::default(),
            interval_per_tick: Default::default(),
            player_entities: {
                let mut player_entities = Vec::new();
                let world = PlayerEntity {
                    class: Class::default(),
                    entity: EntityId::from(0),
                    position: Vector::default(),
                    health: 0,
                    max_health: 0,
                    team: Team::default(),
                    state: PlayerState::Alive,
                    view_angle_horizontal: 0.0,
                    view_angle_vertical: 0.0,
                };
                player_entities.push(world);
                player_entities
            },
            world: Default::default(),
        }
    }
}

impl HeatmapAnalysis {
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
