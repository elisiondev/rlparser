mod models;
mod utils;

use std::collections::HashMap;
use std::io::{self, stdout, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use boxcars::{Attribute, CrcCheck, NetworkParse, ParseError, ParserBuilder, Replay, RigidBody};
use models::{Actor, ActorUpdate, Ball, Game, Player, ReplayOutput, Team};
use utils::{get_array, get_int, get_string};

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

fn serialize<W: Write>(pretty: bool, writer: W, replay: &ReplayOutput) -> anyhow::Result<()> {
    let res = if pretty {
        serde_json::to_writer_pretty(writer, &replay)
    } else {
        serde_json::to_writer(writer, replay)
    };

    res.map_err(|e| e.into())
}

fn run() -> Option<ReplayOutput>{
    let mut active_actors = HashMap::new();
    let mut actors = HashMap::new();
    let mut players:HashMap<String, String> = HashMap::new();
    let mut balls = Vec::new();
    let mut team0 = "".to_string();
    let mut team1 = "".to_string();

    let path = PathBuf::from("data/test3.replay");
    let Ok((_, replay)) = read_file(path) else { todo!()};

    let mut output = ReplayOutput {
        team0: Team::with_score(get_int(&replay.properties, "Team0Score")),
        team1: Team::with_score(get_int(&replay.properties, "Team1Score")),
        players: HashMap::new(),
        ball: Ball::new(),
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
            if actor.object.eq("Archetypes.Ball.Ball_Default") {
                if !balls.contains(&actor.name) {
                    balls.push(actor.name.clone());
                }
            }else if actor.object.eq("Archetypes.Teams.Team0") {
                team0 = actor.name.clone();
            }else if actor.object.eq("Archetypes.Teams.Team1") {
                team1 = actor.name.clone();
            }
            active_actors.insert(new_actor.actor_id.0, actor);
        }

        for updated_actor in &frame.updated_actors {
            let attribute = updated_actor.attribute.clone();
            let attribute_name = replay.objects[updated_actor.object_id.0 as usize].clone();

            if let Attribute::ActiveActor(active_actor) = attribute {
                if active_actor.active {
                    let child = active_actors.get_mut(&updated_actor.actor_id.0).unwrap().name.clone();
                    active_actors.get_mut(&active_actor.actor.0).unwrap().children.push(child);
                }
            };

            let updated_active_actor = active_actors
                .get_mut(&updated_actor.actor_id.0).unwrap();
            
            if attribute_name.eq("Engine.PlayerReplicationInfo:PlayerName") {
                // TODO: handle recreation of players, as in leave/rejoin
                if let Attribute::String(player_name) = &attribute {
                    players.insert(player_name.clone(), updated_active_actor.name.clone());
                };
            }

            let actor_update = ActorUpdate {
                attribute_name,
                value: attribute
            };

            if !updated_active_actor.frames.contains_key(&i) {
                updated_active_actor.frames.insert(i, Vec::new());
            }

            updated_active_actor.frames.get_mut(&i).unwrap().push(actor_update);
        };

        for deleted_actor in &frame.deleted_actors {
            let actor = active_actors.remove(&deleted_actor.0)?;
            actors.insert(actor.name.clone(), actor);
        };
    };

    for (_, left_over) in active_actors {
        actors.insert(left_over.name.clone(), left_over);
    }

    for ball_name in balls {
        if !actors.contains_key(&ball_name) {
            println!("{} not found", ball_name.clone());
            continue;
        }
        let ball_actor = actors.remove(&ball_name).unwrap();
        for (i, updates) in &ball_actor.frames {
            for update in updates {
                if update.attribute_name.eq("TAGame.RBActor_TA:ReplicatedRBState") {
                    if let Attribute::RigidBody(rigid_body) = update.value {
                        output.ball.positions.insert(*i, rigid_body);
                    }
                }
            }
        }
    }

    for player_stats in get_array(&replay.properties, "PlayerStats") {
        let player_name = get_string(&player_stats, "Name");
        let mut player = Player::from_stats(player_stats);

        let player_actor_name = players.get(&player_name).unwrap();
        if !actors.contains_key(player_actor_name) {
            println!("Couldn't find player {}", player_actor_name);
            continue;
        }
        let player_actor = actors.get(player_actor_name).unwrap();
        for (_i, updates) in &player_actor.frames {
            for update in updates {
                if update.attribute_name.eq("TAGame.PRI_TA:ClientLoadouts") && player.loadout == None {
                    if let Attribute::TeamLoadout(loadout) = &update.value {
                        player.loadout = Some(**loadout)
                    }
                }
            }
            if player.loadout != None {
                break;
            }
        }
        for child in &player_actor.children {
            let child_actor = actors.get(child).unwrap();
            if child_actor.object.eq("Archetypes.Car.Car_Default") {
                for (i, child_updates) in &child_actor.frames {
                    for child_update in child_updates {
                        if child_update.attribute_name.eq("TAGame.RBActor_TA:ReplicatedRBState") {
                            if let Attribute::RigidBody(rigid_body) = child_update.value {
                                player.positions.insert(*i, rigid_body);
                            }
                        }
                    }
                }
            } else if child_actor.object.eq("TAGame.Default__CameraSettingsActor_TA") {
                for (_i, child_updates) in &child_actor.frames {
                    for child_update in child_updates {
                        if child_update.attribute_name.eq("TAGame.CameraSettingsActor_TA:ProfileSettings") && player.camera == None {
                            if let Attribute::CamSettings(cam_settings) = &child_update.value {
                                player.camera = Some(**cam_settings)
                            }
                        }
                    }
                    if player.camera != None {
                        break;
                    }
                }
            }
        }
        output.players.insert(player_name.clone(), player); 

    }
    Some(output)
}

fn main(){
    use std::time::Instant;
    let now = Instant::now();
    match run() {
        Some(replay) => {

            //let stdout = io::stdout();
            //let lock = stdout.lock();
            let file = File::create(Path::new("out/replay3.json")).expect("Unable -to open file");
            let _ = serialize(true, BufWriter::new(file), &replay);
        }
        None => {
            println!("Failed");
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
