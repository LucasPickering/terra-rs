use anyhow::Context;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{fs, path::PathBuf, process};
use structopt::StructOpt;
use terra::{World, WorldConfig};

/// TODO
#[derive(Debug, StructOpt)]
#[structopt(name = "terra")]
struct Opt {
    /// Path to a JSON config that defines the world to be generated
    #[structopt(short, long)]
    config: PathBuf,

    /// If given, the generated world will be serialized and saved to this
    /// file.
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// The logging level to use during generation. See https://docs.rs/log/0.4.11/log/enum.LevelFilter.html for options.
    #[structopt(short, long, default_value = "info")]
    log_level: LevelFilter,
}

fn read_file(path: &PathBuf) -> anyhow::Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Error reading file {:?}", path))
}

fn run(opt: Opt) -> anyhow::Result<()> {
    SimpleLogger::new().with_level(opt.log_level).init()?;
    let cfg_string = read_file(&opt.config)?;
    let config: WorldConfig = serde_json::from_str(&cfg_string)
        .with_context(|| "Error deserializing config")?;
    World::generate(config);
    Ok(())
}

fn main() {
    let exit_code = match run(Opt::from_args()) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{:#}", err);
            1
        }
    };
    process::exit(exit_code);
}
