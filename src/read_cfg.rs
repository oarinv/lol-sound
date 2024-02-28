use ini::Ini;

pub fn read_cfg() -> (String, String) {
    let cfg = Ini::load_from_file("config.ini").unwrap();
    let section = cfg.section(Some("config")).unwrap();
    let riot_id = section.get("riot_id").unwrap().to_string();
    let media_path = section.get("media_path").unwrap().to_string();
    return (riot_id, media_path);
}
