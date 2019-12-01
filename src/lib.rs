use anyhow::Result;
use async_std::task;
use std::path::{Path, PathBuf};

pub async fn get_plugins(config_file: &Path) -> Result<Vec<String>> {
    use anyhow::Context;
    use async_std::fs;

    Ok(fs::read_to_string(config_file)
        .await
        .with_context(|| format!("could not read config ({})", config_file.display()))?
        .lines()
        .map(|s| s.into())
        .collect())
}

fn decompress_tar_gz(bytes: &[u8], path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let tar = GzDecoder::new(bytes);
    let mut archive = Archive::new(tar);
    archive.unpack(path)?;

    Ok(())
}

async fn install_plugin(url: &str, path: PathBuf) -> Result<()> {
    use std::process;

    let archive = match surf::get(url).recv_bytes().await {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    decompress_tar_gz(&archive, &path)?;
    println!("Installed {}", url);

    Ok(())
}

pub async fn install_plugins(plugins: Vec<String>, dir: PathBuf) -> Result<()> {
    let mut tasks = Vec::with_capacity(plugins.len());

    plugins.into_iter().for_each(|p| {
        let dir = dir.clone();
        tasks.push(task::spawn(async move { install_plugin(&p, dir).await }));
    });

    for task in tasks {
        task.await?;
    }

    Ok(())
}
