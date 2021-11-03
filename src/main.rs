pub mod prelude;
use prelude::*;

use crate::commands::palette;

// Do NOT make this public
mod commands;

/// Display information about Giovanni
#[poise::command(slash_command, prefix_command)]
async fn info(ctx: Context<'_>) -> Result<(), Error> {
    poise::send_reply(ctx, |f| f
        .content("Rust > Java")
        .embed(|f| f
            .title("About Me")
            .description("I'm Giovanni, servant of all\\* generals.io players")
            .footer(|f| 
                f.text("\\*almost all")
            )
        ))
        .await?;

    Ok(())
}

#[tokio::main]
pub async fn main() {
    poise::Framework::build()
        .token(std::env::var("DISCORD_GIO_TOKEN").unwrap())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(()) }))
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                ..Default::default()
            },
            on_error: |err, ctx| Box::pin(error_handler(err, ctx)),
            ..Default::default()
        })
        .command(info(), |f| f.category("General"))
        .command(commands::game::custom(), |f| f.category("Game"))
        .run()
        .await
        .unwrap();
}

async fn error_handler(e: Error, ctx: poise::ErrorContext<'_, Data, Error>) -> () {
    match ctx {
        poise::ErrorContext::Setup | poise::ErrorContext::Listener(_) => {
            eprintln!("error in bot: {}", e);
        },
        poise::ErrorContext::Command(ctx) => {
            match ctx.ctx().send(|f| f
                    .embed(|f| f
                        .title("Error")
                        .colour(palette::EMBED_ERROR)
                        .description(format!("{}", e)))
                )
                .await {
                Err(e) => {
                    eprintln!("could not report error: {}", e);
                },
                _ => {}
            }
        }
    }
}