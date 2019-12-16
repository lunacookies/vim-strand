use anyhow::Result;
use async_std::task;
use serde::Deserialize;
use std::{
    fmt,
    path::{Path, PathBuf},
};

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

pub fn get_config_dir() -> PathBuf {
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

fn expand_path(path: &Path) -> PathBuf {
    use std::path::Component;

    if path.starts_with("~") {
        let mut components: Vec<_> = path.components().collect();
        let home_dir = get_home_dir().into_os_string();

        // Remove the tilde and add in its place the home directory.
        components.remove(0);
        components.insert(0, Component::Normal(&home_dir));

        // Join the components back into a single unified PathBuf.
        let mut path = PathBuf::new();
        components.iter().for_each(|segment| path.push(segment));

        path
    } else {
        path.into()
    }
}

#[derive(Deserialize)]
pub enum GitProvider {
    GitHub,
    Bitbucket,
}

// git_ref can be a branch name, tag name, or commit hash.
#[derive(Deserialize)]
pub struct GitRepo {
    provider: GitProvider,
    user: String,
    repo: String,
    git_ref: Option<String>,
}

impl fmt::Display for GitRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let git_ref = match &self.git_ref {
            Some(git_ref) => git_ref,
            None => "master",
        };

        match self.provider {
            GitProvider::GitHub => write!(
                f,
                "https://codeload.github.com/{}/{}/tar.gz/{}",
                self.user, self.repo, git_ref
            ),
            GitProvider::Bitbucket => write!(
                f,
                "https://bitbucket.org/{}/{}/get/{}.tar.gz",
                self.user, self.repo, git_ref
            ),
        }
    }
}

#[derive(Deserialize)]
pub struct ArchivePlugin(String);

impl fmt::Display for ArchivePlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Plugin {
    Git(GitRepo),
    Archive(ArchivePlugin),
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Plugin::Git(plugin) => write!(f, "{}", plugin),
            Plugin::Archive(plugin) => write!(f, "{}", plugin),
        }
    }
}

impl Plugin {
    async fn install_plugin(&self, path: PathBuf) -> Result<()> {
        use anyhow::Context;
        use std::process;

        let url = format!("{}", self);
        let archive = match surf::get(url).recv_bytes().await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        };
        decompress_tar_gz(&archive, &path).with_context(|| {
            format!(
                "failed to extact archive while installing plugin from URL {} -- got from server:\n‘{}’",
                self, String::from_utf8_lossy(&archive)
            )
        })?;
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
    let mut config: Config = yaml::from_str(&config)?;
    config.plugin_dir = expand_path(&config.plugin_dir);

    Ok(config)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        let home_dir = get_home_dir();

        assert_eq!(
            expand_path(Path::new("~/foo.txt")),
            home_dir.join("foo.txt")
        );

        assert_eq!(
            expand_path(Path::new("/home/person/foo.txt")),
            PathBuf::from("/home/person/foo.txt")
        );

        assert_eq!(
            expand_path(Path::new("~/bar/baz/quux/foo.txt")),
            home_dir.join("bar/baz/quux/foo.txt")
        );
    }
}
