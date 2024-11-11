use boxcars::HeaderProp;

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