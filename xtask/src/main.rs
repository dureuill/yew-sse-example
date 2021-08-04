use std::ops::Index;

use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use clap::crate_version;
use clap::Clap;
use color_eyre::eyre::Context;
use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::colors::xterm::UserGreen;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Help;

/// Run tasks in this workspace.
#[derive(Clap)]
#[clap(version=crate_version!())]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Distributes the frontend and builds the frontend
    Dist(Dist),
    /// Starts the server, building the backend and frontend as needed
    Run(Run),
}

#[derive(Clap)]
struct Dist {
    /// Build artifacts in release mode, with optimizations
    #[clap(long)]
    release: bool,
}

#[derive(Clap)]
struct Run {
    /// Build artifacts in release mode, with optimizations
    #[clap(long)]
    release: bool,
}

impl Run {
    fn to_dist(&self) -> Dist {
        Dist {
            release: self.release,
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opts: Opts = Opts::parse();

    match opts.cmd {
        SubCommand::Dist(config) => dist(config)?,
        SubCommand::Run(config) => run(config)?,
    }

    Ok(())
}

fn dist_path(metadata: Metadata, is_release: bool) -> Utf8PathBuf {
    metadata
        .target_directory
        .join("static/dist")
        .join(if is_release { "release" } else { "debug" })
}

fn dist(config: Dist) -> Result<()> {
    let cmd = cargo_metadata::MetadataCommand::new();
    let metadata = cmd.exec()?;
    let frontend = metadata
        .workspace_members
        .iter()
        .find(|pkg| metadata[pkg].name == "frontend")
        .wrap_err("Could not find package 'frontend'")?;
    let html_path = metadata
        .index(frontend)
        .manifest_path
        .with_file_name("index.html");
    let dist_path = dist_path(metadata, config.release);

    println!(
        "- Distributing frontend in {}",
        dist_path.bold().fg::<UserGreen>()
    );
    std::fs::create_dir_all(&dist_path).wrap_err("Could not write to the target directory")?;

    let trunk_version = duct::cmd!("trunk", "--version")
        .read()
        .wrap_err("Could not find `trunk`")
        .note("`trunk` is required for the build")
        .suggestion("Install `trunk` with `cargo install trunk`")?;
    println!("- Using {}", trunk_version.bold().fg::<UserGreen>());
    let release = if config.release {
        Some("--release")
    } else {
        None
    };
    let args = IntoIterator::into_iter(["build", "--dist", dist_path.as_str(), html_path.as_str()])
        .chain(release);
    duct::cmd("trunk", args).run()?;
    Ok(())
}

fn run(config: Run) -> Result<()> {
    dist(config.to_dist())?;

    let cmd = cargo_metadata::MetadataCommand::new();
    let metadata = cmd.exec()?;

    let dist_path = dist_path(metadata, config.release);

    let release = if config.release {
        Some("--release")
    } else {
        None
    };
    let args = IntoIterator::into_iter(["run", "-p", "backend"]).chain(release);

    duct::cmd("cargo", args)
        .env("ROCKET_DIST", dist_path)
        .run()
        .wrap_err("Could not run server")?;
    Ok(())
}
