mod models;
mod utils;

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use boxcars::{Attribute, CrcCheck, NetworkParse, ParseError, ParserBuilder, Replay};
use models::{Actor, ReplayOutput};
use utils::{add_ball_position, add_player_position, get_actor_player, get_actor_type, lookup_object, set_parent, set_player};

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

fn run(file_name: &str) -> Option<ReplayOutput>{
    let mut active_actors = HashMap::new();

    let path = PathBuf::from(format!("data/{file_name}.replay"));
    let Ok((_, replay)) = read_file(path) else { todo!()};

    let mut output = ReplayOutput::from(&replay);
    
    for (i, frame) in replay.clone().network_frames?.frames.iter().enumerate() {
        for new_actor in &frame.new_actors {
            let actor = Actor {
                id: new_actor.actor_id,
                name: replay.names[new_actor.name_id? as usize].clone(),
                object: replay.objects[new_actor.object_id.0 as usize].clone(),
                frames: HashMap::new(),
                player: None,
                parent: None,
                children: Vec::new()
            };
            active_actors.insert(new_actor.actor_id.0, actor);
        }

        for updated_actor in &frame.updated_actors {
            let attribute = &updated_actor.attribute;
            let attribute_name = lookup_object(&replay, updated_actor.object_id);
            let updated_actor_id = updated_actor.actor_id.0;

            let updated_active_actor_type = get_actor_type(&active_actors, updated_actor_id);

            match attribute_name.as_str() {
                // component to car
                "TAGame.CarComponent_TA:Vehicle" | 
                // car to player
                "Engine.Pawn:PlayerReplicationInfo" |
                // camera to player
                "TAGame.CameraSettingsActor_TA:PRI" => {
                    if let Attribute::ActiveActor(active_actor) = &attribute {
                        if active_actor.active {
                            set_parent(&mut active_actors, updated_actor_id, active_actor.actor.0);
                        }
                    };
                }
                // until this we don't know who a player actor actually is
                "Engine.PlayerReplicationInfo:PlayerName" => {
                    if let Attribute::String(player_name) = &attribute {
                        set_player(&mut active_actors, updated_actor_id, player_name.clone());
                    };
                }
                // TODO: Camera not working now, player is none on transmit
                "TAGame.CameraSettingsActor_TA:ProfileSettings" => {
                    let updated_actor_player = get_actor_player(&active_actors, updated_actor_id);
                    if !output.players.contains_key(&updated_actor_player) { continue; }

                    if let Attribute::CamSettings(cam_settings) = &attribute {
                        output.players.get_mut(&updated_actor_player).unwrap().camera = Some(**cam_settings);
                    };
                }
                "TAGame.RBActor_TA:ReplicatedRBState" => {
                    if let Attribute::RigidBody(rigid_body) = &attribute {
                        if updated_active_actor_type.eq("Archetypes.Ball.Ball_Default") {
                            add_ball_position(&mut output, i, *rigid_body);
                        }
                        else if updated_active_actor_type.eq("Archetypes.Car.Car_Default") {
                            let updated_actor_player = get_actor_player(&active_actors, updated_actor_id);
                            if output.players.contains_key(&updated_actor_player) {
                                add_player_position(&mut output, &updated_actor_player, i, *rigid_body);
                            }
                        }
                    }
                }
                "TAGame.Team_TA:CustomTeamName" => {
                    if let Attribute::String(name) = attribute{
                        if updated_active_actor_type.eq("Archetypes.Teams.Team0") {
                            output.team0.name = Some(name.clone());
                        }else{
                            output.team1.name = Some(name.clone());
                        }
                    }
                }
                _ => {}
            }
        };
    };

    Some(output)
}

fn main(){
    use std::time::Instant;
    let now = Instant::now();
    let file_name = "test3";
    match run(file_name) {
        Some(replay) => {

            //let stdout = io::stdout();
            //let lock = stdout.lock();
            let file = File::create(Path::new(&format!("out/{file_name}.json"))).expect("Unable -to open file");
            let _ = serialize(true, BufWriter::new(file), &replay);
        }
        None => {
            println!("Failed");
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
