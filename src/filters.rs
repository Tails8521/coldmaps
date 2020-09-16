use crate::heatmap_analyser::{Death, PlayerEntity, PlayerState, Team};
use enum_dispatch::enum_dispatch;
use std::fmt::Display;
use tf_demo_parser::demo::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedOperator {
    Equal,
    NotEqual,
    Greater,
    Smaller,
    GreaterOrEqual,
    SmallerOrEqual,
}

impl OrderedOperator {
    pub const ALL: [OrderedOperator; 6] = [
        OrderedOperator::Equal,
        OrderedOperator::NotEqual,
        OrderedOperator::Greater,
        OrderedOperator::Smaller,
        OrderedOperator::GreaterOrEqual,
        OrderedOperator::SmallerOrEqual,
    ];
}

impl Default for OrderedOperator {
    fn default() -> Self {
        OrderedOperator::Equal
    }
}

impl Display for OrderedOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderedOperator::Equal => write!(f, "="),
            OrderedOperator::NotEqual => write!(f, "≠"),
            OrderedOperator::Greater => write!(f, ">"),
            OrderedOperator::Smaller => write!(f, "<"),
            OrderedOperator::GreaterOrEqual => write!(f, "≥"),
            OrderedOperator::SmallerOrEqual => write!(f, "≤"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyOperator {
    IsPresent,
    IsNotPresent,
}

impl PropertyOperator {
    pub const ALL: [PropertyOperator; 2] = [PropertyOperator::IsPresent, PropertyOperator::IsNotPresent];
}

impl Default for PropertyOperator {
    fn default() -> Self {
        PropertyOperator::IsPresent
    }
}

impl Display for PropertyOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyOperator::IsPresent => write!(f, "present"),
            PropertyOperator::IsNotPresent => write!(f, "not present"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Property {
    Suicide,
    Posthumous,
}

impl Property {
    pub const ALL: [Property; 2] = [Property::Suicide, Property::Posthumous];
}

impl Default for Property {
    fn default() -> Self {
        Property::Suicide
    }
}

impl Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Property::Suicide => write!(f, "Suicide"),
            Property::Posthumous => write!(f, "Posthumous"),
        }
    }
}

#[enum_dispatch]
#[derive(Debug)]
pub enum Filter {
    KillerTeamFilter,
    VictimTeamFilter,
    KillerClassFilter,
    VictimClassFilter,
    KillerElevationFilter,
    VictimElevationFilter,
    Distance2DFilter,
    Distance3DFilter,
    RoundFilter,
    PropertyFilter,
}

#[enum_dispatch(Filter)]
pub trait FilterTrait {
    fn apply(&self, death: &Death) -> bool;
}

#[derive(Debug)]
pub struct KillerTeamFilter {
    pub team: Team,
}

impl FilterTrait for KillerTeamFilter {
    fn apply(&self, death: &Death) -> bool {
        match &death.killer_entity_state {
            Some(PlayerEntity { team, .. }) => *team == self.team,
            None => false,
        }
    }
}

#[derive(Debug)]
pub struct VictimTeamFilter {
    pub team: Team,
}

impl FilterTrait for VictimTeamFilter {
    fn apply(&self, death: &Death) -> bool {
        match &death.victim_entity_state {
            Some(PlayerEntity { team, .. }) => *team == self.team,
            None => false,
        }
    }
}

#[derive(Debug)]
pub struct KillerClassFilter {
    pub classes: [bool; 10],
}

impl FilterTrait for KillerClassFilter {
    fn apply(&self, death: &Death) -> bool {
        match &death.killer_entity_state {
            Some(PlayerEntity { class, .. }) => self.classes[*class as usize],
            None => false,
        }
    }
}

#[derive(Debug)]
pub struct VictimClassFilter {
    pub classes: [bool; 10],
}

impl FilterTrait for VictimClassFilter {
    fn apply(&self, death: &Death) -> bool {
        match &death.victim_entity_state {
            Some(PlayerEntity { class, .. }) => self.classes[*class as usize],
            None => false,
        }
    }
}

#[derive(Debug)]
pub struct KillerElevationFilter {
    pub op: OrderedOperator,
    pub z: f32,
}

impl FilterTrait for KillerElevationFilter {
    fn apply(&self, death: &Death) -> bool {
        match (&death.killer_entity_state, self.op) {
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::Equal) => *z == self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::NotEqual) => *z != self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::Greater) => *z > self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::Smaller) => *z < self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::GreaterOrEqual) => *z >= self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::SmallerOrEqual) => *z <= self.z,
            (None, _) => false,
        }
    }
}

#[derive(Debug)]
pub struct VictimElevationFilter {
    pub op: OrderedOperator,
    pub z: f32,
}

impl FilterTrait for VictimElevationFilter {
    fn apply(&self, death: &Death) -> bool {
        match (&death.victim_entity_state, self.op) {
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::Equal) => *z == self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::NotEqual) => *z != self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::Greater) => *z > self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::Smaller) => *z < self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::GreaterOrEqual) => *z >= self.z,
            (Some(PlayerEntity { position: Vector { z, .. }, .. }), OrderedOperator::SmallerOrEqual) => *z <= self.z,
            (None, _) => false,
        }
    }
}

#[derive(Debug)]
pub struct Distance2DFilter {
    pub op: OrderedOperator,
    pub distance: f32,
}

impl FilterTrait for Distance2DFilter {
    fn apply(&self, death: &Death) -> bool {
        if let (Some(killer_entity), Some(victim_entity)) = (&death.killer_entity_state, &death.victim_entity_state) {
            let distance_x = killer_entity.position.x - victim_entity.position.x;
            let distance_y = killer_entity.position.y - victim_entity.position.y;
            let distance = (distance_x * distance_x + distance_y * distance_y).sqrt();
            match self.op {
                OrderedOperator::Equal => distance == self.distance,
                OrderedOperator::NotEqual => distance != self.distance,
                OrderedOperator::Greater => distance > self.distance,
                OrderedOperator::Smaller => distance < self.distance,
                OrderedOperator::GreaterOrEqual => distance >= self.distance,
                OrderedOperator::SmallerOrEqual => distance <= self.distance,
            }
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct Distance3DFilter {
    pub op: OrderedOperator,
    pub distance: f32,
}

impl FilterTrait for Distance3DFilter {
    fn apply(&self, death: &Death) -> bool {
        if let (Some(killer_entity), Some(victim_entity)) = (&death.killer_entity_state, &death.victim_entity_state) {
            let distance_x = killer_entity.position.x - victim_entity.position.x;
            let distance_y = killer_entity.position.y - victim_entity.position.y;
            let distance_z = killer_entity.position.z - victim_entity.position.z;
            let distance = (distance_x * distance_x + distance_y * distance_y + distance_z * distance_z).sqrt();
            match self.op {
                OrderedOperator::Equal => distance == self.distance,
                OrderedOperator::NotEqual => distance != self.distance,
                OrderedOperator::Greater => distance > self.distance,
                OrderedOperator::Smaller => distance < self.distance,
                OrderedOperator::GreaterOrEqual => distance >= self.distance,
                OrderedOperator::SmallerOrEqual => distance <= self.distance,
            }
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct RoundFilter {
    pub op: OrderedOperator,
    pub round: u32,
}

impl FilterTrait for RoundFilter {
    fn apply(&self, death: &Death) -> bool {
        match self.op {
            OrderedOperator::Equal => death.round == self.round,
            OrderedOperator::NotEqual => death.round != self.round,
            OrderedOperator::Greater => death.round > self.round,
            OrderedOperator::Smaller => death.round < self.round,
            OrderedOperator::GreaterOrEqual => death.round >= self.round,
            OrderedOperator::SmallerOrEqual => death.round <= self.round,
        }
    }
}

#[derive(Debug)]
pub struct PropertyFilter {
    pub op: PropertyOperator,
    pub property: Property,
}

impl FilterTrait for PropertyFilter {
    fn apply(&self, death: &Death) -> bool {
        let ret = match self.property {
            Property::Suicide => death.killer == death.victim,
            Property::Posthumous => match death.killer_entity_state {
                Some(PlayerEntity { state: PlayerState::Alive, .. }) => false,
                _ => true,
            },
        };
        match self.op {
            PropertyOperator::IsPresent => ret,
            PropertyOperator::IsNotPresent => !ret,
        }
    }
}
