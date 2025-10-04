use crossterm::{QueueableCommand, cursor};
use std::io::{Write, stdout};
use std::process::{Command, Stdio};
use std::{thread, time};
use chrono::Local;

fn sleep_30_sec() {
    let seconds_to_sleep = time::Duration::from_secs(30);
    thread::sleep(seconds_to_sleep);
}

fn main() {
    let mut stdout = stdout();
    let date = Local::now();
    println!("AdGuardHome OpenWRT Restarter 1.0.0 initialized on {}.", date.format("%m/%d/%Y %H:%M:%S"));
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
            sleep_30_sec();
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
        }

        sleep_30_sec();
    }
}
