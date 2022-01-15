use std::collections::HashSet;

use types::{GameStats, PlayerStat};

pub mod types;

pub struct NS2Stats {
    games: Vec<GameStats>,
}

impl NS2Stats {
    pub fn from_dir(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            games: Self::load_data(path)?,
        })
    }

    pub fn player_stats(&self) -> impl Iterator<Item = &PlayerStat> {
        self.games
            .iter()
            .map(|game| game.player_stats.values())
            .flatten()
    }

    pub fn player_names(&self) -> HashSet<&str> {
        self.player_stats()
            .map(|ps| ps.player_name.as_str())
            .collect::<HashSet<_>>()
    }

    fn load_data(path: &str) -> std::io::Result<Vec<types::GameStats>> {
        std::fs::read_dir(path)?
            .flat_map(|res| res.map(|e| e.path()))
            .filter_map(|path| path.is_file().then(|| std::fs::read_to_string(&path)))
            .flatten()
            .map(|path| {
                types::GameStats::from_json(&path)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e}")))
            })
            .collect::<std::io::Result<Vec<_>>>()
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_data_parsable() -> std::io::Result<()> {
        super::NS2Stats::load_data("test_data").map(|_| ())
    }
}
