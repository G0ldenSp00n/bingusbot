use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
};

use serde::Deserialize;
use serenity::model::prelude::*;
use toml::de::Error;

#[derive(Deserialize, Debug)]
pub struct ReactionRole {
    pub emoji_id: Option<EmojiId>,
    pub emoji_char: Option<String>,
    pub role_id: RoleId,
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct ReactionRoles {
    pub message_id: Option<MessageId>,
    pub roles: HashMap<String, ReactionRole>,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub reaction_roles: Vec<ReactionRoles>,
}

impl Settings {
    pub fn deserialize() -> Result<Self, Error> {
        let file_str = fs::read_to_string("config/settings.toml").unwrap();
        println!("{}", file_str);
        let reactions_roles: Settings = toml::from_str(file_str.as_str())?;
        return Ok(reactions_roles);
    }
}
