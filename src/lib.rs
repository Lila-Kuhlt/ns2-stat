use std::collections::HashSet;

use types::{GameStats, KillFeed, PlayerStat, SteamId};

pub mod types;

pub struct NS2Stats {
    pub games: Vec<GameStats>,
}

impl NS2Stats {
    pub fn from_dir(path: &str) -> std::io::Result<Self> {
        Ok(Self { games: Self::load_data(path)? })
    }

    pub fn all_player_stats(&self) -> impl Iterator<Item = &PlayerStat> {
        self.games.iter().map(|game| game.player_stats.values()).flatten()
    }

    pub fn player_stats<'a>(&'a self, id: &'a SteamId) -> impl Iterator<Item = &'a PlayerStat> {
        self.games.iter().map(|game| game.player_stats.get(id)).flatten()
    }

    pub fn player_names(&self) -> HashSet<&str> {
        self.all_player_stats().map(|ps| ps.player_name.as_str()).collect::<HashSet<_>>()
    }

    pub fn player_ids(&self) -> HashSet<SteamId> {
        self.games
            .iter()
            .map(|game| game.player_stats.keys())
            .flatten()
            .copied()
            .collect::<HashSet<_>>()
    }

    pub fn kill_feed(&self) -> impl Iterator<Item = &KillFeed> {
        self.games.iter().map(|game| game.kill_feed.iter()).flatten()
    }

    pub fn kd(&self, player: u32) -> (u32, u32) {
        let property = |stat: &PlayerStat, property| stat.marines.get(property).unwrap_or(&0.) + stat.aliens.get(property).unwrap_or(&0.);
        let kills: f64 = self.player_stats(&player).map(|ps| property(ps, "kills")).sum();
        let deaths: f64 = self.player_stats(&player).map(|ps| property(ps, "deaths")).sum();
        (kills as u32, deaths as u32)
    }

    fn load_data(path: &str) -> std::io::Result<Vec<types::GameStats>> {
        std::fs::read_dir(path)?
            .flat_map(|res| res.map(|e| e.path()))
            .filter_map(|path| path.is_file().then(|| std::fs::read_to_string(&path)))
            .flatten()
            .map(|path| types::GameStats::from_json(&path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e}"))))
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
