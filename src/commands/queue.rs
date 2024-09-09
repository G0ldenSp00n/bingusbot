use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::settings::{ReactionRole, Settings};

pub struct QueueCommand {
    settings: Settings,
}

impl QueueCommand {
    pub async fn run(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<(), serenity::Error> {
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default()
                        .ephemeral(true)
                        .select_menu(
                            CreateSelectMenu::new(
                                "minute_wait",
                                CreateSelectMenuKind::String {
                                    options: vec![
                                        CreateSelectMenuOption::new("5 Minutes", "5"),
                                        CreateSelectMenuOption::new("10 Minutes", "10"),
                                        CreateSelectMenuOption::new("15 Minutes", "15"),
                                    ],
                                },
                            )
                            .placeholder("Queue Timer"),
                        ),
                ),
            )
            .await
            .unwrap();

        let interaction = match interaction
            .get_response(&ctx)
            .await
            .unwrap()
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(60 * 2))
            .await
        {
            Some(x) => x,
            None => {
                interaction.delete_response(&ctx).await.unwrap();
                return Ok(());
            }
        };

        let mut game_name_to_roles: HashMap<String, Vec<ReactionRole>> = HashMap::new();
        let minutes_to_wait = interaction.data.clone();
        self.settings.game_queue.iter().for_each(|game_queue| {
            let game_reaction_roles: Vec<ReactionRole> = self
                .settings
                .message_id_to_emoji_reaction_to_reactionrole_lookup()
                .get(&game_queue.roles_message_id.clone())
                .expect("Queue Roles must be Reaction Roles too!")
                .values()
                .clone()
                .filter(|reaction_role| !game_queue.exclude.contains(&reaction_role.title))
                .map(|reaction_role| reaction_role.clone())
                .collect();
            game_name_to_roles.insert(game_queue.game_name.clone(), game_reaction_roles);
        });

        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .ephemeral(true)
                        .select_menu(
                            CreateSelectMenu::new(
                                "ping_roles",
                                CreateSelectMenuKind::String {
                                    options: game_name_to_roles
                                        .get("Deadlock")
                                        .unwrap()
                                        .iter()
                                        .map(|reaction_role| {
                                            if let Some(emoji_id) = reaction_role.emoji_id {
                                                CreateSelectMenuOption::new(
                                                    reaction_role.title.clone(),
                                                    reaction_role.role_id.to_string(),
                                                )
                                                .emoji(ReactionType::Custom {
                                                    animated: false,
                                                    id: emoji_id,
                                                    name: None,
                                                })
                                            } else {
                                                CreateSelectMenuOption::new(
                                                    reaction_role.title.clone(),
                                                    reaction_role.role_id.to_string(),
                                                )
                                                .emoji(ReactionType::Unicode(
                                                    reaction_role.emoji_char.clone().unwrap(),
                                                ))
                                            }
                                        })
                                        .collect(),
                                },
                            )
                            .min_values(1)
                            .max_values(game_name_to_roles.get("Deadlock").unwrap().len() as u8)
                            .placeholder("Roles to Ping"),
                        ),
                ),
            )
            .await
            .unwrap();

        let interaction = match interaction
            .get_response(&ctx)
            .await
            .unwrap()
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(60 * 2))
            .await
        {
            Some(x) => x,
            None => {
                interaction.delete_response(&ctx).await.unwrap();
                return Ok(());
            }
        };

        let roles_to_at = interaction.data.clone();

        if let ComponentInteractionDataKind::StringSelect {
            values: minutes_to_wait_values,
        } = minutes_to_wait.kind
        {
            if let ComponentInteractionDataKind::StringSelect {
                values: roles_to_at_values,
            } = roles_to_at.kind
            {
                let minutes_to_wait_value: u64 =
                    (minutes_to_wait_values[0]).to_string().parse().unwrap();
                interaction.delete_response(&ctx).await.unwrap();

                let mut response = MessageBuilder::new();
                response = response
                    .push_line("## Queueing")
                    .push_line("### Looking to Queue With")
                    .clone();
                roles_to_at_values.iter().for_each(|role_id| {
                    let reaction_role = game_name_to_roles
                        .get("Deadlock")
                        .unwrap()
                        .iter()
                        .find(|reaction_role| &reaction_role.role_id.to_string() == role_id)
                        .unwrap();
                    if let Some(emoji_char) = reaction_role.emoji_char.clone() {
                        response = response
                            .push(emoji_char)
                            .mention(&RoleId::new(role_id.parse().unwrap()))
                            .push_line("")
                            .clone()
                    } else {
                        response = response
                            .mention(&RoleId::new(role_id.parse().unwrap()))
                            .push_line("")
                            .clone()
                    }
                });

                let start = SystemTime::now();
                let since_the_epoch = start
                    .duration_since(UNIX_EPOCH)
                    .expect("Time Went Backwards");
                interaction
                    .get_response(&ctx)
                    .await
                    .unwrap()
                    .channel_id
                    .send_message(
                        &ctx,
                        CreateMessage::new().content(
                            response
                                .push_line("")
                                .push(format!(
                                    "Deadlock Queueing <t:{}:R>",
                                    since_the_epoch.as_secs() + (minutes_to_wait_value * 60)
                                ))
                                .build(),
                        ),
                    )
                    .await
                    .unwrap();
            }
        }
        Ok(())
    }

    pub fn new(settings: Settings) -> QueueCommand {
        QueueCommand { settings }
    }

    pub fn register(&self) -> CreateCommand {
        CreateCommand::new("queue").description("Asks some details about you")
    }
}