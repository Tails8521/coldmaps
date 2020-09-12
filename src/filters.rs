use crate::heatmap_analyser::{Class, Death, PlayerEntity, Team};
use enum_dispatch::enum_dispatch;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnorderedOperator {
    Is,
    IsNot,
}

#[enum_dispatch]
pub enum Filter {
    KillerTeamFilter,
    VictimTeamFilter,
    KillerClassFilter,
    VictimClassFilter,
    KillerElevationFilter,
    VictimElevationFilter,
    Distance2DFilter,
    Distance3DFilter,
}

#[enum_dispatch(Filter)]
pub trait FilterTrait {
    fn apply(&self, death: &Death) -> bool;
}

pub struct KillerTeamFilter {
    pub op: UnorderedOperator,
    pub team: Team,
}

impl FilterTrait for KillerTeamFilter {
    fn apply(&self, death: &Death) -> bool {
        match (&death.killer_entity_state, self.op) {
            (Some(PlayerEntity { team, .. }), UnorderedOperator::Is) => *team == self.team,
            (Some(PlayerEntity { team, .. }), UnorderedOperator::IsNot) => *team != self.team,
            (None, _) => false,
        }
    }
}

pub struct VictimTeamFilter {
    pub op: UnorderedOperator,
    pub team: Team,
}

impl FilterTrait for VictimTeamFilter {
    fn apply(&self, death: &Death) -> bool {
        match (&death.victim_entity_state, self.op) {
            (Some(PlayerEntity { team, .. }), UnorderedOperator::Is) => *team == self.team,
            (Some(PlayerEntity { team, .. }), UnorderedOperator::IsNot) => *team != self.team,
            (None, _) => false,
        }
    }
}

pub struct KillerClassFilter {
    pub op: UnorderedOperator,
    pub class: Class,
}

impl FilterTrait for KillerClassFilter {
    fn apply(&self, death: &Death) -> bool {
        match (&death.killer_entity_state, self.op) {
            (Some(PlayerEntity { class, .. }), UnorderedOperator::Is) => *class == self.class,
            (Some(PlayerEntity { class, .. }), UnorderedOperator::IsNot) => *class != self.class,
            (None, _) => false,
        }
    }
}

pub struct VictimClassFilter {
    pub op: UnorderedOperator,
    pub class: Class,
}

impl FilterTrait for VictimClassFilter {
    fn apply(&self, death: &Death) -> bool {
        match (&death.victim_entity_state, self.op) {
            (Some(PlayerEntity { class, .. }), UnorderedOperator::Is) => *class == self.class,
            (Some(PlayerEntity { class, .. }), UnorderedOperator::IsNot) => *class != self.class,
            (None, _) => false,
        }
    }
}

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
