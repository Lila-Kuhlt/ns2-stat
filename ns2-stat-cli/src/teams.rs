use ns2_stat::GameSummary;

use crate::helpers;

#[allow(dead_code)]
mod balanced_partitioning {
    use ns2_stat::Stat;

    /// Suggests teams by solving the [balanced partitioning problem](https://en.wikipedia.org/wiki/Balanced_number_partitioning).
    /// The first team is marines and the second is aliens.
    pub fn balanced_partitioning<S: AsRef<str>>(players: &[S], score: impl Fn(&str) -> Stat<f32>) -> impl Iterator<Item = (Vec<&str>, Vec<&str>)> {
        // Compute the sums of all possible partitions in an array with 2^n elements.
        // Each possibility is encoded as a bit pattern (the index of the respective sum),
        // where a 0 indicates the 1st team and a 1 indicates the 2nd team.
        let n = 1 << players.len();
        let mut total_scores: Vec<(usize, f32)> = (0..n).map(|p| (p, 0.0)).collect();
        for (i, player) in players.iter().enumerate() {
            let stat = score(player.as_ref());
            for p in 0..n {
                if (p >> i) & 1 == 0 {
                    // marines
                    total_scores[p].1 += stat.marines;
                } else {
                    // aliens
                    total_scores[p].1 -= stat.aliens;
                }
            }
        }

        total_scores.sort_by(|(_, score1), (_, score2)| f32::total_cmp(&score1.abs(), &score2.abs()));
        total_scores
            .into_iter()
            .map(|(p, _)| p)
            .filter(|p| usize::abs_diff(players.len(), 2 * p.count_ones() as usize) <= 1) // the player difference between two teams has to be <= 1
            .map(|p| {
                let mut marines = Vec::with_capacity(players.len() / 2);
                let mut aliens = Vec::with_capacity(players.len() / 2);
                for (i, player) in players.iter().enumerate() {
                    if (p >> i) & 1 == 0 {
                        marines.push(player.as_ref());
                    } else {
                        aliens.push(player.as_ref());
                    }
                }
                (marines, aliens)
            })
    }
}

/// Analyzes the past games, sorted by the length, in descending order.
fn analyze_past_games(
    mut games: Vec<GameSummary>,
    players: Vec<String>,
    marine_commander: Option<String>,
    alien_commander: Option<String>,
) -> impl Iterator<Item = GameSummary> {
    // sort by length in descending order
    games.sort_by(|game1, game2| f32::total_cmp(&game1.round_length, &game2.round_length).reverse());

    games.into_iter().filter(move |game| {
        players.len() == game.marines.players.len() + game.aliens.players.len() // correct amount of players
            && marine_commander.as_ref() == game.marines.commander.as_ref() // marine commander matches
            && alien_commander.as_ref() == game.aliens.commander.as_ref() // alien commander matches
            && players.iter().all(|player| game.marines.players.contains_key(player) || game.aliens.players.contains_key(player)) // all players match
    })
}

/// Print balanced team suggestions.
pub fn suggest_teams(games: Vec<GameSummary>, players: Vec<String>, marine_commander: Option<String>, alien_commander: Option<String>) {
    println!("Team suggestions");
    println!("================");
    analyze_past_games(games, players, marine_commander, alien_commander).take(4).for_each(|game| {
        println!();
        println!(
            "Marines: {}",
            helpers::format_with(game.marines.players.keys(), ", ", |f, player| if game.marines.is_commander(&player) {
                write!(f, "[{}]", player)
            } else {
                write!(f, "{}", player)
            }),
        );
        println!(
            "Aliens: {}",
            helpers::format_with(game.aliens.players.keys(), ", ", |f, player| if game.aliens.is_commander(&player) {
                write!(f, "[{}]", player)
            } else {
                write!(f, "{}", player)
            }),
        );
        println!("({:.3} min, winner: {:?})", game.round_length / 60.0, game.winning_team);
    });
}
