mod commands;
mod reaction_roles;
mod settings;

use std::env;

use commands::queue::QueueCommand;
use dotenv::dotenv;
use rand::seq::SliceRandom;
use serenity::builder::{
    CreateChannel, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::futures::future::join_all;
use serenity::http::Http;
use serenity::model::{guild, prelude::*};
use serenity::{async_trait, prelude::*};
use settings::Settings;

struct Handler {
    settings: Settings,
    queue_command: QueueCommand,
}

enum CollectorEvent {
    ReactionAdd(Reaction),
    ReactionRemove(Reaction),
}

impl Handler {
    fn new() -> Self {
        let settings = Settings::deserialize().expect("Should match the config format");
        let queue_command = QueueCommand::new(settings.clone());
        Handler {
            settings,
            queue_command,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "queue" => {
                    self.queue_command.run(&ctx, &command).await.unwrap();
                    None
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                };
            };
        };
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if let Some(channel_id) = new.channel_id {
            if let Ok(channel) = ctx.http.get_channel(channel_id).await {
                if let Some(channel) = channel.guild() {
                    if let Some(parent_id) = channel.parent_id {
                        if let Some(category_settings) =
                            self.settings.voice_expander.get(&parent_id)
                        {
                            let voice_channels: Vec<GuildChannel> = ctx
                                .http
                                .get_channels(channel.guild_id)
                                .await
                                .unwrap()
                                .iter()
                                .filter_map(|ch| {
                                    if ch.parent_id == Some(parent_id)
                                        && ch.kind == ChannelType::Voice
                                    {
                                        Some(ch.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if voice_channels.len() < category_settings.max_channels {
                                let current_names: Vec<String> =
                                    voice_channels.iter().map(|ch| ch.name.clone()).collect();
                                let name_options: Vec<String> = category_settings
                                    .channel_names
                                    .iter()
                                    .filter(|chn| !current_names.contains(chn))
                                    .map(|name| name.clone())
                                    .collect();
                                if let Ok(guild) = ctx.http.get_guild(channel.guild_id).await {
                                    let number_of_empty_channels = voice_channels
                                        .iter()
                                        .filter(|ch| ch.members(&ctx).unwrap_or(vec![]).len() == 0)
                                        .count();
                                    if number_of_empty_channels == 0 {
                                        let channel_name = name_options
                                            .choose(&mut rand::thread_rng())
                                            .unwrap_or(&"ERROR".to_string())
                                            .clone();
                                        guild
                                            .create_channel(
                                                &ctx,
                                                CreateChannel::new(channel_name)
                                                    .kind(ChannelType::Voice)
                                                    .category(parent_id)
                                                    .audit_log_reason(
                                                        "Create a New Empty Voice Channel",
                                                    ),
                                            )
                                            .await
                                            .unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(old) = old {
            if let Some(guild_id) = old.guild_id {
                let voice_channels: Vec<GuildChannel> = ctx
                    .http
                    .get_channels(guild_id)
                    .await
                    .unwrap()
                    .iter()
                    .filter_map(|ch| {
                        if ch.kind == ChannelType::Voice {
                            Some(ch.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut channels_to_delete = vec![];
                self.settings.voice_expander.keys().for_each(|category_id| {
                    let mut voice_channel_deletes: Vec<_> = voice_channels
                        .iter()
                        .rev()
                        .skip(1)
                        .filter_map(|vch| {
                            let members = vch.members(&ctx).unwrap_or(vec![]);
                            if vch.parent_id == Some(*category_id) && members.len() == 0 {
                                Some(vch.delete(&ctx))
                            } else {
                                None
                            }
                        })
                        .collect();
                    channels_to_delete.append(&mut voice_channel_deletes);
                });
                join_all(channels_to_delete).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("{} is Connected!", self.settings.general.name);

        if self.settings.flags.deadlock_queue_start {
            Command::create_global_command(&ctx.http, self.queue_command.register())
                .await
                .expect("Failed to Register Command");
        }

        reaction_roles::ReactionRole::register(&ctx, &self.settings).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's id.
    let _bot_id = match http.get_current_user().await {
        Ok(info) => info.id,
        Err(why) => panic!("Could not access user info: {:?}", why),
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::AUTO_MODERATION_CONFIGURATION;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::new())
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
