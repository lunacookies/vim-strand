use anyhow::{bail, Result};
use async_std::{sync, task};
use serde::Deserialize;
use shrinkwraprs::Shrinkwrap;
use std::{
    convert::TryFrom,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use thiserror::Error;
use url::Url;

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
    #[cfg(target_os = "macos")]
    let dir = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => get_home_dir().join(".config"),
    };

    #[cfg(not(target_os = "macos"))]
    let dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: could not locate config directory -- exiting.");
            std::process::exit(1);
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
    GitLab,
    Bitbucket,
}

#[derive(Error, Debug)]
pub enum GitProviderParseError {
    #[error("Git provider {0} not recognised -- try ‘github’, ‘gitlab’ or ‘bitbucket’ instead")]
    UnknownProvider(String),
}

impl FromStr for GitProvider {
    type Err = GitProviderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" => Ok(GitProvider::GitHub),
            "gitlab" => Ok(GitProvider::GitLab),
            "bitbucket" => Ok(GitProvider::Bitbucket),
            _ => Err(Self::Err::UnknownProvider(s.into())),
        }
    }
}

// git_ref can be a branch name, tag name, or commit hash.
#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct GitRepo {
    provider: GitProvider,
    user: String,
    repo: String,
    git_ref: String,
}

impl TryFrom<&GitRepo> for Url {
    type Error = url::ParseError;

    fn try_from(gr: &GitRepo) -> Result<Self, Self::Error> {
        Url::parse(&match gr.provider {
            GitProvider::GitHub => format!(
                "https://codeload.github.com/{}/{}/tar.gz/{}",
                gr.user, gr.repo, gr.git_ref
            ),
            GitProvider::GitLab => format!(
                "https://gitlab.com/{0}/{1}/-/archive/{2}/{0}-{2}.tar.gz",
                gr.user, gr.repo, gr.git_ref
            ),
            GitProvider::Bitbucket => format!(
                "https://bitbucket.org/{}/{}/get/{}.tar.gz",
                gr.user, gr.repo, gr.git_ref
            ),
        })
    }
}

impl fmt::Display for GitRepo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repo)
    }
}

#[derive(Error, Debug)]
pub enum GitRepoParseError {
    #[error("no user was found")]
    MissingUser,
    #[error("failed to parse Git provider")]
    ProviderParse(#[from] GitProviderParseError),
}

fn split_on_pattern<'a>(s: &'a str, pattern: &str, i: &mut usize) -> Option<&'a str> {
    s.find(pattern).map(|x| {
        *i += x;
        let result = &s[..x];
        *i += pattern.len(); // Skip the pattern we matched on.

        result
    })
}

impl FromStr for GitRepo {
    type Err = GitRepoParseError;

    // TODO: Refactor to remove the index and mutate a slice instead.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Index through the input’s characters so that we can ignore the part that has already
        // been parsed.
        let mut i = 0;

        // Default to GitHub when the provider is elided.
        let provider = split_on_pattern(&input, "@", &mut i)
            .map_or(Ok(GitProvider::GitHub), GitProvider::from_str)?;

        let user =
            split_on_pattern(&input[i..], "/", &mut i).ok_or_else(|| Self::Err::MissingUser)?;

        // When the ‘:’ signifier for a Git reference is found, the part preceding it must be the
        // repo name and the part after the Git reference. If it is not found, the rest of ‘input’
        // must be the repo name, in this case using ‘master’ as the default Git reference.
        //
        // FIXME: Some repos have something different to ‘master’ as their default branch. Handle
        // this somehow.
        let (repo, git_ref) = match split_on_pattern(&input[i..], ":", &mut i) {
            Some(repo) => (repo, &input[i..]),
            None => (&input[i..], "master"),
        };

        Ok(Self {
            provider,
            user: user.into(),
            repo: repo.into(),
            git_ref: git_ref.into(),
        })
    }
}

impl TryFrom<String> for GitRepo {
    type Error = GitRepoParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

#[derive(Deserialize, Shrinkwrap)]
pub struct ArchivePlugin(Url);

impl fmt::Display for ArchivePlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

impl FromStr for ArchivePlugin {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Url::from_str(s).map(ArchivePlugin)
    }
}

enum InstallStateKind {
    Downloading,
    Extracting,
    Installed,
    Retry(u32),
    Error(anyhow::Error),
}

impl fmt::Display for InstallStateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::*;

        match self {
            InstallStateKind::Downloading => write!(f, "{}", "Downloading".cyan().bold()),
            InstallStateKind::Extracting => write!(f, " {}", "Extracting".blue().bold()),
            InstallStateKind::Installed => write!(f, "✓ {}", "Installed".green().bold()),
            InstallStateKind::Retry(i) => {
                write!(f, "      {}: attempt #{} of", "Retry".yellow().bold(), i)
            }
            InstallStateKind::Error(e) => write!(f, "×     {}: {}", "Error".red().bold(), e),
        }
    }
}

struct InstallState {
    status: InstallStateKind,
    name: String,
}

impl fmt::Display for InstallState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.status, self.name)
    }
}

#[derive(Deserialize)]
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

impl TryFrom<&Plugin> for Url {
    type Error = url::ParseError;

    fn try_from(p: &Plugin) -> Result<Self, Self::Error> {
        match p {
            Plugin::Git(gr) => Url::try_from(gr),
            Plugin::Archive(a) => Ok((*a).clone()),
        }
    }
}

#[derive(Error, Debug)]
pub enum PluginParseError {
    #[error("failed to parse Git repo: {0}")]
    GitParse(#[from] GitRepoParseError),
    #[error("failed to parse archive plugin: {0}")]
    ArchiveParse(#[from] url::ParseError),
}

impl FromStr for Plugin {
    type Err = PluginParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ArchivePlugin::from_str(s)
            .map(Plugin::Archive)
            .or_else(|_| GitRepo::from_str(s).map(Plugin::Git).map_err(|e| e.into()))
    }
}

async fn recv_bytes_retry(
    url: &str,
    s: &sync::Sender<InstallState>,
    name: &str,
) -> Result<Vec<u8>> {
    let mut attempts = 0;
    let max_attempts = 5;

    // Try downloading five times before giving up.
    loop {
        match surf::get(url).recv_bytes().await {
            Ok(response) => return Ok(response),
            Err(_) if attempts < max_attempts => attempts += 1,
            Err(e) => bail!("failed retrieving contents at URL {}: {}", url, e),
        }

        s.send(InstallState {
            status: InstallStateKind::Retry(attempts),
            name: name.into(),
        })
        .await;

        task::sleep(Duration::from_secs(2)).await; // Sleep for two seconds between attempts.
    }
}

impl Plugin {
    async fn install(&self, path: PathBuf, s: sync::Sender<InstallState>) -> Result<()> {
        use anyhow::Context;

        let name = self.to_string();

        s.send(InstallState {
            status: InstallStateKind::Downloading,
            name: name.clone(),
        })
        .await;

        let recv_bytes = recv_bytes_retry(&Url::try_from(self)?.as_str(), &s, &name)
            .await
            .with_context(|| "failed downloading plugin")?;

        if &b"404: Not Found\n" == &recv_bytes.as_slice() {
            bail!("plugin does not exist (404)");
        }

        s.send(InstallState {
            status: InstallStateKind::Extracting,
            name: name.clone(),
        })
        .await;

        decompress_tar_gz(&recv_bytes, &path)
            .with_context(|| "failed to extract plugin archive")?;

        s.send(InstallState {
            status: InstallStateKind::Installed,
            name,
        })
        .await;

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
    use pbr::MultiBar;

    let mut tasks = Vec::with_capacity(plugins.len());
    let mut multi = MultiBar::new(); // Holds the spinners of all plugins

    plugins.into_iter().for_each(|p| {
        // We have to make a fresh clone of ‘dir’ for each plugin so that the task’s future stays
        // 'static.
        let dir = dir.clone();

        // Create a new spinner connected to the MultiBar that shows only the spinner itself and the
        // message we set.
        let mut spinner = multi.create_bar(0);
        spinner.show_bar = false;
        spinner.show_counter = false;
        spinner.show_percent = false;
        spinner.show_speed = false;
        spinner.show_time_left = false;
        spinner.tick_format("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"); // Nice spinner characters.

        // Channel to communicate between the ticker task and the plugin installation task.
        let (s, r): (_, sync::Receiver<InstallState>) = sync::channel(1);

        tasks.push(task::spawn(async move {
            let ticker = task::spawn(async move {
                // Tick the spinner every fifty milliseconds until the plugin has finished
                // installing or an error occurs.
                loop {
                    if r.is_full() {
                        let install_state = r.recv().await.unwrap();
                        let msg = format!("{}  ", install_state);

                        if let InstallStateKind::Installed | InstallStateKind::Error(_) =
                            install_state.status
                        {
                            spinner.finish_print(&msg);
                            break;
                        } else {
                            spinner.message(&msg);
                        }
                    }

                    spinner.tick();
                    task::sleep(Duration::from_millis(50)).await;
                }
            });

            // If the plugin install fails we send the error that occurred to the spinner for
            // display to the user.
            let install = task::spawn(async move {
                if let Err(e) = p.install(dir, s.clone()).await {
                    s.send(InstallState {
                        status: InstallStateKind::Error(e),
                        name: p.to_string(),
                    })
                    .await;
                }
            });

            ticker.await;
            install.await;
        }));
    });

    // Start listening for spinner activity just before the plugins’ installation is commenced.
    multi.listen();

    for task in tasks {
        task.await;
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
