use avior_infuser_lib::log::Log;
use avior_infuser_lib::Job;
use avior_infuser_lib::{log::Logger, AssignedClient};

use crate::cfg::Config;

use std::convert::TryFrom;
use std::time::SystemTime;
use std::{
    error::Error,
    fs::{self, File},
    io::{prelude::*, BufReader},
    path::Path,
};

pub struct DirectoryTraverser<'yingking> {
    cfg: &'yingking Config,
    logger: &'yingking mut Logger,
}

impl<'yingking> DirectoryTraverser<'yingking> {
    pub fn new(
        cfg: &'yingking Config,
        logger: &'yingking mut Logger,
    ) -> DirectoryTraverser<'yingking> {
        DirectoryTraverser { cfg, logger }
    }

    pub fn traverse(&mut self, dir: &str) -> Result<Vec<Job>, Box<dyn Error>> {
        let mut jobs: Vec<Job> = Vec::new();
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

            // only iterate over files that have the specified file extension
            let mut filetypes_iterator = self.cfg.filetypes.iter();
            if filetypes_iterator.any(|filetype| filetype == &extension.to_lowercase()) {
                let mut ignored_filetypes_iter = self.cfg.ignored_filetypes.iter();

                let path_to_str = match path.to_str() {
                    Some(path_str) => path_str,
                    None => {
                        self.logger.add(&"could not find path string");
                        continue;
                    }
                };

                // filter out files if they have an ignored sibling file
                if ignored_filetypes_iter.any(|suffix| {
                    let mut check_path_str = path_to_str.clone().to_string();
                    check_path_str.push_str(".");
                    check_path_str.push_str(suffix);
                    Path::new(&check_path_str).exists()
                }) {
                    println!("skipping {} because file was ignored by user", path_to_str);
                    continue;
                }

                // filter out files that are too new
                if let Ok(metadata) = file.metadata() {
                    let last_modified = metadata.modified()?;
                    let duration = match SystemTime::now().duration_since(last_modified) {
                        Ok(duration) => duration,
                        Err(_) => continue,
                    };
                    let days_duration =
                        i32::try_from(duration.as_secs() / (60 * 60 * 24)).unwrap_or(0);
                    if days_duration < self.cfg.min_age {
                        println!(
                            "skipping {} due to ({} age / {} mininum)",
                            path_to_str, days_duration, self.cfg.min_age
                        );
                        continue;
                    }
                }

                println!("scanning {}", path_to_str);

                let parse_result =
                    self.get_file_titles(&path.with_extension("txt").to_str().unwrap_or_default());

                // if found, add job to the output vector
                if let Ok((name, subtitle)) = parse_result {
                    let job = Job {
                        id: None,
                        name,
                        subtitle,
                        path: String::from(path_to_str),
                        assigned_client: AssignedClient::default(),
                        custom_parameters: Vec::new(),
                    };
                    jobs.push(job);
                }
            }
        }
        Ok(jobs)
    }

    fn get_file_titles(&mut self, path: &str) -> Result<(String, String), Box<dyn Error>> {
        let mut name: String = "".to_string();
        let mut subtitle: String = "".to_string();

        let file = match File::open(Path::new(path)) {
            Ok(opened_file) => opened_file,
            Err(err) => {
                self.logger.add(&format!("{} for: {}", err, path));
                return Err(Box::new(err));
            }
        };
        //let decoded = DecodeReaderBytesBuilder::new().encoding(Some(WINDOWS_1252)).build(file);
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(err) => {
                    self.logger.add(&format!("{} for: {}", err, path));
                    return Err(Box::new(err));
                }
            };
            let name_tag = "Title=";
            let subtitle_tag = "Info=";
            if line.starts_with(name_tag) {
                name = line[name_tag.len()..].to_owned();
            } else if line.starts_with(subtitle_tag) {
                subtitle = line[subtitle_tag.len()..].to_owned();
            }
        }
        Ok((name.to_owned(), subtitle.to_owned()))
    }
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
