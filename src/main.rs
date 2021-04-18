use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    path,
};
mod cfg;
mod dir;
use avior_infuser_lib::db::get_machine_jobcount;
use avior_infuser_lib::MongoClient;
use avior_infuser_lib::*;
static CFG_STRING: &str = "dis_config.toml";

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = cfg::read(CFG_STRING)?;
    let db_client: MongoClient = db::connect(&cfg.db_url)?;
    let grouped_clients = get_grouped_clients(&db_client, &cfg.db_name)?;
    Ok(())
}

fn get_grouped_clients<'a>(
    db_client: &MongoClient,
    db_name: &String,
) -> Result<(Vec<Client>, BTreeMap<i32, HashMap<&'a Client, Option<i32>>>), Box<dyn Error>> {
    let client_vec = db::get_clients(&db_client, db_name)?;
    let machine_jobcounts = get_machine_jobcount(&db_client, db_name)?;
    let grouped_clients = group_clients(&client_vec, machine_jobcounts);
    Ok((client_vec, grouped_clients))
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
*/
