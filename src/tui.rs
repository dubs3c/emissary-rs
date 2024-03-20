use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Emissary-rs")]
#[command(version = "1.0")]
#[command(version, about = "Send data through chat channels. Made by @dubs3c.", long_about = None)]
pub struct Args {
    /// Message to send
    #[arg(short, long)]
    pub msg: Option<String>,

    /// Specify the field that contains the message. Default is 'message'
    #[arg(short, long)]
    pub txt: Option<String>,
}
