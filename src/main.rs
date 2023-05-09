use core::panic;
use std::{
    collections::{BTreeMap, HashMap},
    env,
    error::Error,
    fmt::Display,
};
mod cfg;
mod dir;
use avior_infuser_lib::MongoClient;
use avior_infuser_lib::*;
use log::{Log, Logger};

use crate::dir::DirectoryTraverser;
static CFG_STRING: &str = "dis_config.toml";
static LOG_PATH: &str = "avior_dis.log";
const IDENTITY: &str = "avior dis, version 0.1.1 - dot";

trait LogExt {
    fn log_and_flush(self, logger: &mut Logger) -> Self;
    fn log(self, logger: &mut Logger) -> Self;
}

impl<T, E> LogExt for Result<T, E>
where
    E: std::fmt::Display + std::fmt::Debug,
{
    fn log_and_flush(self, logger: &mut Logger) -> Self {
        if let Err(e) = &self {
            logger.add(&format!("{}", &e));
            if let Err(e) = logger.flush(LOG_PATH, log::Mode::Append) {
                eprint!("{:?}", e);
            }
        }
        self
    }
    fn log(self, logger: &mut Logger) -> Self {
        if let Err(e) = &self {
            logger.add(&format!("{}", &e));
            eprint!("{:?}", e);
        }
        self
    }
}

#[derive(Debug)]
struct VecWrapper<'a>(&'a [String]);

impl<'a> Display for VecWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        out.push('[');
        for (idx, elem) in self.0.iter().enumerate() {
            out.push_str(elem);
            if idx < self.0.len() - 1 {
                out.push_str(", ")
            } else {
                out.push(']');
            }
        }
        write!(f, "{}", out)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut logger: Logger = Log::new(IDENTITY);

    let args: Vec<String> = env::args().collect();
    // print args using println! except for the first one
    println!("cli arguments:");
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            println!("{}", arg);
        }
    }
    if args.len() < 2 {
        panic!("invalid scan path, scan path must be specified");
    }

    logger.add(&format!("scan directory: {}", args[1]));

    let cfg = cfg::read(CFG_STRING)?;
    let db_client: MongoClient = db::connect(&cfg.db_url)?;
    let client_vec = db::get_clients(&db_client, &cfg.db_name).log_and_flush(&mut logger)?;
    let default_client = client_vec
        .iter()
        .find(|client| client.name == cfg.default_client)
        .expect("mate, the default client has to exist...")
        .to_owned();
    let mut grouped_clients: BTreeMap<i32, HashMap<Client, Option<i32>>> = group_clients(
        client_vec,
        db::get_machine_jobcount(&db_client, &cfg.db_name)?,
    );

    let mut dir_trav: DirectoryTraverser = DirectoryTraverser::new(&cfg, &mut logger);

    //let parsed_jobs = vec![j1, j2];
    let parsed_jobs = dir_trav.traverse(&args[1]).log_and_flush(&mut logger)?;
    let res = push_all_parsed(
        parsed_jobs,
        &db_client,
        &cfg.db_name,
        &default_client,
        &mut grouped_clients,
        &mut logger,
    );

    logger.flush(LOG_PATH, log::Mode::Append)?;
    res
}

fn push_all_parsed(
    mut parsed_jobs: Vec<Job>,
    db_client: &MongoClient,
    db_name: &String,
    default_client: &Client,
    grouped_clients: &mut BTreeMap<i32, HashMap<Client, Option<i32>>>,
    logger: &mut Logger,
) -> Result<(), Box<dyn Error>> {
    let mut insert_jobs: Vec<Job> = parsed_jobs
        .drain(..)
        .filter(|parsed_job| {
            let exists_result = db::job_exists(&db_client, db_name, &parsed_job.path);
            match exists_result {
                Ok(exists) => {
                    return !exists;
                }
                Err(err) => {
                    logger.add(&format!(
                        "error {:?} while trying to find job {}",
                        err, &parsed_job.path
                    ));
                    return false;
                }
            };
        })
        .collect();
    if insert_jobs.len() == 0 {
        logger.add(&format! {"database already up to date"})
    }
    for job in insert_jobs.iter_mut() {
        let grouped_clients_reload = grouped_clients.clone();
        let eligible = get_eligible_client(&grouped_clients_reload);
        let push_result = match eligible {
            // push to eligible if available
            Ok((client, current, max)) => {
                logger.add(&format!(
                    "pushing {} to {} with {}/{} job(s) and priority {}",
                    job.path, client.name, current, max, client.priority
                ));
                push_and_increment(&db_client, db_name, job, client, grouped_clients)
            }
            // push to default instead if no client is eligible
            Err(_) => {
                logger.add(&format!(
                    "no eligible found, pushing {} to default client {}",
                    job.path, default_client.name
                ));
                push_and_increment(&db_client, db_name, job, default_client, grouped_clients)
            }
        };
        // if a db push fails, log it and retry with the next job.
        // In case the database is messed up, dis can be run again safely since it doesn't add duplicates
        if let Err(e) = push_result {
            logger.add(&format!("could not push {}, error: {}", job.path, e));
        }
    }
    Ok(())
}

fn push_and_increment(
    db_client: &MongoClient,
    db_name: &String,
    job: &mut Job,
    client: &Client,
    grouped_clients: &mut BTreeMap<i32, HashMap<Client, Option<i32>>>,
) -> Result<String, Box<dyn Error>> {
    job.assigned_client = client.to_owned().into();
    let res = db::insert_job(&db_client, db_name, job)?;
    let prio_group = grouped_clients.get_mut(&client.priority).unwrap();
    prio_group
        .entry(client.to_owned())
        .and_modify(|assigned| match assigned {
            Some(v) => *assigned = Some(*v + 1),
            None => *assigned = Some(1),
        });
    Ok(res)
}

/*
//directory
media file.(mkv/mpg/ts/whatevs)
media file.log - ignore
media file.txt - ignore
media file.INFO.log => no JOB

=> Job

cli:
param1: path
param2: 3

=> library


    let j1 = Job {
        id: None,
        path: "\\\\vdr-u\\SDuRec\\Recording\\exists\\Geheimnisvolle Wildblumen_2021-04-10-14-58-01-arte HD (AC3,deu).ts".to_string(),
        name: "Geheimnisvolle Wildblumen".to_string(),
        subtitle: "Blütenpracht im Wald".to_string(),
        assigned_client: AssignedClient::default(),
        custom_parameters: Vec::new()
    };
    let j2 =  Job {
        id: None,
        path: "\\\\vdr-u\\SDuRec\\Recording\\Die Zauberflöte von Mozart_2021-04-18-23-48-00-arte HD (AC3,deu).ts".to_string(),
        name: "Die Zauberflöte von Mozart".to_string(),
        subtitle: "".to_string(),
        assigned_client: AssignedClient::default(),
        custom_parameters: Vec::new()
    };
    let j3 =  Job {
        id: None,
        path: "\\\\vdr-u\\SDuRec\\Recording\\Die Zauberflizzle von Mozizzle_2021-04-18-23-48-00-arte HD (AC3,deu).ts".to_string(),
        name: "Die Zauberflizzle von Mozizzle".to_string(),
        subtitle: "".to_string(),
        assigned_client: AssignedClient::default(),
        custom_parameters: Vec::new()
    };

*/

#[cfg(test)]
mod tests {
    use crate::cfg;
    use crate::dir::DirectoryTraverser;
    use avior_infuser_lib::log::*;
    use std::error::Error;

    static IDENTITY: &str = "avior dis, version 0.1.1 - tarantula";
    static CFG_STRING: &str = "dis_config.toml";
    static LOG_PATH: &str = "avior_traverse_test.log";

    #[test]
    fn test_traverse() -> Result<(), Box<dyn Error>> {
        let mut logger: Logger = Log::new(IDENTITY);
        let cfg = cfg::read(CFG_STRING)?;
        let mut dir_trav: DirectoryTraverser = DirectoryTraverser::new(&cfg, &mut logger);
        let mut job_vector = dir_trav.traverse(&"D:\\Recording");
        // print the vector
        for job in job_vector.iter_mut() {
            logger.add(&format!("{:?}", job));
        }
        logger.flush(LOG_PATH, Mode::Overwrite)
    }
}
