use anyhow::Result;
use async_std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opts {
    /// Prints out the config file location
    #[structopt(long)]
    config_location: bool,
}

#[async_std::main]
async fn main() -> Result<()> {
    let opts = Opts::from_args();

    let config_dir = get_config_dir();

    // We do this before loading the config file because loading it is not actually needed to
    // display the config file’s location.
    if opts.config_location {
        println!("{}", config_dir.display());
        return Ok(());
    }

    let config_path = config_dir.join("config.yaml");
    let config = strand::get_config(&config_path).await?;

    // Clean out the plugin directory before installing.
    ensure_empty_dir(&config.plugin_dir).await?;
    strand::install_plugins(config.plugins, config.plugin_dir).await?;

    Ok(())
}

fn get_home_dir() -> PathBuf {
    use std::process;

    match dirs::home_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: could not locate home directory -- exiting.");
            process::exit(1);
        }
    }
}

fn get_config_dir() -> PathBuf {
    use std::{env, process};

    #[cfg(target_os = "macos")]
    let dir = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => get_home_dir().join("config"),
    };

    #[cfg(not(target_os = "macos"))]
    let dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: could not locate config directory -- exiting.");
            process::exit(1);
        }
    };

    dir.join("strand")
}

async fn remove_path(path: &Path) -> Result<()> {
    if fs::metadata(path).await?.is_dir() {
        fs::remove_dir_all(path).await?;
    } else {
        fs::remove_file(path).await?;
    }

    Ok(())
}

async fn ensure_empty_dir(path: &Path) -> Result<()> {
    if path.exists() {
        remove_path(path).await?;
    }

    fs::create_dir_all(path).await?;

    Ok(())
}
