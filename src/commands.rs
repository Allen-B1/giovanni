//! Command framework

use core::future::Future;
use std::{any::{self, Any, TypeId}, sync::{Mutex, Arc, RwLock}, collections::HashMap, ops::Deref, borrow::Cow, pin::Pin};

use serenity::{client::Context, model::{prelude::Ready, id::CommandId, interactions::{application_command::{ApplicationCommand, ApplicationCommandOptionType, ApplicationCommandInteraction}, Interaction}}};

#[derive(Clone)]
pub struct Arg {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub type_: any::TypeId,
    pub required: bool
}

#[derive(Clone)]
pub struct Command {
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub args: Vec<Arg>,
    pub handler: fn(&Context, &ApplicationCommandInteraction) -> Pin<Box<dyn Future<Output=crate::Result<()>> + Send>>
}

pub struct Commands {
    // Used before ready is called
    // DOES NOT CHANGE. EVER.
    commands: Vec<Command>,

    // Filled after ready is called
    commands_map: RwLock<HashMap<CommandId, usize>>,

    on_error: Option<fn(&Context, &ApplicationCommandInteraction, Box<dyn std::error::Error>) -> Pin<Box<dyn Future<Output=()> + Send>>>,
}

fn typeid_to_optiontype(typeid: TypeId) -> ApplicationCommandOptionType {
    if typeid == TypeId::of::<u64>() ||
        typeid == TypeId::of::<u32>() ||
        typeid == TypeId::of::<u16>() ||
        typeid == TypeId::of::<u8>() ||
        typeid == TypeId::of::<u128>() ||
        typeid == TypeId::of::<i64>() ||
        typeid == TypeId::of::<i32>() ||
        typeid == TypeId::of::<i16>() ||
        typeid == TypeId::of::<i8>() ||
        typeid == TypeId::of::<i128>() ||

        typeid == TypeId::of::<usize>() ||
        typeid == TypeId::of::<isize>() {
            ApplicationCommandOptionType::Integer
    } else if typeid == TypeId::of::<f32>() ||
        typeid == TypeId::of::<f64>()  {
            ApplicationCommandOptionType::Number
    } else if typeid == TypeId::of::<bool>() {
            ApplicationCommandOptionType::Boolean
    } else if typeid == TypeId::of::<serenity::model::user::User>() || typeid == TypeId::of::<serenity::model::guild::Member>() {
            ApplicationCommandOptionType::User
    } else if typeid == TypeId::of::<serenity::model::guild::Role>() {
            ApplicationCommandOptionType::Role
    } else {
            ApplicationCommandOptionType::String
    }
}

impl Commands {
    pub fn new(commands: Vec<Command>) -> Commands {
        Commands {
            commands: commands,
            commands_map: Default::default(),
            on_error: None
        }
    }

    pub fn on_error(&mut self, f: fn(&Context, &ApplicationCommandInteraction, Box<dyn std::error::Error>) -> Pin<Box<dyn Future<Output=()> + Send>>) {
        self.on_error = Some(f);
    }
}

#[serenity::async_trait]
impl serenity::client::EventHandler for Commands {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let mut futures_ = Vec::new();
        for command in self.commands.iter() {
            let name = command.name.clone();
            let desc = command.description.clone();
            let args = command.args.clone();
            futures_.push(ApplicationCommand::create_global_application_command(&ctx.http, |cmd| {
                cmd.name(name).description(desc);
                for arg in args {
                    cmd.create_option(|opt| {
                        opt.name(&arg.name)
                            .description(&arg.description)
                            .required(arg.required)
                            .kind(typeid_to_optiontype(arg.type_))
                    });
                }
                cmd
            }));
        }

        let out = futures::future::join_all(futures_).await.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();
        let mut map = self.commands_map.write().unwrap();
        for (idx, item) in out.iter().enumerate() {
            map.insert(item.id, idx);
            eprintln!("registered command: {} {}", item.name, item.id);
        }
    }


    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(cmd) => {
                let mut handler;
                {
                    let map = self.commands_map.read().unwrap();
                    handler = self.commands.get(*map.get(&cmd.data.id).unwrap_or(&10000));
                }
                if let Some(handler) = handler {
                    let future = match (handler.handler)(&ctx, &cmd).await {
                        Err(e) => {
                            if let Some(f) = self.on_error {
                                Some(f(&ctx, &cmd, e))
                            } else {
                                None
                            }
                        },
                        _ => None
                    };

                    if let Some(future) = future {
                        future.await;
                    }
                }
            },
            _ => {}
        }
    }
}