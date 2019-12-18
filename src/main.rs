use anyhow::Result;
use async_std::fs;
use std::path::Path;
use strand::Plugin;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opts {
    /// Prints out the config file location
    #[structopt(long)]
    config_location: bool,

    #[structopt(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(StructOpt)]
enum Subcommand {
    /// Install a specific plugin without adding it to the config file
    #[structopt(name = "install")]
    Install {
        /// A list of plugins to install and add to the config file
        // require at least one Plugin in the Vec
        #[structopt(name = "PLUGINS", required = true)]
        plugins: Vec<Plugin>,
    },
}

#[async_std::main]
async fn main() -> Result<()> {
    let opts = Opts::from_args();

    let config_dir = strand::get_config_dir();
    let config_path = config_dir.join("config.yaml");

    // We do this before loading the config file because loading it is not actually needed to
    // display the config file’s location.
    if opts.config_location {
        println!("{}", config_path.display());
        return Ok(());
    }

    let config = strand::get_config(&config_path).await?;

    // Install all plugins specified by the install subcommand.
    if let Some(Subcommand::Install { plugins }) = opts.subcommand {
        strand::install_plugins(plugins, config.plugin_dir).await?;
        return Ok(()); // Early return since we don’t need to install plugins from the config file.
    }

    // Clean out the plugin directory before installing.
    ensure_empty_dir(&config.plugin_dir).await?;
    strand::install_plugins(config.plugins, config.plugin_dir).await?;

    Ok(())
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
