use anyhow::Result;
use async_std::task;
use serde::Deserialize;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub struct GitHubPlugin {
    pub user: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct ArchivePlugin(String);

#[derive(Deserialize)]
pub enum Plugin {
    GitHub(GitHubPlugin),
    Archive(ArchivePlugin),
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Plugin::GitHub(plugin) => write!(
                f,
                "https://codeload.github.com/{}/{}/tar.gz/master",
                plugin.user, plugin.repo
            ),
            Plugin::Archive(plugin) => write!(f, "{}", plugin.0),
        }
    }
}

impl Plugin {
    async fn install_plugin(&self, path: PathBuf) -> Result<()> {
        use std::process;

        let url = format!("{}", self);
        let archive = match surf::get(url).recv_bytes().await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        };
        decompress_tar_gz(&archive, &path)?;
        println!("Installed {}", self);

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub plugin_dir: PathBuf,
    pub plugins: Vec<Plugin>,
}

pub async fn get_config(config_file: &Path) -> Result<Config> {
    use async_std::fs;

    let config = fs::read_to_string(config_file).await?;
    Ok(yaml::from_str(&config)?)
}

fn decompress_tar_gz(bytes: &[u8], path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let tar = GzDecoder::new(bytes);
    let mut archive = Archive::new(tar);
    archive.unpack(path)?;

    Ok(())
}

pub async fn install_plugins(plugins: Vec<Plugin>, dir: PathBuf) -> Result<()> {
    let mut tasks = Vec::with_capacity(plugins.len());

    plugins.into_iter().for_each(|p| {
        let dir = dir.clone();
        tasks.push(task::spawn(async move { p.install_plugin(dir).await }));
    });

    for task in tasks {
        task.await?;
    }

    Ok(())
}
