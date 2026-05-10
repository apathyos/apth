mod cli;
mod event_stream;
mod globals;
mod inputs;
mod outputs;
mod seats;
mod types;
mod utils;
mod windows;
mod workspaces;

use clap::Parser;
use serde::Serialize;
use wayland_client::{Connection, globals::registry_queue_init};

use crate::{
    cli::{Commands, InputCommands, Output, OutputCommands},
    event_stream::{IPC_SOCKET_PATH, Socket, StreamContext},
    inputs::Inputs,
    outputs::Outputs,
    seats::Seats,
    utils::system::get_runtime_dir,
    windows::Windows,
    workspaces::Workspaces,
};

#[derive(Default, Serialize)]
pub struct App {
    #[serde(skip)]
    pub stream_context: StreamContext,
    #[serde(skip)]
    pub seats: Seats,
    pub input: Inputs,
    pub output: Outputs,
    pub windows: Windows,
    pub workspaces: Workspaces,
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();
    let conn = Connection::connect_to_env().unwrap();
    let mut app = App::new();
    app.stream_context = StreamContext::new(2048);

    let (globals, mut queue) = registry_queue_init::<App>(&conn).unwrap();
    let qh = queue.handle();

    queue.roundtrip(&mut app).unwrap();

    Inputs::bind_input_manager(&mut app, globals.registry().clone(), &globals, &qh);
    Outputs::bind_outputs_manager(&mut app, &globals, &qh);
    Workspaces::bind_workspace_manager(&mut app, &globals, &qh);
    Windows::bind_toplevel_manager(&mut app, &globals, &qh);

    let pretty = &args.pretty;

    if let Some(cmd) = &args.command {
        match cmd {
            Commands::Listen => {
                let runtime_dir = get_runtime_dir()?;
                let socket_path = runtime_dir + IPC_SOCKET_PATH;
                let stream_context = app.stream_context.clone();

                tokio::spawn(async move {
                    if let Err(e) = Socket::run(&socket_path, stream_context).await {
                        eprintln!("stream server failed: {e:#}");
                    }
                });

                loop {
                    queue.blocking_dispatch(&mut app).unwrap();
                    stream_flush!(app.stream_context);
                }
            }
            Commands::Windows => {
                queue.blocking_dispatch(&mut app).unwrap();
                println!("{}", Output::serialize(&app.windows.windows, pretty));
            }
            Commands::Workspaces => {
                queue.blocking_dispatch(&mut app).unwrap();
                println!("{}", Output::serialize(&app.workspaces.workspaces, pretty));
            }
            Commands::Output { command } => {
                queue.blocking_dispatch(&mut app).unwrap();

                if let Some(cmd) = command {
                    match cmd {
                        OutputCommands::Outputs { command } => match command {
                            _ => {
                                println!("{}", Output::serialize(&app.output.outputs, pretty));
                            }
                        },
                    }
                } else {
                    println!("{}", Output::serialize(&app.output, pretty));
                }
            }
            Commands::Input { command } => {
                queue.blocking_dispatch(&mut app).unwrap();

                if let Some(cmd) = command {
                    match cmd {
                        InputCommands::Keyboard => {
                            println!("{}", Output::serialize(&app.input.keyboard, pretty));
                        }
                    }
                } else {
                    println!("{}", Output::serialize(&app.input, pretty));
                }
            }
        }
    } else {
        queue.blocking_dispatch(&mut app).unwrap();
        println!("{}", Output::serialize(&app, pretty));
    }

    Ok(())
}
