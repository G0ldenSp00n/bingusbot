use std::{collections::HashMap, env, fs};

use serde::Deserialize;
use serenity::model::prelude::*;
use toml::de::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct ReactionRole {
    pub emoji_id: Option<EmojiId>,
    pub emoji_char: Option<String>,
    pub role_id: RoleId,
    pub title: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReactionRoles {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub roles: HashMap<String, ReactionRole>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameQueues {
    pub game_name: String,
    pub roles_message_id: MessageId,
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Flags {
    pub deadlock_queue_start: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct General {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub general: General,
    pub reaction_roles: Vec<ReactionRoles>,
    pub flags: Flags,
    pub game_queue: Vec<GameQueues>,
}

impl Settings {
    pub fn deserialize() -> Result<Self, Error> {
        let file_str = fs::read_to_string(
            env::var("CONFIG_PATH").unwrap_or("config/settings.toml".to_string()),
        )
        .unwrap();
        let reactions_roles: Settings = toml::from_str(file_str.as_str())?;
        return Ok(reactions_roles);
    }

    pub fn message_id_to_channel_id(&self) -> HashMap<MessageId, ChannelId> {
        let mut message_id_to_channel_id_hashmap: HashMap<MessageId, ChannelId> = HashMap::new();
        for reaction_role_message in self.reaction_roles.clone() {
            let message_id = reaction_role_message.message_id;
            let channel_id = reaction_role_message.channel_id;
            message_id_to_channel_id_hashmap.insert(message_id, channel_id);
        }
        message_id_to_channel_id_hashmap
    }

    pub fn message_id_to_emoji_reaction_to_reactionrole_lookup(
        &self,
    ) -> HashMap<MessageId, HashMap<String, ReactionRole>> {
        let mut message_id_to_emoji_reaction_to_role_lookup: HashMap<
            MessageId,
            HashMap<String, ReactionRole>,
        > = HashMap::new();
        for reaction_role_message in self.reaction_roles.clone() {
            let message_id = reaction_role_message.message_id;
            if !message_id_to_emoji_reaction_to_role_lookup.contains_key(&message_id) {
                message_id_to_emoji_reaction_to_role_lookup.insert(message_id, HashMap::new());
            }

            for reaction_name in reaction_role_message.roles.keys() {
                let reaction_role = reaction_role_message.roles.get(reaction_name).unwrap();
                if let Some(emoji_id) = reaction_role.emoji_id {
                    if let Some(emoji_reaction_to_role_lookup) =
                        message_id_to_emoji_reaction_to_role_lookup.get_mut(&message_id)
                    {
                        emoji_reaction_to_role_lookup
                            .insert(emoji_id.to_string(), reaction_role.clone());
                    }
                } else if let Some(emoji_string) = reaction_role.emoji_char.as_deref() {
                    if let Some(emoji_reaction_to_role_lookup) =
                        message_id_to_emoji_reaction_to_role_lookup.get_mut(&message_id)
                    {
                        emoji_reaction_to_role_lookup
                            .insert(emoji_string.to_owned(), reaction_role.clone());
                    }
                }
            }
        }
        message_id_to_emoji_reaction_to_role_lookup
    }
}
