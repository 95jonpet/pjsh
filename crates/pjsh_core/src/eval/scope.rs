use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

pub struct Scope {
    frames: Arc<Mutex<Vec<Frame>>>,
}

impl Scope {
    pub fn envs(&self) -> HashMap<String, String> {
        let mut envs = HashMap::new();

        for frame in self.frames.lock().iter().rev() {
            for (key, value) in &frame.env {
                if !envs.contains_key(key) {
                    envs.insert(key.clone(), value.clone());
                }
            }
        }

        envs
    }

    pub fn get_env(&self, name: &str) -> Option<String> {
        for frame in self.frames.lock().iter().rev() {
            if let Some(value) = frame.env.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn set_env(&self, name: String, value: String) {
        if let Some(frame) = self.frames.lock().last_mut() {
            frame.env.insert(name, value);
        }
    }

    pub fn unset_env(&self, name: &str) {
        if let Some(frame) = self.frames.lock().last_mut() {
            frame.env.remove(name);
        }
    }

    pub fn get_alias(&self, name: &str) -> Option<String> {
        for frame in self.frames.lock().iter().rev() {
            if let Some(value) = frame.aliases.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn aliases(&self) -> HashMap<String, String> {
        let mut aliases = HashMap::new();

        for frame in self.frames.lock().iter().rev() {
            for (key, value) in &frame.aliases {
                if !aliases.contains_key(key) {
                    aliases.insert(key.clone(), value.clone());
                }
            }
        }

        aliases
    }

    pub fn set_alias(&self, name: String, value: String) {
        if let Some(frame) = self.frames.lock().last_mut() {
            frame.aliases.insert(name, value);
        }
    }

    pub fn unset_alias(&self, name: &str) {
        if let Some(frame) = self.frames.lock().last_mut() {
            frame.aliases.remove(name);
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            frames: Arc::new(Mutex::new(vec![Frame::new()])),
        }
    }
}

pub struct Frame {
    pub aliases: HashMap<String, String>,
    pub env: HashMap<String, String>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            env: HashMap::new(),
        }
    }
}
