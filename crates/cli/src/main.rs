use anyhow::{anyhow, bail, Context};
use config::{Config, File};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
};
use structopt::StructOpt;
use strum::{Display, EnumString};
use terra::{timed, RenderConfig, TileLens, World, WorldConfig, WorldRenderer};

/// CLI for generating worlds via the Terra generation kit.
#[derive(Debug, StructOpt)]
#[structopt(name = "terra")]
struct Opt {
    /// Path to a config file that defines the world to be generated. Supported
    /// formats: JSON, TOML
    #[structopt(short, long)]
    config: Option<PathBuf>,

    /// Path to an existing .bin world file to load
    #[structopt(short, long)]
    bin: Option<PathBuf>,

    /// If given, the generated world will be saved to this directory. The
    /// exact files that appear in the directory are defined by the output
    /// formats. See `--output-formats` for more info
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// The format(s) to output the world in. Supported formats:
    ///
    /// bin - Binary representation that can be reloaded by this CLI and
    ///   other tools later. Use this for persisting & sharing worlds
    ///
    /// cfg - The full config object used for the world, in TOML format
    ///
    /// json - JSON representation. Similar to the binary format, but slower
    ///   and much less compact
    ///
    /// svg - 2D rendering of the world
    ///
    /// stl - 3D rendering of the world
    // TODO include link to docs about binary format after core crate is pub
    #[structopt(short = "f", long)]
    output_formats: Vec<OutputFormat>,

    /// The lens used to determine the color of each tile. Only relevant for
    /// rendered output formats, such as SVG.
    // TODO include link to TileLens docs here after core crate is published
    #[structopt(long, default_value = "surface")]
    lens: TileLens,

    /// Hide geographic features such as rivers, lakes, etc? Only relevant for
    /// rendered output formats, such as SVG.
    #[structopt(long)]
    hide_features: bool,

    /// The logging level to use during world generation. See
    /// https://docs.rs/log/0.4.11/log/enum.LevelFilter.html for options
    #[structopt(long, default_value = "info")]
    log_level: LevelFilter,
}

/// Different output formats.
#[derive(Copy, Clone, Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
enum OutputFormat {
    // If you change this, make sure to update the help text for
    // `--output-formats`!
    /// Export the world in a serialized binary format, which can be
    /// deserialized later to recover the world
    Bin,
    /// Export the world's full config in a human-readable file
    Cfg,
    /// Export the world in a serialized JSON format, which can be deserialized
    /// later to recover the world. This is similar to the bin format, but is
    /// human readable at the cost of being slower and much less compact
    Json,
    /// Render the world as a 2D SVG
    Svg,
    /// Render the world as a 3D STL
    Stl,
    /* If you change this, make sure to update the help text for
     * `--output-formats`! */
}

impl OutputFormat {
    fn file_ext(self) -> &'static str {
        match self {
            Self::Bin => "bin",
            Self::Cfg => "toml",
            Self::Json => "json",
            Self::Svg => "svg",
            Self::Stl => "stl",
        }
    }
}

/// Options to configure rendered output formats.
#[derive(Copy, Clone, Debug)]
pub struct RenderOptions {
    lens: TileLens,
    show_features: bool,
}

fn load_config(config_path: &Path) -> anyhow::Result<WorldConfig> {
    // Load config
    let mut settings = Config::new();
    let config_path = config_path.to_str().ok_or_else(|| {
        anyhow!("invalid character in path {:?}", config_path)
    })?;
    settings
        .merge(File::with_name(config_path))
        .context("error reading config file")?;
    settings.try_into().context("error reading config")
}

/// Generate an output form of the world in the given format.
fn gen_output(
    output_dir: &Path,
    output_format: OutputFormat,
    world: &World,
    renderer: &WorldRenderer,
) -> anyhow::Result<()> {
    fn generate_bytes(
        output_format: OutputFormat,
        world: &World,
        renderer: &WorldRenderer,
    ) -> Vec<u8> {
        match output_format {
            OutputFormat::Bin => {
                // Serialize the entire world via msgpack
                world.to_bin()
            }
            OutputFormat::Cfg => {
                // Serialize just the world config via toml
                toml::to_string_pretty(world.config())
                    // Panics only if config format isn't serializable (a bug)
                    .expect("error serializing config")
                    .into_bytes()
            }
            OutputFormat::Json => {
                // Serialize the entire world via JSON
                world.to_json().into()
            }
            OutputFormat::Svg => {
                // Render the world in 2D
                renderer.render_as_svg(world).into_bytes()
            }
            OutputFormat::Stl => {
                // Render the world in 3D
                renderer.render_as_stl(world)
            }
        }
    }

    let output_file_path = output_dir
        .join("world")
        .with_extension(output_format.file_ext());

    timed!(
        format!(
            "Generating {} output and writing to {:?}",
            output_format, &output_file_path
        ),
        log::Level::Info,
        {
            let bytes = generate_bytes(output_format, world, renderer);
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output_file_path)
                .with_context(|| {
                    format!("error opening output file {:?}", &output_file_path)
                })?;
            file.write_all(&bytes).with_context(|| {
                format!("error writing to file {:?}", &output_file_path)
            })?;
        }
    );

    Ok(())
}

/// Run the CLI with some options
fn run(opt: Opt) -> anyhow::Result<()> {
    SimpleLogger::new().with_level(opt.log_level).init()?;

    let world = match opt {
        Opt {
            config: Some(config_path),
            bin: None,
            ..
        } => {
            // Load world config and use it to generate a new world
            let config = load_config(&config_path)?;
            World::generate(config)?
        }
        Opt {
            config: None,
            bin: Some(input_path),
            ..
        } => {
            // Load existing world from a file
            let file = OpenOptions::new()
                .read(true)
                .open(&input_path)
                .with_context(|| {
                    format!("error opening world file {:?}", input_path)
                })?;
            let world = World::from_bin(file)?;
            info!("Loaded world from {:?}", &input_path);
            world
        }
        _ => bail!(
            "must pass exactly one of --config (to generate a new world) \
            or --input (to load an existing world)"
        ),
    };

    // If an output dir was specified, write out output format(s) there
    if let Some(output_dir) = opt.output {
        if opt.output_formats.is_empty() {
            bail!("output dir was specified, but no output formats were given")
        }
        fs::create_dir_all(&output_dir)?;

        let renderer = WorldRenderer::new(RenderConfig {
            tile_lens: opt.lens,
            show_features: !opt.hide_features,
            ..Default::default()
        })
        .context("invalid render config")?;
        for output_format in opt.output_formats {
            gen_output(&output_dir, output_format, &world, &renderer)?;
        }
    }

    Ok(())
}

fn main() {
    let exit_code = match run(Opt::from_args()) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error: {:#}", err);
            1
        }
    };
    process::exit(exit_code);
}
