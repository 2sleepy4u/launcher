#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] 
use std::process::{Command, ExitStatus};
use std::io::{Result, Write};
use chrono::DateTime;

pub fn execute(exe: &str, args: &[&str]) -> Result<ExitStatus> {
    Command::new(exe).args(args).spawn()?.wait()
}

const DEFAULT_EXE_NAME: &str = "default.exe";

const GITHUB_NAME: &str = include_str!("../launcher.config");

#[tokio::main]
async fn main() {
    let version_date =
    if let Ok(value) = std::fs::read_to_string(".version") {
        if value.is_empty() {
            1431648000
        } else {
            value.parse().unwrap_or(0)
        }
    } else {
        let mut version_file = std::fs::File::create_new(".version").unwrap();
        let date = 1431648000;
        version_file.write_all(&date.to_string().as_bytes()).unwrap();
        drop(version_file);
        date

    };

    let (username, repo_name) = GITHUB_NAME.split_once("/").unwrap();

    let version_datetime = DateTime::from_timestamp(version_date, 0).unwrap();
    let octocrab = octocrab::instance();
    let page = octocrab.repos(username, repo_name.trim())
        .releases()
        .get_latest()
        .await
        .unwrap();
 
    let url = page.assets.first().unwrap().browser_download_url.clone();
    let exe_name = url.to_string();
    let exe_name = exe_name.split("/").last().unwrap_or(DEFAULT_EXE_NAME);

    let published_date = page.published_at.unwrap();
    if published_date > version_datetime {
        {
            let mut tmpfile = std::fs::File::create(exe_name).unwrap();
            let client = reqwest::ClientBuilder::new().user_agent("anyting/aaa").build().unwrap();
            let res = client.get(url).send().await.unwrap().bytes().await.unwrap();

            tmpfile.write_all(&res).unwrap();
        }
        let mut version_file = std::fs::File::options().write(true).open(".version").unwrap();
        version_file.write_all(&published_date.timestamp().to_string().as_bytes()).unwrap();
    }
    let _a = execute(exe_name, &Vec::new());
    //let mut zip = zip::ZipArchive::new(tmpfile).unwrap();
}
