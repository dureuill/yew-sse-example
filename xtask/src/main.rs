use std::ops::Index;

use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use clap::crate_version;
use clap::Clap;
use color_eyre::eyre::Context;
use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::colors::xterm::UserGreen;
use color_eyre::owo_colors::colors::xterm::UserYellow;
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
    /// Install the frontend and backend to an output directory
    Install(Install),
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
#[derive(Clap)]
struct Install {
    /// Output directory for the installation
    #[clap(short, long)]
    output: Option<Utf8PathBuf>,
}

impl Install {
    fn to_dist(&self) -> Dist {
        Dist {
            release: true,
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let opts: Opts = Opts::parse();

    match opts.cmd {
        SubCommand::Dist(config) => dist(config)?,
        SubCommand::Run(config) => run(config)?,
        SubCommand::Install(config) => install(config)?,
    }

    Ok(())
}

fn dist_path(metadata: &Metadata, is_release: bool) -> Utf8PathBuf {
    metadata
        .target_directory
        .join("static/dist")
        .join(if is_release { "release" } else { "debug" })
}

fn backend_path(metadata: &Metadata, is_release: bool) -> Utf8PathBuf {
    metadata
        .target_directory
        .join(if is_release { "release" } else { "debug" })
        .join("backend")
}

fn output_default_path(metadata: &Metadata) -> Utf8PathBuf {
    metadata.workspace_root.join("output")
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
    let dist_path = dist_path(&metadata, config.release);

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

    let dist_path = dist_path(&metadata, config.release);

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

fn install(config: Install) -> Result<()> {
    dist(config.to_dist())?;

    let cmd = cargo_metadata::MetadataCommand::new();
    let metadata = cmd.exec()?;

    let dist_path = dist_path(&metadata, true);
    let backend_path = backend_path(&metadata, true);
    let output_dir = config.output.unwrap_or_else(|| output_default_path(&metadata));

    println!(
        "- Building backend in {}",
        backend_path.bold().fg::<UserGreen>()
    );

    let args = IntoIterator::into_iter(["build", "--release", "-p", "backend"]);
    duct::cmd("cargo", args)
        .run()
        .wrap_err("Could not build backend")?;


    println!(
        "- Copying frontend to {}",
        output_dir.join("static/dist").bold().fg::<UserGreen>()
    );

    std::fs::create_dir_all(&output_dir).wrap_err("Cannot create output dir")?;

    std::fs::remove_dir_all(&output_dir).wrap_err("Error while cleaning output directory")?;

    std::fs::create_dir_all(&output_dir.join("static")).wrap_err("Cannot create output dir")?;

    // no function in the stdlib to copy a directory, this will have to do for now
    let errors = copy_dir::copy_dir(&dist_path, &output_dir.join("static/dist"))
        .wrap_err("Could not copy dist dir to output dir")?;

    if !errors.is_empty() {
        eprintln!(
            "{} Copy succeeded, but the following errors occurred during the copy:",
            "WARNING:".bold().fg::<UserYellow>()
        );
        for error in errors {
            eprintln!("\t{}", error.fg::<UserYellow>())
        }
    }

    println!(
        "- Copying backend to {}",
        output_dir.join("backend").bold().fg::<UserGreen>()
    );

    std::fs::copy(backend_path, output_dir.join("backend"))
        .wrap_err("Copying the backend failed")?;

    Ok(())
}
