use std::fs;
use std::fs::File;
use std::io::BufReader;
use reqwest::ClientBuilder;
use serde_json::Value;
use sysinfo::System;
use rand::seq::SliceRandom;
use rodio::{OutputStream, Sink};


async fn play_music(){
    let current_folder = "./wav";  // Replace with the actual folder path
    let wav_files: Vec<String> = fs::read_dir(current_folder)
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter_map(|path| path.to_str().map(|s| s.to_owned()))
        .filter(|s| s.ends_with(".mp3"))
        .collect();

    let mut rng = rand::thread_rng();
    let Some(random_file) = wav_files.choose(&mut rng)else { todo!() };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = File::open(random_file).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);
    sink.sleep_until_end();
}

async fn in_game()-> Result<(), Box<dyn std::error::Error>> {
    let mut last_event_id = -1;
    Ok(loop {
        let client = ClientBuilder::new().danger_accept_invalid_certs(true).build()?;
        match client.get("https://127.0.0.1:2999/liveclientdata/eventdata").send().await {
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
                                        let killer_name = event.get("KillerName").and_then(|n| n.as_str()).unwrap_or("Unknown");
                                        let victim_name = event.get("VictimName").and_then(|n| n.as_str()).unwrap_or("Unknown");
                                        println!("{} kill {}", killer_name, victim_name);
                                        if killer_name == "突破手牛爺爺" || victim_name == "突破手牛爺爺" {
                                            play_music().await;
                                        }
                                        last_event_id = event_id;  // Update last event ID to current event ID
                                    }
                                }
                            }
                        }
                    }
                }
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
            if process.name() == process_name{
                in_game().await.expect("TODO: panic message");
            }

        }



    }
}

#[tokio::main]
async fn main() {
    check().await;
}