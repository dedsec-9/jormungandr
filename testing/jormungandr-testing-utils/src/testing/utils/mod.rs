mod observer;

use crate::testing::jormungandr::TestingDirectory;
use fs_extra::dir::{move_dir, CopyOptions};
pub use observer::{Event, Observable, Observer};
use std::thread::panicking;

pub fn persist_dir_on_panic(
    temp_dir: Option<TestingDirectory>,
    additional_contents: Vec<(&str, &str)>,
) {
    if panicking() {
        let logs_dir = match tempfile::Builder::new().prefix("jormungandr_").tempdir() {
            Ok(dir) => dir.into_path(),
            Err(e) => {
                eprintln!("Could not create logs dir: {}", e);
                return;
            }
        };

        println!(
            "persisting node temp_dir after panic: {}",
            logs_dir.display()
        );

        if let Some(dir) = temp_dir {
            let options = CopyOptions {
                content_only: true,
                ..Default::default()
            };
            move_dir(dir.path(), &logs_dir, &options)
                .map(|_| ())
                .unwrap_or_else(|e| eprintln!("Could not move files to new dir: {}", e));
        }

        for (filename, content) in additional_contents {
            std::fs::write(logs_dir.join(filename), content)
                .unwrap_or_else(|e| eprint!("Could not write {} to disk: {}", filename, e));
        }
    }
}
