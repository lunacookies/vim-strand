use anyhow::Result;
use async_std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opts {
    /// Sets the directory to install plugins into
    #[structopt(name = "PLUGIN DIR")]
    plugin_dir: PathBuf,
}

#[async_std::main]
async fn main() -> Result<()> {
    use async_macros::try_join;

    let opts = Opts::from_args();

    let config_dir = get_conf_dir();
    let plugin_file = config_dir.join("config");

    // Doing this in parallel is almost certainly complete overkill.
    let plugin_dir_empty = ensure_empty_dir(&opts.plugin_dir);
    let config_dir_exists = ensure_dir_exists(&config_dir);
    try_join!(plugin_dir_empty, config_dir_exists).await?;

    let plugins = strand::get_plugins(&plugin_file).await?;

    strand::install_plugins(plugins, opts.plugin_dir).await?;

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

fn get_conf_dir() -> PathBuf {
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

async fn ensure_dir_exists(path: &Path) -> Result<()> {
    Ok(if !path.exists() {
        fs::create_dir_all(path).await?
    })
}

async fn ensure_empty_dir(path: &Path) -> Result<()> {
    if path.exists() {
        remove_path(path).await?;
    }

    fs::create_dir_all(path).await?;

    Ok(())
}
