use ns2_stat::input_types::{GameStats, WinningTeam};

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

struct PastGame<'a> {
    length: f32,
    winner: WinningTeam,
    marines: Team<'a>,
    aliens: Team<'a>,
}

struct Team<'a> {
    commander: &'a str,
    players: Vec<&'a str>,
}

/// Analyzes the past games, sorted by the length, in descending order.
fn analyze_past_games<'a, I, S, S1, S2>(
    games: I,
    players: &'a [S],
    marine_commander: Option<S1>,
    alien_commander: Option<S2>,
) -> impl Iterator<Item = PastGame<'a>>
where
    I: Iterator<Item = &'a GameStats>,
    S: AsRef<str>,
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let mut summarized_games: Vec<_> = games
        .map(|game| {
            let mut marines = Team {
                commander: "",
                players: Vec::new(),
            };
            let mut marine_com_time = 0.0;
            let mut aliens = Team {
                commander: "",
                players: Vec::new(),
            };
            let mut alien_com_time = 0.0;

            for player_stat in game.player_stats.values() {
                if player_stat.marines.time_played >= player_stat.aliens.time_played {
                    marines.players.push(&player_stat.player_name);
                    if player_stat.marines.commander_time > marine_com_time {
                        marine_com_time = player_stat.marines.commander_time;
                        marines.commander = &player_stat.player_name;
                    }
                } else {
                    aliens.players.push(&player_stat.player_name);
                    if player_stat.aliens.commander_time > alien_com_time {
                        alien_com_time = player_stat.aliens.commander_time;
                        aliens.commander = &player_stat.player_name;
                    }
                }
            }

            PastGame {
                length: game.round_info.round_length,
                winner: game.round_info.winning_team,
                marines,
                aliens,
            }
        })
        .collect();
    // sort `summarized_games` by length in descending order
    summarized_games.sort_by(|game1, game2| f32::total_cmp(&game1.length, &game2.length).reverse());

    summarized_games.into_iter().filter(move |game| {
        players.len() == game.marines.players.len() + game.aliens.players.len() // correct amount of players
            && marine_commander.iter().all(|player| player.as_ref() == game.marines.commander) // marine commander matches
            && alien_commander.iter().all(|player| player.as_ref() == game.aliens.commander) // alien commander matches
            && players.iter().all(|player| game.marines.players.contains(&player.as_ref()) || game.aliens.players.contains(&player.as_ref())) // all players match
    })
}

/// Print balanced team suggestions.
pub fn suggest_teams<'a, I, S, S1, S2>(games: I, players: &'a [S], marine_commander: Option<S1>, alien_commander: Option<S2>)
where
    I: Iterator<Item = &'a GameStats>,
    S: AsRef<str>,
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    println!("Team suggestions");
    println!("================");
    analyze_past_games(games, players, marine_commander, alien_commander).take(4).for_each(|game| {
        println!();
        println!(
            "Marines: {}",
            helpers::format_with(game.marines.players.into_iter(), ", ", |f, player| if player == game.marines.commander {
                write!(f, "[{}]", player)
            } else {
                write!(f, "{}", player)
            }),
        );
        println!(
            "Aliens: {}",
            helpers::format_with(game.aliens.players.into_iter(), ", ", |f, player| if player == game.aliens.commander {
                write!(f, "[{}]", player)
            } else {
                write!(f, "{}", player)
            }),
        );
        println!("({:.3} min, winner: {:?})", game.length / 60.0, game.winner);
    });
}
