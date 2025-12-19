use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Number of times to greet
    #[arg(short, long, default_value = "rustotp.txt", help = "Path to data file")]
    pub data: String,
}
