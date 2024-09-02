mod settings;

use std::collections::HashMap;
use std::env;

use serenity::collector::collect;
use serenity::futures::StreamExt;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::{async_trait, prelude::*};
use settings::{ReactionRole, Settings};

const ROLE_MESSAGE_ID: MessageId = MessageId::new(1239812899476738059);
const OTHER_MESSAGE_ID: MessageId = MessageId::new(1239856642405961728);

const MINECRAFT_REACTION_EMOJI: EmojiId = EmojiId::new(1240566802539614238);
const MINECRAFT_ROLE: RoleId = RoleId::new(1240142607103819776);

const SMALL_OUTING_REACTION_EMOJI: &str = "☕";
const SMALL_OUTING_ROLE: RoleId = RoleId::new(1240569555097489489);

struct Handler;

enum CollectorEvent {
    ReactionAdd(Reaction),
    ReactionRemove(Reaction),
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("BingusBot is connected!");

        let settings = Settings::deserialize().expect("Should match the config format");
        println!("{:?}", settings);
        println!(
            "{:?}",
            settings
                .reaction_roles
                .get(0)
                .expect("There should always be at least one reaction role")
                .message_id
        );

        let mut custom_emoji_reaction_to_role_lookup: HashMap<EmojiId, RoleId> = HashMap::new();
        let mut emoji_reaction_to_role_lookup: HashMap<String, RoleId> = HashMap::new();
        for reaction_role in settings.reaction_roles {
            for reaction_name in reaction_role.roles.keys() {
                let reaction_role = reaction_role.roles.get(reaction_name).unwrap();
                if let Some(emoji_id) = reaction_role.emoji_id {
                    custom_emoji_reaction_to_role_lookup.insert(emoji_id, reaction_role.role_id);
                } else if let Some(emoji_string) = reaction_role.emoji_char.as_deref() {
                    emoji_reaction_to_role_lookup
                        .insert(emoji_string.to_owned(), reaction_role.role_id);
                }
            }
        }

        let reaction_collector = collect(&ctx.shard, |event| match event {
            Event::ReactionAdd(event) => Some(CollectorEvent::ReactionAdd(event.reaction.clone())),
            Event::ReactionRemove(event) => {
                Some(CollectorEvent::ReactionRemove(event.reaction.clone()))
            }
            _ => None,
        });

        reaction_collector
            .for_each(|reaction_event| async {
                match reaction_event {
                    CollectorEvent::ReactionAdd(reaction) => {
                        match reaction.message_id {
                            ROLE_MESSAGE_ID => {
                                match reaction.emoji {
                                    ReactionType::Custom {
                                        animated: _,
                                        id: MINECRAFT_REACTION_EMOJI,
                                        name: _,
                                    } => {
                                        if let Some(member) = reaction.member {
                                            member
                                                .add_role(&ctx.http, MINECRAFT_ROLE)
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    ReactionType::Unicode(emoji) => match emoji.as_str() {
                                        SMALL_OUTING_REACTION_EMOJI => {
                                            if let Some(member) = reaction.member {
                                                member
                                                    .add_role(&ctx.http, SMALL_OUTING_ROLE)
                                                    .await
                                                    .unwrap();
                                            }
                                        }
                                        _ => (),
                                    },
                                    _ => {
                                        println!("Unknown Emoji Reaction {}", reaction.emoji);
                                    }
                                };
                            }
                            OTHER_MESSAGE_ID => (),
                            _ => (),
                        };
                    }
                    CollectorEvent::ReactionRemove(reaction) => match reaction.message_id {
                        ROLE_MESSAGE_ID => {
                            match reaction.emoji {
                                ReactionType::Custom {
                                    animated: _,
                                    id: MINECRAFT_REACTION_EMOJI,
                                    name: _,
                                } => {
                                    if let Some(guild_id) = reaction.guild_id {
                                        if let Some(user_id) = reaction.user_id {
                                            let member =
                                                &ctx.http.get_member(guild_id, user_id).await;
                                            if let Ok(member) = member {
                                                member
                                                    .remove_role(&ctx.http, MINECRAFT_ROLE)
                                                    .await
                                                    .unwrap();
                                            }
                                        }
                                    }
                                }
                                ReactionType::Unicode(emoji) => match emoji.as_str() {
                                    SMALL_OUTING_REACTION_EMOJI => {
                                        if let Some(guild_id) = reaction.guild_id {
                                            if let Some(user_id) = reaction.user_id {
                                                let member =
                                                    &ctx.http.get_member(guild_id, user_id).await;
                                                if let Ok(member) = member {
                                                    member
                                                        .remove_role(&ctx.http, SMALL_OUTING_ROLE)
                                                        .await
                                                        .unwrap();
                                                }
                                            }
                                        }
                                    }
                                    _ => (),
                                },

                                _ => {
                                    println!("Unknown Emoji Reaction {}", reaction.emoji);
                                }
                            };
                        }
                        OTHER_MESSAGE_ID => {
                            println!("{} other !!!", reaction.emoji);
                        }
                        _ => (),
                    },
                };
            })
            .await;
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's id.
    let _bot_id = match http.get_current_user().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access user info: {:?}", why),
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
