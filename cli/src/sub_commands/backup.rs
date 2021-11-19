use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use anyhow::Context;
use argh::FromArgs;

use database::models::comment::CommentModel;
use database::models::post::PostModel;

use crate::Run;

#[derive(FromArgs, PartialEq, Debug)]
/// backup
#[argh(subcommand, name = "backup")]
pub struct BackupSubCommand {
    #[argh(option, short = 'o')]
    /// output
    output: Option<PathBuf>,

    #[argh(option, short = 'r')]
    /// recover backup
    recover: Option<PathBuf>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct BackupStruct {
    config: config::MaopConfig,
    post: Vec<PostModel>,
    comment: Vec<CommentModel>,
}

impl BackupSubCommand {
    pub fn run(&self, args: &Run) {
        config::init(args.conf.iter().map(|s| s.into()).collect())
            .expect("config error");
        let config = config::get_config_temp();

        if let Some(path) = &self.recover {
            // recover

            let file = OpenOptions::new()
                .read(true)
                .open(path)
                .with_context(|| {
                    format!("not found {}", path.display())
                })
                .unwrap();

            let file_len = file.metadata().unwrap().len();

            let mut data =
                Vec::with_capacity((file_len as f64 * 1.5) as usize);
            brotli::Decompressor::new(
                BufReader::new(file),
                file_len as usize,
            )
            .read_to_end(&mut data)
            .unwrap();

            let backup =
                rmp_serde::from_slice::<BackupStruct>(&data).unwrap();

            let _: Result<(), ()> = try {
                let output = path.parent().ok_or(())?.join(format!(
                    "{}.json",
                    path.file_name().ok_or(())?.to_string_lossy()
                ));
                let mut config_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&output)
                    .unwrap();
                config_file
                    .write_all(
                        serde_json::to_string_pretty(&backup.config)
                            .unwrap()
                            .as_bytes(),
                    )
                    .unwrap();
                config_file.sync_data().unwrap();
                println!(
                    "config file recover to {}",
                    output.display()
                );
            };

            Result::<_, anyhow::Error>::unwrap(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        let db = database::new().await?;
                        database::models::post::Post::recover(
                            &db,
                            backup.post.clone(),
                        )
                        .await?;
                        database::models::comment::Comment::recover(
                            &db,
                            backup.comment.clone(),
                        )
                        .await?;
                        Ok(())
                    }),
            );
        } else {
            // backup

            let output = self.output.clone().unwrap_or_else(|| {
                let backup_dir = config.data_path().join("backup");
                create_dir_all(&backup_dir).unwrap();
                backup_dir.join(format!(
                    "{}.backup",
                    chrono::Local::now().to_rfc3339()
                ))
            });

            let config_backup = config::MaopConfig::clone(&*config);

            let (post_backup, comment_backup) = Result::<_, anyhow::Error>::unwrap(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        let db = database::new().await?;
                        Ok((database::models::post::Post::find_all(&db).await?, database::models::comment::Comment::find_all(&db).await?))
            }));

            let backup = BackupStruct {
                config: config_backup,
                post: post_backup,
                comment: comment_backup,
            };

            let data = rmp_serde::to_vec(&backup).unwrap();

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output)
                .unwrap();

            let mut writer = brotli::CompressorWriter::new(
                &mut file,
                data.len(),
                11,
                22,
            );
            writer.write_all(&data).unwrap();

            writer.flush().unwrap();
            std::mem::drop(writer);
            file.sync_all().unwrap();

            println!("backup to {}", output.display());
        }
    }
}
