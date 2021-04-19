use avior_infuser_lib::log::Logger;

use crate::{LogExt, cfg::Config};

use std::{
    path::Path,
    error::Error,
    fs::{self, DirEntry, File},
    io::{self, BufReader, prelude::*}
};


pub struct DirectoryTraverser<'yingking> {
    cfg: &'yingking Config,
    logger: &'yingking mut Logger
}

impl<'yingking> DirectoryTraverser<'yingking> {
    pub fn new(cfg: &'yingking Config, logger: &'yingking mut Logger) -> DirectoryTraverser<'yingking> {
        DirectoryTraverser { cfg, logger }
    }

    pub fn traverse(&mut self, dir: &str) -> Result<(), Box<dyn Error>> {
        for entry in fs::read_dir(Path::new(dir))? {
            let file = entry?;
            if !file.file_type()?.is_file() {
              continue;
            }
            let path = file.path();
            let extension = path
                .extension()
                .and_then(|osstr| osstr.to_str())
                .unwrap_or("");
            let filetypes_iterator = self.cfg.filetypes.iter();
            if filetypes_iterator.any(|filetype| filetype == extension) {
                let filename = path.file_name()
                .and_then(|osstr| osstr.to_str())
                .unwrap_or("");
                let ignored_filetypes_iter = self.cfg.ignored_filetypes.iter();
                if ignored_filetypes_iter.any(|suffix| filename.ends_with(suffix)) {
                    continue;
                } else {
                  let file_stem = path.file_stem().and_then(|osstr| osstr.to_str()).unwrap_or("").to_owned();
                  file_stem.push_str(".txt");
                  let (name, subtitle) = get_file_titles(&file_stem, self.logger)?;
                }
            }
        }
        Ok(())
    }
}

fn get_file_titles(path: &str, logger: &mut Logger) -> Result<(String, String), Box<dyn Error>> {
    let mut name: String;
    let mut subtitle: String; 
    let file = File::open(Path::new(path)).log(logger)?;
    let file = BufReader::new(file);
    for line in file.lines() {
      let line = line?;
      let name_tag = "Title=";
      let subtitle_tag = "Info=";
      if line.starts_with(name_tag) {
        name = line[name_tag.len()-1..].to_owned();
      } else if line.starts_with(subtitle_tag) {
        subtitle = line[subtitle_tag.len()-1..].to_owned();
      }
    }
    Ok((name.to_owned(), subtitle.to_owned()))
  }

//directory
// media file.(mkv/mpg/ts/whatevs)
// media file.log - ignore
// media file.txt - ignore
// media file.INFO.log => no JOB


/*
[General]
Version=1.1

[Media]
Created=19.04.2021 05:00:08
Channel=arte HD (AC3,deu)

[0]
Id=4561
Date=19.04.2021
Time=05:00:00
Duration=01:30:00
Title=Angela Hewitt spielt die Goldberg-Variationen
Info=Musik Deutschland 2020
Description=Bachs berühmte Goldberg-Variationen gelten als Prüfstein für die spielerische Reife eines jeden Pianisten. ARTE zeigt das Stück in voller Länge, interpretiert von der Pianistin Angela Hewitt am Ort von Bachs beruflichem Schaffen: der Thomaskirche Leipzig. 2020 wurde der kanadischen Musikerin für ihre Verdienste um das musikalische OEuvre des Komponisten die Bach-Medaille verliehen.|[16:9] [H.264] [HD]|[stereo] [deu]|[stereo] [fra]|[stereo] [mul]|[stereo] [deu]|[DVB subtitles] [deu]|[DVB subtitles] [deu]|[DVB subtitles] [fra]|PDC: 19.04. 05:00 (631104)
Charset=255
Content=112
MinimumAge=0
TimerID={99E235EE-A5CE-4124-B2F4-DE10DFB1A4CD}

[Stats]
Errors=0
Size=7,21 GB (7741799016 bytes)
Avr. Datarate=1,457 MB/s
Device=DVBViewer Media Server (VDR-M) 8
 */