use anyhow::ensure;
use anyhow::Context;
use std::path::Path;
use std::path::PathBuf;
use tokio::task::JoinSet;

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "download",
    description = "download a post from imgchest.com"
)]
pub struct Options {
    #[argh(positional, description = "the url of the post")]
    pub url: String,

    #[argh(
        option,
        short = 'o',
        long = "out-dir",
        default = "PathBuf::from(\".\")",
        description = "the directory to download to"
    )]
    pub out_dir: PathBuf,
}

pub async fn exec(client: imgchest::Client, options: Options) -> anyhow::Result<()> {
    let post = client
        .get_scraped_post(options.url.as_str())
        .await
        .context("failed to get post")?;

    let out_dir = options.out_dir.join(&*post.id);

    tokio::fs::create_dir_all(&out_dir)
        .await
        .context("failed to create out dir")?;

    let post_json = serde_json::to_string(&post)?;
    tokio::fs::write(out_dir.join("post.json"), &post_json).await?;

    let mut join_set = JoinSet::new();
    let mut total_downloads = post.image_count;
    for image in post.images.iter() {
        if image.video_link.is_some() {
            total_downloads += 1;
        }
        spawn_image_download(&client, &mut join_set, image, &out_dir);
    }

    if let Some(extra_image_count) = post.extra_image_count {
        let extra_files = client
            .load_extra_files_for_scraped_post(&post)
            .await
            .context("failed to load extra for post")?;

        ensure!(extra_files.len() == usize::try_from(extra_image_count)?);

        for image in extra_files.iter() {
            if image.video_link.is_some() {
                total_downloads += 1;
            }
            spawn_image_download(&client, &mut join_set, image, &out_dir);
        }
    }

    let mut last_error = None;
    let mut downloaded = 0;
    while let Some(result) = join_set.join_next().await {
        match result
            .context("failed to join tokio task")
            .and_then(|result| result)
        {
            Ok(_new_download) => {
                downloaded += 1;
                println!("{downloaded} / {total_downloads}...");
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    if let Some(e) = last_error {
        return Err(e);
    }

    Ok(())
}

fn spawn_image_download(
    client: &imgchest::Client,
    join_set: &mut JoinSet<anyhow::Result<bool>>,
    file: &imgchest::ScrapedPostFile,
    out_dir: &Path,
) {
    {
        let client = client.clone();
        let link = file.link.clone();
        let out_path_result = file
            .link
            .split('/')
            .next_back()
            .context("missing file name")
            .map(|file_name| out_dir.join(file_name));
        join_set.spawn(async move {
            let out_path = out_path_result?;
            if tokio::fs::try_exists(&out_path)
                .await
                .context("failed to check if file exists")?
            {
                return Ok(false);
            }

            nd_util::download_to_path(&client.client, &link, &out_path).await?;

            Ok(true)
        });
    }

    if let Some(video_link) = file.video_link.clone() {
        let client = client.clone();
        let out_path_result = video_link
            .split('/')
            .next_back()
            .context("missing file name")
            .map(|file_name| out_dir.join(file_name));
        join_set.spawn(async move {
            let out_path = out_path_result?;
            if tokio::fs::try_exists(&out_path)
                .await
                .context("failed to check if file exists")?
            {
                return Ok(false);
            }

            nd_util::download_to_path(&client.client, &video_link, &out_path).await?;

            Ok(true)
        });
    }
}
