pub use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// prettify data output
    #[arg(short, long)]
    pub pretty: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// send compositor events in a continuous stream
    Listen,
    /// perform operations on input devices
    Input {
        #[command(subcommand)]
        command: Option<InputCommands>,
    },
    /// perform operations on output devices
    Output {
        #[command(subcommand)]
        command: Option<OutputCommands>,
    },
    /// perform operations on windows
    Windows,
    /// perform operations on worskpaces
    Workspaces,
}

#[derive(Subcommand)]
pub enum InputCommands {
    /// keyboard info
    Keyboard,
}

#[derive(Subcommand)]
pub enum OutputCommands {
    /// perform operations on physical outputs
    Outputs {
        #[command(subcommand)]
        command: Option<OutputsCommands>,
    }
}

#[derive(Subcommand)]
pub enum OutputsCommands {
    /// currently focused output
    FocusedOutput,
}
