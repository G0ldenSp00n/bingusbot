mod commands;
mod reaction_roles;
mod settings;

use std::env;

use commands::queue::QueueCommand;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::http::Http;
use serenity::model::prelude::*;
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
