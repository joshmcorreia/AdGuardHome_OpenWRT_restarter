use chrono::Local;
use crossterm::{QueueableCommand, cursor};
use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;
use std::io::{Write, stdout};
use std::process::{Command, Stdio};
use std::{thread, time};

fn sleep_seconds(num_sec_to_sleep: u64) {
    let seconds_to_sleep = time::Duration::from_secs(num_sec_to_sleep);
    thread::sleep(seconds_to_sleep);
}

async fn send_discord_message(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let webhook_url = "REPLACE_ME";
    let http = Http::new("");
    let webhook = Webhook::from_url(&http, webhook_url).await?;
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
    let date = Local::now();
    println!(
        "AdGuardHome OpenWRT Restarter 1.0.0 initialized on {}.",
        date.format("%m/%d/%Y %H:%M:%S")
    );
    let mut times_checked = 0;

    let google_dns_ip_address = "8.8.8.8";
    let google_hostname = "google.com";

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
            println!("The internet is down!");
            // There isn't a way to send the user a discord message since their internet is out,
            // but we at least log it in the console
            sleep_seconds(30);
            continue;
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
            send_discord_message("@everyone DNS broke so AdGuardHome was restarted").await?;
        }

        sleep_seconds(30);
    }
}
