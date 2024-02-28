use rand::seq::SliceRandom;
use reqwest::ClientBuilder;
use rodio::{OutputStream, Sink};
use serde_json::Value;
use std::fs;
use std::fs::File;

use std::io::BufReader;
use std::time::Duration;
use sysinfo::System;

mod read_cfg;
use read_cfg::read_cfg;

async fn play_music() {
    let (_riot_id, media_path) = read_cfg();
    let current_folder = media_path; // Replace with the actual folder path
    let wav_files: Vec<String> = fs::read_dir(current_folder)
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter_map(|path| path.to_str().map(|s| s.to_owned()))
        .filter(|s| s.ends_with(".mp3"))
        .collect();

    let mut rng = rand::thread_rng();
    let Some(random_file) = wav_files.choose(&mut rng) else {
        todo!()
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = File::open(random_file).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);
    sink.sleep_until_end();
}

async fn in_game() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_event_id = -1;
    let (riot_id, _media_path) = read_cfg();
    Ok(loop {
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()?;
        match client
            .get("https://127.0.0.1:2999/liveclientdata/eventdata")
            .send()
            .await
        {
            Ok(res) => {
                if res.status().is_success() {
                    let body = res.text().await?;
                    let json_value: Value = serde_json::from_str(&body)?;

                    let events = json_value["Events"].as_array().ok_or("Events not found")?;
                    for event in events {
                        if let Some(event_name) = event["EventName"].as_str() {
                            if event_name == "ChampionKill" {
                                if let Some(event_id) = event["EventID"].as_i64() {
                                    if event_id > last_event_id {
                                        let killer_name = event
                                            .get("KillerName")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("Unknown");
                                        let victim_name = event
                                            .get("VictimName")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("Unknown");
                                        println!("{} kill {}", killer_name, victim_name);
                                        if killer_name == riot_id || victim_name == riot_id {
                                            play_music().await;
                                        }
                                        last_event_id = event_id; // Update last event ID to current event ID
                                    }
                                }
                            }
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    })
}

async fn check() {
    let mut sys = System::new_all();
    loop {
        sys.refresh_processes();
        let process_name = "League of Legends.exe";
        for (_pid, process) in sys.processes() {
            if process.name() == process_name {
                in_game().await.expect("TODO: panic message");
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() {
    check().await;
}
