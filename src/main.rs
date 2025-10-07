use chrono::{DateTime, Local};
use crossterm::{QueueableCommand, cursor};
use serde::Deserialize;
use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;
use std::fs;
use std::io::{Write, stdout};
use std::process::{Command, Stdio};
use std::{thread, time};
use toml;

#[derive(Deserialize, Debug)]
struct Config {
    webhook_url: String,
}

fn sleep_seconds(num_sec_to_sleep: u64) {
    let seconds_to_sleep = time::Duration::from_secs(num_sec_to_sleep);
    thread::sleep(seconds_to_sleep);
}

async fn send_discord_message(
    message: String,
    webhook_url: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let http = Http::new("");
    let webhook = Webhook::from_url(&http, &webhook_url).await?;
    let builder = ExecuteWebhook::new().content(message).username("JoshBot");
    webhook.execute(&http, false, builder).await?;
    Ok(())
}

fn restart_adguardhome() {
    let restart_adguardhome_result = Command::new("service")
        .arg("adguardhome")
        .arg("restart")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Restarting adguardhome");
    if !restart_adguardhome_result.success() {
        println!("Failed to restart adguardhome!");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let script_start_time = Local::now();
    println!(
        "AdGuardHome OpenWRT Restarter 1.0.0 initialized on {}.",
        script_start_time.naive_local().format("%m/%d/%Y %r")
    );
    let mut times_checked = 0;

    let toml_content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&toml_content)?;

    let google_dns_ip_address = "8.8.8.8";
    let google_hostname = "google.com";
    let mut internet_outage_start_time: Option<DateTime<Local>> = None;

    loop {
        times_checked += 1;
        stdout.queue(cursor::SavePosition).unwrap();
        stdout
            .write_all(format!("Checked {} times", times_checked).as_bytes())
            .unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();

        let ping_google_ip_result = Command::new("ping")
            .arg(google_dns_ip_address)
            .arg("-c")
            .arg("1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Checking if we have an internet connection");
        if !ping_google_ip_result.success() {
            // only set the internet_outage_start_time when the internet first goes out, otherwise
            // the time will be continuously updated even though it's the same outage
            if internet_outage_start_time.is_none() {
                internet_outage_start_time = Some(Local::now());
                println!(
                    "The internet went down at {}!",
                    internet_outage_start_time
                        .expect("Failed to get the time that the internet outage started")
                        .naive_local()
                        .format("%m/%d/%Y %r")
                );
            }
            sleep_seconds(5);
            continue;
        }
        if internet_outage_start_time.is_some() {
            let internet_outage_message = format!(
                "@everyone The internet went out at {} but is now back online.",
                internet_outage_start_time
                    .expect("Failed to get the time that the internet outage started")
                    .naive_local()
                    .format("%m/%d/%Y %r")
            );
            send_discord_message(internet_outage_message, &config.webhook_url).await?;
            internet_outage_start_time = None;
        }

        let ping_google_com_result = Command::new("ping")
            .arg(google_hostname)
            .arg("-c")
            .arg("1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Checking if DNS is working");
        if !ping_google_com_result.success() {
            println!("DNS appears to be down!");
            restart_adguardhome();
            // give the service some time to come back online
            sleep_seconds(10);
            send_discord_message(
                "@everyone DNS broke so AdGuardHome was restarted".to_string(),
                &config.webhook_url,
            )
            .await?;
        }

        sleep_seconds(30);
    }
}
