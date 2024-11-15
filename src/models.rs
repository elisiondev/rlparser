use std::collections::HashMap;

use boxcars::{ActorId, Attribute, CamSettings, HeaderProp, Replay, RigidBody, TeamLoadout};
use serde::Serialize;

use crate::utils::{get_array, get_int, get_int64, get_platform, get_string};

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct ReplayOutput {
    pub team0: Team,
    pub team1: Team,
    pub players: HashMap<String, Player>,
    pub ball: Ball,
    pub game: Game
}

impl ReplayOutput {
    pub fn from(replay: &Replay) -> ReplayOutput {
        let mut out = ReplayOutput {
            team0: Team::with_score(get_int(&replay.properties, "Team0Score")),
            team1: Team::with_score(get_int(&replay.properties, "Team1Score")),
            players: HashMap::new(),
            ball: Ball::new(),
            game: Game {
                game_type: replay.game_type.clone(),
                match_type: get_string(&replay.properties, "MatchType"),
                team_size: get_int(&replay.properties, "TeamSize"),
                had_bots: false,
                no_contest: false,
                date: get_string(&replay.properties, "Date"),
                id: get_string(&replay.properties, "Id"),
                map_name: get_string(&replay.properties, "MapName"),
            }
        };
        for stat in get_array(&replay.properties, "PlayerStats") {
            let player = Player::from_stats(stat);
            out.players.insert(player.name.clone(), player);
        }
        out
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct Team {
    pub name: Option<String>,
    pub color: Option<i32>,
    pub score: Option<i32>,
    pub winner: Option<bool>,
    pub forfeit: Option<bool>
}

impl Team {
    pub fn with_score(score: i32) -> Team {
        Team {
            name: None,
            color: None,
            score: Some(score),
            winner: None,
            forfeit: None
        }
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
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
    pub camera: Option<CamSettings>,
    pub loadout: Option<TeamLoadout>,
    pub positions: HashMap<usize, RigidBody>
}

impl Player {
    pub fn from_stats(stats: Vec<(String, HeaderProp)>) -> Player {
        let player_name = get_string(&stats, "Name");
        let platform = get_platform(&stats);
        let online_id = get_int64(&stats, "OnlineID");
        Player {
            name: player_name.clone(),
            tag: if platform.eq("Steam") {
                format!("{}/{}", platform, online_id)
            } else {
                format!("{}/{}", platform, player_name)
            },
            platform,
            score: get_int(&stats, "Score"),
            goals: get_int(&stats, "Goals"),
            assists: get_int(&stats, "Assists"),
            saves: get_int(&stats, "Saves"),
            shots: get_int(&stats, "Shots"),
            positions: HashMap::new(),
            mvp: false,
            full_time: true,
            joined_late: false,
            left_early: false,
            camera: None,
            loadout: None,
        }
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct Ball {
    pub positions: HashMap<usize, RigidBody>
}

impl Ball {
    pub fn new() -> Ball {
        Ball {
            positions: HashMap::new()
        }
    }
}

#[derive(Serialize, Debug, PartialEq, Clone)]
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
    pub player: Option<String>,
    pub parent: Option<i32>,
    pub children: Vec<i32>
}

#[derive(Serialize, Debug)]
pub struct ActorUpdate {
    pub attribute_name: String,
    pub value: Attribute
}