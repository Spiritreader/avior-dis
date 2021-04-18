use crate::cfg::Config;
use avior_infuser_lib::*;
use std::{
    error::Error,
    fs::{self, FileType, Metadata, Path},
};
use walkdir::{DirEntry, WalkDir};

pub struct DirectoryTraverser<'yingking> {
    cfg: &'yingking Config,
}

impl<'yingking> DirectoryTraverser<'yingking> {
    pub fn new(cfg: &'yingking Config) -> DirectoryTraverser<'yingking> {
        DirectoryTraverser { cfg }
    }

    pub fn traverse(&self, dir: &str) -> Result<(), Box<dyn Error>> {
        for entry in fs::read_dir(Path::new(dir)) {
            let file = entry?;
            let metadata = file.metadata()?;
            let path = file.path();
        }
        Ok(())
    }

    fn is_ignored(&self, entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|filename| {
                self.cfg
                    .ignored_filetypes
                    .iter()
                    .any(|&suffix| filename.ends_with(suffix))
            })
            .unwrap_or(false)
    }

    fn is_mediatype(&self, entry: &DirEntry) -> bool {
        // entry.metadata().
    }
}

//directory
// media file.(mkv/mpg/ts/whatevs)
// media file.log - ignore
// media file.txt - ignore
// media file.INFO.log => no JOB
