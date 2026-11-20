//! Parsed MPRIS metadata from `playerctl`.

#[derive(Default, Clone, Debug)]
pub struct PlayerMeta {
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub player_name_raw: String,
    pub art_url: String,
    pub pos_secs: f64,
    pub len_secs: f64,
}

/// Parses one `playerctl metadata --format` line.
/// Returns `(metadata, player_active)` — active for both Playing and Paused.
pub fn parse_metadata(line: &str) -> (PlayerMeta, bool) {
    let mut meta = PlayerMeta::default();
    let parts: Vec<&str> = line.split("|//|").collect();
    if parts.len() < 5 {
        return (meta, false);
    }

    let status = parts[0].trim();
    meta.title = parts[1].trim().to_string();
    meta.artist = parts[2].trim().to_string();
    meta.player_name_raw = parts[3].trim().to_string();
    meta.art_url = parts[4].trim().to_string();
    if parts.len() >= 7 {
        meta.pos_secs = parts[5].trim().parse::<f64>().unwrap_or(0.0) / 1_000_000.0;
        meta.len_secs = parts[6].trim().parse::<f64>().unwrap_or(0.0) / 1_000_000.0;
    }
    let playing = status.eq_ignore_ascii_case("playing");
    let active = playing || status.eq_ignore_ascii_case("paused");
    meta.playing = playing;
    (meta, active)
}

/// Keeps track identity across a transiently incomplete MPRIS update.
pub fn merge_transient_metadata(previous: &PlayerMeta, mut incoming: PlayerMeta) -> PlayerMeta {
    if previous.title.is_empty() && previous.artist.is_empty() {
        return incoming;
    }
    let same_player = !previous.player_name_raw.is_empty()
        && !incoming.player_name_raw.is_empty()
        && previous
            .player_name_raw
            .eq_ignore_ascii_case(&incoming.player_name_raw);
    if !same_player {
        return incoming;
    }
    match true {
        _ if incoming.title.is_empty() => {
            incoming.title = previous.title.clone();
            incoming.artist = previous.artist.clone();
            incoming.art_url = previous.art_url.clone();
        }
        _ if incoming.title == previous.title => {
            if incoming.artist.is_empty() {
                incoming.artist = previous.artist.clone();
            }
            if incoming.art_url.is_empty() {
                incoming.art_url = previous.art_url.clone();
            }
        }
        _ => {}
    }
    incoming
}

pub fn format_time(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    format!("{}:{:02}", total / 60, total % 60)
}
