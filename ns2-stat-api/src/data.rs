use std::collections::BTreeMap;
use std::path::PathBuf;
use std::{fmt, io};

use fs_err as fs;
use ns2_stat::input_types::GameStats;

#[derive(Debug)]
struct JsonParseError {
    source: serde_json::Error,
    path: PathBuf,
}

impl fmt::Display for JsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse file `{}`", self.path.display())
    }
}

impl std::error::Error for JsonParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub fn load<P: Into<PathBuf>>(path: P) -> io::Result<BTreeMap<u32, GameStats>> {
    fs::read_dir(path)?
        .map(|entry| {
            let path = entry?.path();
            let game = serde_json::from_str::<GameStats>(&fs::read_to_string(&path)?)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, JsonParseError { source: e, path }))?;
            Ok((game.round_info.round_date, game))
        })
        .collect::<io::Result<_>>()
}
