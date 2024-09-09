use serenity::all::{ChannelId, Event, ReactionType};
use serenity::collector::collect;
use serenity::futures::StreamExt;
use serenity::prelude::*;

use crate::settings::Settings;
use crate::CollectorEvent;

pub struct ReactionRole {}

impl ReactionRole {
    pub async fn register(ctx: &Context, settings: &Settings) -> () {
        let message_id_to_emoji_reaction_to_role_lookup =
            settings.message_id_to_emoji_reaction_to_reactionrole_lookup();

        let channel_id = ChannelId::new(1282242098380406784);
        for message_id in message_id_to_emoji_reaction_to_role_lookup.keys() {
            let message = channel_id.message(&ctx.http, message_id).await.unwrap();
            for reaction_role in message_id_to_emoji_reaction_to_role_lookup
                .get(message_id)
                .unwrap()
                .values()
            {
                if let Some(emoji_id) = reaction_role.emoji_id {
                    message
                        .react(
                            ctx,
                            ReactionType::Custom {
                                animated: false,
                                id: emoji_id,
                                name: Some(reaction_role.title.clone()),
                            },
                        )
                        .await
                        .unwrap();
                }

                if let Some(emoji_char) = reaction_role.emoji_char.clone() {
                    message
                        .react(ctx, ReactionType::Unicode(emoji_char.to_string()))
                        .await
                        .unwrap();
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
                        if let Some(member) = reaction.member {
                            if let Some(emoji_reaction_to_role_lookup) =
                                message_id_to_emoji_reaction_to_role_lookup
                                    .get(&reaction.message_id)
                            {
                                match reaction.emoji {
                                    ReactionType::Custom {
                                        animated: _,
                                        id,
                                        name: _,
                                    } => {
                                        if let Some(role_id) =
                                            emoji_reaction_to_role_lookup.get(&(id.to_string()))
                                        {
                                            member
                                                .add_role(&ctx.http, role_id.role_id)
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    ReactionType::Unicode(emoji) => {
                                        if let Some(role_id) =
                                            emoji_reaction_to_role_lookup.get(&emoji)
                                        {
                                            member
                                                .add_role(&ctx.http, role_id.role_id)
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    _ => {
                                        println!("Unknown Emoji Reaction {}", reaction.emoji);
                                    }
                                };
                            }
                        }
                    }
                    CollectorEvent::ReactionRemove(reaction) => {
                        if let Some(emoji_reaction_to_role_lookup) =
                            message_id_to_emoji_reaction_to_role_lookup.get(&reaction.message_id)
                        {
                            match reaction.emoji {
                                ReactionType::Custom {
                                    animated: _,
                                    id,
                                    name: _,
                                } => {
                                    if let Some(guild_id) = reaction.guild_id {
                                        if let Some(user_id) = reaction.user_id {
                                            let member =
                                                &ctx.http.get_member(guild_id, user_id).await;
                                            if let Ok(member) = member {
                                                if let Some(role_id) = emoji_reaction_to_role_lookup
                                                    .get(&(id.to_string()))
                                                {
                                                    member
                                                        .remove_role(&ctx.http, role_id.role_id)
                                                        .await
                                                        .unwrap();
                                                }
                                            }
                                        }
                                    }
                                }
                                ReactionType::Unicode(emoji) => {
                                    if let Some(guild_id) = reaction.guild_id {
                                        if let Some(user_id) = reaction.user_id {
                                            let member =
                                                &ctx.http.get_member(guild_id, user_id).await;
                                            if let Ok(member) = member {
                                                if let Some(role_id) =
                                                    emoji_reaction_to_role_lookup.get(&emoji)
                                                {
                                                    member
                                                        .remove_role(&ctx.http, role_id.role_id)
                                                        .await
                                                        .unwrap();
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    println!("Unknown Emoji Reaction {}", reaction.emoji);
                                }
                            };
                        }
                    }
                }
            })
            .await;
    }
}
