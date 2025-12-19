use std::io::Read;

use crate::entry::Entry;
use clap::Parser;

mod entry;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of times to greet
    #[arg(short, long, default_value = "rustotp.txt")]
    data: String,
}

fn main() {
    let args = Args::parse();

    if std::path::Path::new(&args.data).exists() == false {
        std::fs::File::create(&args.data).expect("Cannot create --data file");
    }
    let mut data_file = std::fs::File::open(args.data).expect(("Cannot open --data file"));
    let mut content: String = String::new();
    data_file
        .read_to_string(&mut content)
        .expect("Cannot read content of --data file");

    let mut entries: Vec<Entry> = Vec::new();

    for line in content.lines() {
        let entry = Entry::parse(line);
        entries.push(entry);
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries.iter().for_each(|e| {
        println!("{}: {}", e.name, e.current_code());
    });
}
