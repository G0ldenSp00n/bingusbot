use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serenity::builder::*;
use serenity::futures::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::settings::{ReactionRole, Settings};

use tokio::time::sleep;

const APPROX_MATCH_LENGTH_MINS: u64 = 40;

pub struct QueueCommand {
    settings: Settings,
}

impl QueueCommand {
    pub async fn run(
        &self,
        ctx: &Context,
        queue_command: &CommandInteraction,
    ) -> Result<(), serenity::Error> {
        queue_command
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
                                        CreateSelectMenuOption::new(
                                            "30 Seconds",
                                            format!("{}", 0.5 * 60.0),
                                        ),
                                        CreateSelectMenuOption::new(
                                            "1 Minute",
                                            format!("{}", 1 * 60),
                                        ),
                                        CreateSelectMenuOption::new(
                                            "5 Minutes",
                                            format!("{}", 5 * 60),
                                        ),
                                        CreateSelectMenuOption::new(
                                            "10 Minutes",
                                            format!("{}", 10 * 60),
                                        ),
                                        CreateSelectMenuOption::new(
                                            "15 Minutes",
                                            format!("{}", 15 * 60),
                                        ),
                                    ],
                                },
                            )
                            .placeholder("Queue Timer"),
                        ),
                ),
            )
            .await
            .unwrap();

        let queue_time_select_menu_interaction = match queue_command
            .get_response(&ctx)
            .await
            .unwrap()
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(60 * 2))
            .await
        {
            Some(x) => x,
            None => {
                queue_command.delete_response(&ctx).await.unwrap();
                return Ok(());
            }
        };

        let mut game_name_to_roles: HashMap<String, Vec<ReactionRole>> = HashMap::new();
        let minutes_to_wait = queue_time_select_menu_interaction.data.clone();
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

        queue_time_select_menu_interaction
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

        let queue_roles_to_mention_select_menu_interaction =
            match queue_time_select_menu_interaction
                .get_response(&ctx)
                .await
                .unwrap()
                .await_component_interaction(&ctx.shard)
                .timeout(Duration::from_secs(60 * 2))
                .await
            {
                Some(x) => x,
                None => {
                    queue_time_select_menu_interaction
                        .delete_response(&ctx)
                        .await
                        .unwrap();
                    return Ok(());
                }
            };

        let roles_to_at = queue_roles_to_mention_select_menu_interaction.data.clone();

        if let ComponentInteractionDataKind::StringSelect {
            values: minutes_to_wait_values,
        } = minutes_to_wait.kind
        {
            if let ComponentInteractionDataKind::StringSelect {
                values: roles_to_at_values,
            } = roles_to_at.kind
            {
                let seconds_to_wait_value: u64 =
                    (minutes_to_wait_values[0]).to_string().parse().unwrap();
                queue_roles_to_mention_select_menu_interaction
                    .create_response(
                        &ctx,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .content("Message Sent!")
                                .components(vec![]),
                        ),
                    )
                    .await
                    .unwrap();

                let mut queueing_up_message = MessageBuilder::new();
                queueing_up_message
                    .push_line(format!(
                        "## {} is Queueing",
                        queue_roles_to_mention_select_menu_interaction
                            .user
                            .clone()
                            .global_name
                            .unwrap_or(
                                queue_roles_to_mention_select_menu_interaction
                                    .user
                                    .name
                                    .clone()
                            )
                    ))
                    .push_line("### Looking to Play with");
                roles_to_at_values.iter().for_each(|role_id| {
                    let reaction_role = game_name_to_roles
                        .get("Deadlock")
                        .unwrap()
                        .iter()
                        .find(|reaction_role| &reaction_role.role_id.to_string() == role_id)
                        .unwrap();
                    if let Some(emoji_char) = reaction_role.emoji_char.clone() {
                        queueing_up_message
                            .push(emoji_char)
                            .mention(&RoleId::new(role_id.parse().unwrap()))
                            .push_line("");
                    } else {
                        queueing_up_message
                            .mention(&RoleId::new(role_id.parse().unwrap()))
                            .push_line("");
                    }
                });

                let start = SystemTime::now();
                let since_the_epoch = start
                    .duration_since(UNIX_EPOCH)
                    .expect("Time Went Backwards");
                let channel_id = queue_roles_to_mention_select_menu_interaction
                    .get_response(&ctx)
                    .await
                    .unwrap()
                    .channel_id
                    .clone();

                let mut queue_countdown_message = channel_id
                    .send_message(
                        &ctx,
                        CreateMessage::new().content(
                            queueing_up_message
                                .clone()
                                .push_line("")
                                .push_line(format!(
                                    "Deadlock Queueing <t:{}:R>",
                                    since_the_epoch.as_secs() + (seconds_to_wait_value)
                                ))
                                .build(),
                        ),
                    )
                    .await
                    .unwrap();

                let mut approx_match = queueing_up_message.clone();
                approx_match
                    .push_line("")
                    .push_line(format!(
                        "Started Queueing <t:{}:R>",
                        since_the_epoch.as_secs() + (seconds_to_wait_value)
                    ))
                    .push_line(format!(
                        "Approx. Next Match <t:{}:R>",
                        since_the_epoch.as_secs()
                            + (seconds_to_wait_value)
                            + (APPROX_MATCH_LENGTH_MINS * 60),
                    ));

                queue_roles_to_mention_select_menu_interaction
                    .delete_response(&ctx)
                    .await
                    .unwrap();
                sleep(Duration::from_secs(seconds_to_wait_value)).await;
                queue_countdown_message
                    .edit(
                        &ctx,
                        EditMessage::new().content(approx_match.build()).button(
                            CreateButton::new("wait_for_me")
                                .label("Toggle Join Next Game")
                                .style(ButtonStyle::Success),
                        ),
                    )
                    .await
                    .unwrap();
                let mut join_next_game_button_stream = queue_countdown_message
                    .await_component_interactions(&ctx)
                    .timeout(Duration::from_secs(
                        (APPROX_MATCH_LENGTH_MINS * 60) - (seconds_to_wait_value),
                    ))
                    .stream();

                let mut users_waiting = vec![];
                while let Some(interaction) = join_next_game_button_stream.next().await {
                    interaction
                        .create_response(&ctx, CreateInteractionResponse::Acknowledge)
                        .await
                        .unwrap();
                    if !users_waiting.contains(&interaction.user.id) {
                        users_waiting.push(interaction.user.id);
                    } else {
                        let user_index = users_waiting
                            .iter()
                            .position(|user_id| *user_id == interaction.user.id)
                            .unwrap();
                        users_waiting.remove(user_index);
                    }
                    queue_countdown_message
                        .edit(
                            &ctx,
                            EditMessage::new().content(
                                approx_match.build()
                                    + QueueCommand::build_next_game_queue_list_message(
                                        &users_waiting,
                                    )
                                    .as_str(),
                            ),
                        )
                        .await
                        .unwrap();
                }

                queue_countdown_message
                    .edit(
                        &ctx,
                        EditMessage::new()
                            .content(
                                queueing_up_message
                                    .clone()
                                    .push_line(format!(
                                        "Started Queueing <t:{}:R>",
                                        since_the_epoch.as_secs() + (seconds_to_wait_value)
                                    ))
                                    .build(),
                            )
                            .components(vec![]),
                    )
                    .await
                    .unwrap();
            }
        }
        Ok(())
    }

    fn build_next_game_queue_list_message(users_waiting: &Vec<UserId>) -> String {
        let mut message = MessageBuilder::new();
        if users_waiting.len() > 0 {
            message.push_line("### Waiting For Next Game");
            users_waiting.iter().for_each(|user_id| {
                message.push_line("").mention(user_id);
            })
        }
        message.build()
    }

    pub fn new(settings: Settings) -> QueueCommand {
        QueueCommand { settings }
    }

    pub fn register(&self) -> CreateCommand {
        CreateCommand::new("queue").description("Asks some details about you")
    }
}
