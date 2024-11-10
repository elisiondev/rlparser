mod models;
mod utils;

use std::collections::HashMap;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::fs::{self};
use boxcars::{Attribute, CrcCheck, NetworkParse, ParseError, ParserBuilder, Replay};
use models::{Actor, ActorUpdate, Ball, Game, Player, ReplayOutput, Team};
use utils::{get_array, get_int, get_platform, get_string};

fn read_file(file_path: PathBuf) -> anyhow::Result<(PathBuf, Replay)> {
    // Try to mmap the file first so we don't have to worry about potentially allocating a large
    // buffer in case there is like a 10GB iso file that ends in .replay

    // from rrrocket
    let f = fs::File::open(&file_path)?;
    let mmap = unsafe { memmap2::MmapOptions::new().map(&f) };
    match mmap {
        Ok(data) => {
            let replay = parse_replay(&data)?;
            Ok((file_path, replay))
        }
        Err(_) => {
            // If the mmap fails, just try reading the file
            let data = fs::read(&file_path)?;
            let replay = parse_replay(&data)?;
            Ok((file_path, replay))
        }
    }
}

fn parse_replay(data: &[u8]) -> Result<Replay, ParseError> {
    ParserBuilder::new(data)
        .with_crc_check(CrcCheck::Always)
        .with_network_parse(NetworkParse::Always)
        .parse()
}

fn serialize<W: Write>(pretty: bool, writer: W, replay: &Replay) -> anyhow::Result<()> {
    let res = if pretty {
        serde_json::to_writer_pretty(writer, &replay)
    } else {
        serde_json::to_writer(writer, replay)
    };

    res.map_err(|e| e.into())
}

fn run() -> Option<HashMap<String, Actor>>{
    let mut active_actors = HashMap::new();
    let mut actors = HashMap::new();
    let mut players:HashMap<String, String> = HashMap::new();
    let mut balls = Vec::new();
    let mut team0 = "".to_string();
    let mut team1 = "".to_string();

    let path = PathBuf::from("C:/Users/parml/replay/rlparser/data/test3.replay");
    let Ok((_, replay)) = read_file(path) else { todo!()};

    let mut output = ReplayOutput {
        team0: Team { 
            name: "".to_string(), 
            color: 0, 
            score: get_int(&replay.properties, "Team0Score"), 
            winner: false, 
            forfeit: false 
        },
        team1: Team { 
            name: "".to_string(), 
            color: 0, 
            score: get_int(&replay.properties, "Team1Score"), 
            winner: false, 
            forfeit: false 
        },
        players: HashMap::new(),
        ball: Ball {
            positions: HashMap::new(),
        },
        game: Game {
            game_type: replay.game_type,
            match_type: get_string(&replay.properties, "MatchType"),
            team_size: get_int(&replay.properties, "TeamSize"),
            had_bots: false,
            no_contest: false,
            date: get_string(&replay.properties, "Date"),
            id: get_string(&replay.properties, "Id"),
            map_name: get_string(&replay.properties, "MapName"),
        },
    };
    
    for (i, frame) in replay.network_frames?.frames.iter().enumerate() {
        for new_actor in &frame.new_actors {
            let actor = Actor {
                id: new_actor.actor_id,
                name: replay.names[new_actor.name_id? as usize].clone(),
                object: replay.objects[new_actor.object_id.0 as usize].clone(),
                frames: HashMap::new(),
                parent: String::from(""),
                children: Vec::new()
            };
            if actor.object == "Archetypes.Ball.Ball_Default".to_string() {
                balls.push(actor.name.clone());
            }else if actor.object == "Archetypes.Teams.Team0".to_string() {
                team0 = actor.name.clone();
            }else if actor.object == "Archetypes.Teams.Team1".to_string() {
                team1 = actor.name.clone();
            }
            active_actors.insert(new_actor.actor_id, actor);
        }

        for updated_actor in &frame.updated_actors {
            let attribute = updated_actor.attribute.clone();
            let attribute_name = replay.objects[updated_actor.object_id.0 as usize].clone();

            let mut parent = "".to_string();
            if let Attribute::ActiveActor(active_actor) = attribute {
                let child = active_actors.get_mut(&updated_actor.actor_id)?.name.clone();
                parent = active_actors.get_mut(&active_actor.actor)?.name.clone();
                active_actors.get_mut(&active_actor.actor)?.children.push(child);
            }

            let updated_active_actor = active_actors
                .get_mut(&updated_actor.actor_id)?;
            
            if attribute_name == "Engine.PlayerReplicationInfo:PlayerName".to_string() {
                // TODO: handle recreation of players, as in leave/rejoin
                if let Attribute::String(player_name) = &attribute {
                    players.insert(player_name.clone(), updated_active_actor.name.clone());
                }
            }

            let actor_update = ActorUpdate {
                attribute_name: attribute_name,
                value: attribute
            };

            updated_active_actor
                .frames.get_mut(&i)
                .get_or_insert(&mut Vec::new())
                .push(actor_update);

            if !parent.is_empty() {
                updated_active_actor.parent = parent;
            }
        }

        for deleted_actor in &frame.deleted_actors {
            let actor = active_actors.remove(deleted_actor)?;
            actors.insert(actor.name.clone(), actor);
        }
    };

    for player_stats in get_array(&replay.properties, "PlayerStats") {
        let player_name = get_string(&player_stats, "Name");
        let online_id = get_string(&player_stats, "OnlineID");
        let platform = get_platform(&player_stats);
        let player = Player {
            name: player_name.clone(),
            tag: if platform == "Steam".to_string() {
                format!("{}/{}", platform, online_id)
            } else {
                format!("{}/{}", platform, player_name)
            },
            platform,
            score: get_int(&player_stats, "Score"),
            goals: get_int(&player_stats, "Goals"),
            assists: get_int(&player_stats, "Assists"),
            saves: get_int(&player_stats, "Saves"),
            shots: get_int(&player_stats, "Shots"),
            mvp: false,
            full_time: false,
            joined_late: false,
            left_early: false,
            camera: None,
            loadout: None,
            title: None,
            positions: HashMap::new(),
        };
        output.players.insert(player_name.clone(), player); 
    }
    return Some(actors);
}

fn main(){
    run();
    //serde_json.run();
}
