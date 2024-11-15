use std::collections::HashMap;

use boxcars::{HeaderProp, ObjectId, Replay, RigidBody};

use crate::models::{Actor, ReplayOutput};

pub fn get_header_value(properties: &Vec<(String, HeaderProp)>, key: &str) -> Option<HeaderProp> {
    for (prop_name, prop_value) in properties {
        if prop_name.eq(key) {
            return Some(prop_value.clone());
        }
    }
    None
}

pub fn get_int(properties: &Vec<(String, HeaderProp)>, key: &str) -> i32 {
    get_header_value(properties, key).unwrap().as_i32().unwrap()
}

pub fn get_string(properties: &Vec<(String, HeaderProp)>, key: &str) -> String {
    match get_header_value(properties, key) {
        Some(prop) => {
            match prop.as_string() {
                Some(out) => {
                    return out.to_string();
                }
                None => {
                    panic!("Couldn't convert {} to string", key);
                }
            }
        }
        None => {
            panic!("Couldn't find prop {}", key);
        }
    }
}

pub fn get_int64(properties: &Vec<(String, HeaderProp)>, key: &str) -> u64 {
    match get_header_value(properties, key) {
        Some(prop) => {
            match prop.as_u64() {
                Some(out) => {
                    return out;
                }
                None => {
                    panic!("Couldn't convert {} to u64", key);
                }
            }
        }
        None => {
            panic!("Couldn't find prop {}", key);
        }
    }
}

pub fn get_array(properties: &Vec<(String, HeaderProp)>, key: &str) -> Vec<Vec<(String, HeaderProp)>> {
    get_header_value(properties, key).unwrap().as_array().unwrap().clone()
}

pub fn get_byte(properties: &Vec<(String, HeaderProp)>, key: &str) -> Option<String> {
    match get_header_value(properties, key).unwrap() {
        HeaderProp::Byte {
            kind: _, 
            value 
        } => {
            return value
        }
        _ => ()
    }
    None
}

pub fn get_platform(properties: &Vec<(String, HeaderProp)>) -> String {
    get_byte(properties, "Platform")
        .unwrap_or("Unknown".to_string())
        .replace("OnlinePlatform_","")
}

pub fn get_actor(actors: &HashMap<i32, Actor>, id: i32) -> &Actor {
    match actors.get(&id) {
        Some(actor) => actor,
        None => panic!("Could not find actor {}", id)
    }
}

pub fn get_actor_mut(actors: &mut HashMap<i32, Actor>, id: i32) -> &mut Actor {
    match actors.get_mut(&id) {
        Some(actor) => actor,
        None => panic!("Could not find actor {}", id)
    }
}

pub fn get_actor_type(actors: &HashMap<i32, Actor>, id: i32) -> String {
    get_actor(actors, id).object.clone()
}

pub fn get_actor_player(actors: &HashMap<i32, Actor>, id: i32) -> String {
    match &get_actor(actors, id).player {
        Some(player) => player.clone(),
        None => String::from("None")
    }
}

pub fn lookup_object(replay: &Replay, id: ObjectId) -> &String {
    &replay.objects[id.0 as usize]
}

pub fn set_parent(actors: &mut HashMap<i32, Actor>, id: i32, parent: i32) {
    get_actor_mut(actors, id).parent = Some(parent);
    //get_actor_mut(actors, id).player = get_actor(actors, parent).player.clone();
    // attempt to skip child loop if we already know player
    if get_actor(actors, parent).player != None {
        get_actor_mut(actors, id).player = get_actor(actors, parent).player.clone();
        return;
    }
    if get_actor(actors, parent).children.contains(&id) {return;}
    get_actor_mut(actors, parent).children.push(id);
}

pub fn set_player(actors: &mut HashMap<i32, Actor>, id: i32, player: String) {
    get_actor_mut(actors, id).player = Some(player.clone());
    for child in get_actor_mut(actors, id).children.to_owned() {
        //get_actor_mut(actors, id).children.remove(i);
        get_actor_mut(actors, child).player = Some(player.clone());
    }
}

pub fn add_ball_position(output: &mut ReplayOutput, frame: usize, rigid_body: RigidBody) {
    output.ball.positions.insert(frame, rigid_body);
}

pub fn add_player_position(output: &mut ReplayOutput, player_name: &String, frame: usize, rigid_body: RigidBody) {
    match output.players.get_mut(player_name) {
        Some(player) => player.positions.insert(frame, rigid_body),
        None => panic!("Player on field that doesn't belong {}", player_name)
    };
}