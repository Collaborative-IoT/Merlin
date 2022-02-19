use std::collections::{HashMap, HashSet};

pub fn add_to_map<V>(map: &mut HashMap<String, V>, key: String, data: V) -> bool {
    if !map.contains_key(&key) {
        map.insert(key, data);
        return true;
    } else {
        return false;
    }
}

pub fn add_to_set(set: &mut HashSet<String>, key: String) -> bool {
    if !set.contains(&key) {
        set.insert(key);
        return true;
    } else {
        return false;
    }
}
