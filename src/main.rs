#[macro_use]
extern crate lazy_static;

extern crate serenity;

mod commands;
mod database;
mod palette;

use std::{env, any::TypeId, sync::Mutex, ops::Deref, pin::Pin, f32::consts::E};
use async_once::AsyncOnce;
use futures::Future;
use serde::{Deserialize, Serialize};
use serenity::{model::{guild::Member, interactions::application_command::ApplicationCommandInteraction, id::UserId}, client::Context, builder::CreateEmbed, prelude::RwLock};
use sqlx::sqlite::SqliteConnectOptions;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/*
struct Handler;

use serenity::{model::interactions::{application_command, Interaction}, client::Context};

#[serenity::async_trait]
impl serenity::client::EventHandler for Handler {
    async fn ready(&self, ctx: serenity::client::Context, ready: serenity::model::gateway::Ready) {
        println!("I am {}", ready.user.name);

        let commands = application_command::ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|cmd| {
                cmd.name("profile").description("profile infogtg").create_option(|opt| {
                    opt.name("username").description("username").kind(application_command::ApplicationCommandOptionType::String).required(true)  
                })
            })
        })
        .await
        .unwrap();

        println!("{:?}", &commands);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(cmd) => {
                if cmd.data.name == "profile" {
                    cmd.create_interaction_response(&ctx.http, |resp| {
                        resp.interaction_response_data(|data| {
                            data.content("TODO!")
                        })
                    }).await.unwrap();
                }
            },
            _ => {}
        }
    }
}*/

lazy_static! {
    static ref DB: AsyncOnce<database::Database> = AsyncOnce::new(async {
        database::Database::new(
            sqlx::sqlite::SqlitePoolOptions::new().connect_with(SqliteConnectOptions::new().filename("sqlite:data.db").create_if_missing(true)).await.unwrap())
            .await
            .unwrap()
    });
}

async fn create_user_embed(username: &str, discord: Option<UserId>) -> Result<CreateEmbed> {
    #[derive(Serialize, Deserialize)]
    struct Stars {
        ffa: Option<String>,
        #[serde(rename = "duel")]
        m1v1: Option<String>,
        #[serde(rename = "2v2")]
        m2v2: Option<String>,

        #[serde(rename = "ffa-alltime")]
        ffa_alltime: Option<String>,
        #[serde(rename = "duel-alltime")]
        m1v1_alltime: Option<String>,
        #[serde(rename = "2v2-alltime")]
        m2v2_alltime: Option<String>,
    }

    #[derive(Serialize, Deserialize)]
    struct StarsAndRanks {
        stars: Stars,
    }

    let resp: StarsAndRanks = reqwest::get(&format!("https://generals.io/api/starsAndRanks?u={}", urlencoding::encode(&username)))
        .await?
        .json()
        .await?;

    let mut iq: f64 = resp.stars.m1v1_alltime.unwrap_or("0.0".to_string()).parse()?;
    iq = (iq - 65.0) / 8.0;
    iq = iq * 15.0 + 100.0;
    let mut iq = iq as i64;
    iq += username.len() as i64 - 5;
    iq += 2 * username.chars().filter(|&x| x == 'a' || x == 'A').collect::<Vec<_>>().len() as i64;
    iq -= 2 * username.chars().filter(|&x| x == 'c' || x == 'C').collect::<Vec<_>>().len() as i64;
    iq -= 3 * username.chars().filter(|&x| x == 'd' || x == 'D').collect::<Vec<_>>().len() as i64;
    iq -= 5 * username.chars().filter(|&x| x == 'f' || x == 'F').collect::<Vec<_>>().len() as i64;
    if iq > 160 { iq = 160; }
    if iq < 0 { iq = 0; }

    let mut embed = CreateEmbed::default();
    embed.title(format!("Profile: {}", &username));
    embed.description(
        format!(concat!(
            "{}",
            "**FFA Stars**: {}\n",
            "**1v1 Stars**: {}\n",
            "**Estimated IQ**: {}"
        ), 
            if let Some(discord) = discord { format!("**Discord**: <@{}>\n", discord.0) } else { "".to_string() },
            resp.stars.ffa.map(|x| x.parse::<f64>()).transpose()?.map(|x| format!("{:.2}", x)).unwrap_or("---".to_string()),
            resp.stars.m1v1.map(|x| x.parse::<f64>()).transpose()?.map(|x| format!("{:.2}", x)).unwrap_or("---".to_string()),
            iq)
    );
    embed.color(palette::EMBED_GAME);
    Ok(embed)
}

fn handle_user(ctx: &Context, i: &ApplicationCommandInteraction) -> Pin<Box<dyn Future<Output=Result<()>> + Send>> {
    let i = i.clone();
    let ctx = ctx.clone();

    Box::pin(async move {
        let mut user = i.user.id;

        for opt in &i.data.options {
            if opt.name == "mention" && opt.value.is_some() {
                user = UserId(opt.value.as_ref().unwrap().as_str().unwrap().parse()?);
            }
        }

        let db = DB.get().await;

        let username = db.get_username(user.0).await?;
        if let Some(username) = username {
            let embed = create_user_embed(&username, Some(user)).await?;
            embeds::respond(&ctx, &i, embed).await?;
        } else {
            embeds::respond(&ctx, &i, embeds::error(Option::<String>::None, "Discord user not registered")).await?;
        }

        Ok(())
    })
}

fn handle_profile(ctx: &Context, i: &ApplicationCommandInteraction) -> Pin<Box<dyn Future<Output=Result<()>> + Send>> { 
    let ctx = ctx.clone();
    let i = i.clone();

    let mut username = String::new();
    for opt in &i.data.options {
        if opt.name == "username" && opt.value.is_some() {
            username.push_str(opt.value.as_ref().unwrap().as_str().unwrap())
        }
    }

    Box::pin(async move {
        // make sure account exists
        let resp = reqwest::Client::new().get(&format!("https://generals.io/api/validateUsername?u={}", urlencoding::encode(&username)))
            .send()
            .await?;
        let resp = resp.json::<serde_json::Value>().await?;
        if resp.as_bool() != Some(true) {
            embeds::respond(&ctx, &i, embeds::error(Option::<String>::None, "generals.io username does not exist")).await?;
            return Ok(())
        }

        eprintln!("ok");

        // get discord
        let db = DB.get().await;
        let disc = db.get_discord(&username).await?.map(UserId);

        let e = create_user_embed(&username, disc).await?;
        embeds::respond(&ctx, &i, e).await?;  

        Ok(())
    })
}

mod embeds {
    use std::borrow::Cow;
    use crate::palette;

    use serenity::{builder::CreateEmbed, client::Context, model::interactions::application_command::ApplicationCommandInteraction};

    pub fn error(title: Option<impl Into<Cow<'static, str>>>, desc: impl Into<Cow<'static, str>>) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.title(title.map(|e| e.into()).unwrap_or_else(|| "Error".into()))
            .description(desc.into())
            .color(palette::EMBED_ERROR);
        embed
    }

    pub fn respond<'a>(ctx: &'a Context, i: &'a ApplicationCommandInteraction, e: CreateEmbed) -> impl futures::Future<Output = Result<(), impl std::error::Error>> + 'a {
        i.create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|data| {
                data.add_embed(e)
            })
        })
    }
}

fn handle_register(ctx: &Context, i: &ApplicationCommandInteraction) -> Pin<Box<dyn Future<Output=Result<()>> + Send>> {
    let (i, ctx) = (i.clone(), ctx.clone());
    let mut user = String::new();
    for opt in &i.data.options {
        if opt.name == "username" {
            user.push_str(opt.value.as_ref().unwrap_or(&serde_json::Value::Null).as_str().unwrap());
        }
    }
    let discord = i.user.id;

    Box::pin(async move {
        // check 1: username
        if !user.to_lowercase().starts_with("[b-tier]") {
            embeds::respond(&ctx, &i, embeds::error(Some("Register Error"), "generals.io username does not begin with [B-tier]")).await?;
            return Ok(())
        }

        // check 2: validate username
        let resp = reqwest::Client::new().get(&format!("https://generals.io/api/validateUsername?u={}", urlencoding::encode(&user)))
            .send()
            .await?;
        let resp = resp.json::<serde_json::Value>().await?;
        if resp.as_bool() != Some(true) {
            embeds::respond(&ctx, &i, embeds::error(Some("Register Error"), "generals.io username does not exist")).await?;
            return Ok(())
        }

        // check 3: check replays
        let resp = reqwest::Client::new().get(&format!("https://generals.io/api/replaysForUsername?u={}&offset=0&count=1", urlencoding::encode(&user)))
            .send()
            .await?;
        let resp = resp.json::<serde_json::Value>().await?;
        if resp.as_array().map(|x| x.len()) != Some(1) {
            embeds::respond(&ctx, &i, embeds::error(Some("Register Error"), "generals.io username does not have games")).await?;
            return Ok(())
        }

        // check 4: make sure neither exist in DB
        let db = DB.get().await;
        let has_discord = db.get_username(*discord.as_u64()).await?.is_some();
        let has_username = db.get_discord(&user).await?.is_some();

        if has_discord || has_username {
                embeds::respond(&ctx, &i, embeds::error(Some("Register Error"), 
                    "generals.io username or discord user already registered")).await?;
                    return Ok(())
        }

        db.add_username(*discord.as_u64(), &user).await?;

        i.create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|data| {
                data.create_embed(|embed| {
                    embed.title("Registered")
                        .description(&format!("Username: {}\nDiscord: <@{}>", user, discord.as_u64()))
                        .color(palette::EMBED_GAME)
                })
            })
        }).await?;

        Ok(())
    })
}

lazy_static!{
    static ref COMMAND_USER: commands::Command = commands::Command {
        name: "user".into(),
        description: "shows generals.io profile for discord user".into(),
        args: vec![
            commands::Arg { name: "mention".into(), description: "the discord user".into(), required: false, type_: TypeId::of::<Member>() }
        ],
        handler: handle_user,
    };
    static ref COMMAND_REGISTER: commands::Command = commands::Command {
        name: "register".into(),
        description: "registers generals.io username to discord user".into(),
        args: vec![
            commands::Arg { name: "username".into(), description: "generals.io username".into(), required: true, type_: TypeId::of::<String>() }
        ],
        handler: handle_register
    };
    static ref COMMAND_PROFILE: commands::Command = commands::Command {
        name: "profile".into(),
        description: "shows profile of generals.io user".into(),
        args: vec![
            commands::Arg { name: "username".into(), description: "generals.io username".into(), required: true, type_: TypeId::of::<String>() }
        ],
        handler: handle_profile
    };
}

fn on_error(ctx: &Context, i: &ApplicationCommandInteraction, error: Box<dyn std::error::Error>) -> Pin<Box<dyn Future<Output=()> + Send>> {
    let ctx = ctx.clone();
    let i =i.clone();
    let str = format!("{}\n\n```rust\n{:?}\n```", error, error);
    Box::pin(async move {
        embeds::respond(&ctx, &i, embeds::error(Some("Internal Error"), str)).await;
    })
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_GIO_TOKEN").expect("Expected a token in the environment");
    let application_id = env::var("DISCORD_GIO_APPID")
        .expect("Expected $DISCORD_GIO_APPID in the environment")
        .parse::<u64>()
        .expect("$DISCORD_GIO_APPID must be an unsigned integer");

    DB.get().await;

    let mut commands = commands::Commands::new(vec![COMMAND_PROFILE.clone(), COMMAND_REGISTER.clone(), COMMAND_USER.clone()]);
    commands.on_error(on_error);

    let mut client = serenity::Client::builder(&token).event_handler(commands).application_id(application_id).await.expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}