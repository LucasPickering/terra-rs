use anyhow::Context;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process,
};
use structopt::StructOpt;
use terra::{World, WorldConfig};

/// CLI for generating worlds via the Terra generation kit.
#[derive(Debug, StructOpt)]
#[structopt(name = "terra")]
struct Opt {
    /// Path to a JSON config that defines the world to be generated
    #[structopt(short, long)]
    config: PathBuf,

    /// If given, the generated world will be serialized and saved to this
    /// file. Typically has a .bin extension.
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// The logging level to use during generation. See https://docs.rs/log/0.4.11/log/enum.LevelFilter.html for options.
    #[structopt(short, long, default_value = "info")]
    log_level: LevelFilter,
}

fn run(opt: Opt) -> anyhow::Result<()> {
    SimpleLogger::new().with_level(opt.log_level).init()?;

    // Load config
    let cfg_string = fs::read_to_string(&opt.config).with_context(|| {
        format!("Error reading config file from {:?}", &opt.config)
    })?;
    let config: WorldConfig = serde_json::from_str(&cfg_string)
        .with_context(|| "Error deserializing config")?;

    let world = World::generate(config);

    if let Some(output) = opt.output {
        let world_bytes = rmp_serde::to_vec(&world)
            .with_context(|| "Error serializing world")?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&output)
            .with_context(|| {
                format!("Error opening output file {:?}", &output)
            })?;
        file.write_all(&world_bytes).with_context(|| {
            format!("Error writing world to file {:?}", &output)
        })?;
    }

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
