use std::collections::HashMap;

use boxcars::{ActorId, Attribute, RigidBody};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ReplayOutput {
    pub team0: Team,
    pub team1: Team,
    pub players: HashMap<String, Player>,
    pub ball: Ball,
    pub game: Game
}

#[derive(Serialize, Debug)]
pub struct Team {
    pub name: String,
    pub color: i32,
    pub score: i32,
    pub winner: bool,
    pub forfeit: bool
}

#[derive(Serialize, Debug)]
pub struct Player {
    pub name: String,
    pub tag: String,
    pub platform: String,
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub mvp: bool,
    pub full_time: bool,
    pub joined_late: bool,
    pub left_early: bool,
    pub camera: Option<CameraSettings>,
    pub loadout: Option<ClientLoadout>,
    pub title: Option<String>,
    pub positions: HashMap<usize, RigidBody>
}

#[derive(Serialize, Debug)]
pub struct CameraSettings {
    pub fov: f32,
    pub height: f32,
    pub angle: f32,
    pub distance: f32,
    pub stiffness: f32,
    pub swivel: f32,
    pub transition: f32
}

#[derive(Serialize, Debug)]
pub struct ClientLoadout {
    pub version: u32,
    pub body: u32,
    pub decal: u32,
    pub wheels: u32,
    pub rocket_trail: u32,
    pub antenna: u32,
    pub topper: u32,
    pub engine: u32,
    pub goal_explosion: u32,
    pub banner: u32,
}

#[derive(Serialize, Debug)]
pub struct Ball {
    pub positions: HashMap<usize, RigidBody>
}

#[derive(Serialize, Debug)]
pub struct Game {
    pub game_type: String,
    pub match_type: String,
    pub team_size: i32,
    pub date: String,
    pub id: String,
    pub map_name: String,
    pub had_bots: bool,
    pub no_contest: bool
}

#[derive(Serialize, Debug)]
pub struct Actor {
    pub id: ActorId,
    pub name: String,
    pub object: String,
    pub frames: HashMap<usize, Vec<ActorUpdate>>,
    pub parent: String,
    pub children: Vec<String>
}

#[derive(Serialize, Debug)]
pub struct ActorUpdate {
    pub attribute_name: String,
    pub value: Attribute
}