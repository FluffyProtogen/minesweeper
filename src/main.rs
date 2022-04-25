use dashmap::DashMap;
use drawing::{DefaultMinesweeperDrawer, MinesweeperDrawer};
use game::{Game, GameDataKey, GameState};
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::Args;
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::http::AttachmentType;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use std::borrow::Cow;
use std::sync::Arc;

#[macro_use]
extern crate lazy_static;

mod data;
mod drawing;
mod game;
mod text;

#[group]
#[commands(startgame, dig, flag, unflag, help, resend, stopgame)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]

async fn main() {
    let config = data::load_configuration().unwrap_or_else(|e| {
        println!("Couldn't parse config.json: {}", e);
        panic!();
    });

    let running_games = Arc::new(DashMap::<u64, Game>::new());

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&config.token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    client
        .data
        .write()
        .await
        .insert::<GameDataKey>(Arc::clone(&running_games));

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn startgame(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let author = &msg.author;

    if author.bot {
        return Ok(());
    }

    if args.len() != 1 {
        msg.channel_id
            .say(
                &ctx.http,
                "Usage:\nstartgame easy\nstartgame medium\nstartgame hard",
            )
            .await
            .ok();
        return Ok(());
    }

    let args = args.message().to_ascii_lowercase();

    let game_settings = match args.as_str() {
        "easy" => (10, 8, 10),
        "medium" => (18, 14, 40),
        "hard" => (24, 20, 99),
        _ => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Usage:\nstartgame easy\nstartgame medium\nstartgame hard",
                )
                .await
                .ok();
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let game_data = data.get::<GameDataKey>().unwrap();

    if game_data.contains_key(&author.id.0) {
        msg.channel_id.say(&ctx.http, "You already have a running game!\nUse the command stopgame to end your current game if you would like to end it.\nUse the command resend if you would like to see your current progress.").await.ok();
        return Ok(());
    }

    let game = Game::new(game_settings.0, game_settings.1, game_settings.2);

    send_game_render(ctx, msg.channel_id, &game).await.ok();

    game_data.insert(author.id.0, game);

    Ok(())
}

#[command]
async fn dig(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let author = &msg.author;

    if author.bot {
        return Ok(());
    }

    let coordinates = process_coordinates(&args);

    if coordinates.is_none() {
        msg.channel_id.say(&ctx.http, "Usage: dig X Y").await.ok();
        return Ok(());
    }

    let coordinates = coordinates.unwrap();

    let data = ctx.data.read().await;
    let game_data_map = data.get::<GameDataKey>().unwrap();
    let game_data = game_data_map.get_mut(&author.id.0);

    if let Some(mut game) = game_data {
        if coordinates.0 == 0
            || coordinates.1 == 0
            || coordinates.0 > game.width
            || coordinates.1 > game.height
        {
            msg.channel_id
                .say(&ctx.http, "Coordinates out of bounds!")
                .await
                .ok();
            return Ok(());
        }

        game.dig((coordinates.0 - 1, coordinates.1 - 1));

        send_game_render(ctx, msg.channel_id, &game).await.ok();

        if game.state == GameState::Lost || game.state == GameState::Won {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.add_embed(|embed| {
                        let difference = game.last_move_time - game.time_started;
                        let minutes = difference.num_minutes();
                        let seconds = difference.num_seconds() - difference.num_minutes() * 60;
                        embed.title("Game Summary");
                        embed.description(format!(
                            "Game {} in {} minute{} and {} second{}",
                            if game.state == GameState::Won {
                                "won"
                            } else {
                                "lost"
                            },
                            minutes,
                            if minutes == 1 { "" } else { "s" },
                            seconds,
                            if seconds == 1 { "" } else { "s" }
                        ));
                        embed.field(
                            "Grid Size",
                            format!("{} by {}", game.width, game.height),
                            true,
                        );
                        embed.field("Mine Count", format!("{}", game.number_of_mines), true);
                        embed
                    });
                    m
                })
                .await
                .unwrap();
            drop(game);
            game_data_map.remove(&msg.author.id.0);
        }
        Ok(())
    } else {
        msg.channel_id.say(
            &ctx.http,
            "You don't have any running games! Use the command startgame [difficulty] to start a game.",
        )
        .await
        .ok();
        return Ok(());
    }
}

#[command]
async fn flag(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let author = &msg.author;

    if author.bot {
        return Ok(());
    }

    let coordinates = process_coordinates(&args);

    if coordinates.is_none() {
        msg.channel_id.say(&ctx.http, "Usage: dig X Y").await.ok();
        return Ok(());
    }

    let coordinates = coordinates.unwrap();

    let data = ctx.data.read().await;
    let game_data = data.get::<GameDataKey>().unwrap();
    let game_data = game_data.get_mut(&author.id.0);

    if let Some(mut game) = game_data {
        if coordinates.0 == 0
            || coordinates.1 == 0
            || coordinates.0 > game.width
            || coordinates.1 > game.height
        {
            msg.channel_id
                .say(&ctx.http, "Coordinates out of bounds!")
                .await
                .ok();
            return Ok(());
        }

        game.flag((coordinates.0 - 1, coordinates.1 - 1));

        send_game_render(ctx, msg.channel_id, &game).await.ok();

        Ok(())
    } else {
        msg.channel_id.say(
            &ctx.http,
            "You don't have any running games! Use the command startgame [difficulty] to start a game.",
        )
        .await
        .ok();
        return Ok(());
    }
}

#[command]
async fn unflag(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let author = &msg.author;

    if author.bot {
        return Ok(());
    }

    let coordinates = process_coordinates(&args);

    if coordinates.is_none() {
        msg.channel_id.say(&ctx.http, "Usage: dig X Y").await.ok();
        return Ok(());
    }

    let coordinates = coordinates.unwrap();

    let data = ctx.data.read().await;
    let game_data = data.get::<GameDataKey>().unwrap();
    let game_data = game_data.get_mut(&author.id.0);

    if let Some(mut game) = game_data {
        if coordinates.0 == 0
            || coordinates.1 == 0
            || coordinates.0 > game.width
            || coordinates.1 > game.height
        {
            msg.channel_id
                .say(&ctx.http, "Coordinates out of bounds!")
                .await
                .ok();
            return Ok(());
        }

        game.unflag((coordinates.0 - 1, coordinates.1 - 1));

        send_game_render(ctx, msg.channel_id, &game).await.ok();

        Ok(())
    } else {
        msg.channel_id.say(
            &ctx.http,
            "You don't have any running games! Use the command startgame [difficulty] to start a game.",
        )
        .await
        .ok();
        return Ok(());
    }
}

#[command]
async fn stopgame(ctx: &Context, msg: &Message) -> CommandResult {
    let author = &msg.author;

    if author.bot {
        return Ok(());
    }

    let data = ctx.data.read().await;
    let game_data = data.get::<GameDataKey>().unwrap();

    if game_data.get_mut(&author.id.0).is_some() {
        game_data.remove(&author.id.0);

        msg.channel_id
            .say(&ctx.http, "Successfuly ended game.")
            .await
            .ok();

        return Ok(());
    } else {
        msg.channel_id.say(
            &ctx.http,
            "You don't have any running games! Use the command startgame [difficulty] to start a game.",
        )
        .await
        .ok();
        return Ok(());
    }
}

#[command]
async fn resend(ctx: &Context, msg: &Message) -> CommandResult {
    let author = &msg.author;

    if author.bot {
        return Ok(());
    }

    let data = ctx.data.read().await;
    let game_data = data.get::<GameDataKey>().unwrap();
    let game_data = game_data.get_mut(&author.id.0);

    if let Some(game) = game_data {
        send_game_render(ctx, msg.channel_id, &game).await.ok();
        return Ok(());
    } else {
        msg.channel_id.say(
            &ctx.http,
            "You don't have any running games! Use the command startgame [difficulty] to start a game.",
        )
        .await
        .ok();
        return Ok(());
    }
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            &ctx.http,
            "Commands: startgame, stopgame, dig, flag, unflag, help, resend",
        )
        .await
        .ok();
    Ok(())
}

fn process_coordinates(args: &Args) -> Option<(u32, u32)> {
    let mut args = args.message().split(" ");

    let mut positions = vec![];

    for _ in 0..2 {
        if let Some(position) = args.next() {
            if let Ok(position) = position.parse::<u32>() {
                positions.push(position);
            }
        }
    }

    if let Some(_) = args.next() {
        return None;
    }

    if positions.len() != 2 {
        None
    } else {
        Some((positions[0], positions[1]))
    }
}

async fn send_game_render(
    ctx: &Context,
    channel: ChannelId,
    game: &Game,
) -> Result<Message, serenity::Error> {
    let map = DefaultMinesweeperDrawer::draw_board(game);

    let attachment = AttachmentType::Bytes {
        data: Cow::Owned(map.encode_png().unwrap()),
        filename: "File.png".to_string(),
    };

    channel
        .send_message(&ctx.http, |m| m.add_file(attachment))
        .await
}
