use std::fmt::Display;

use ns2_stat::GameSummary;

use crate::helpers;
use crate::skill::{compute_skills, Skill};

/// Suggests teams by solving the [balanced partitioning problem](https://en.wikipedia.org/wiki/Balanced_number_partitioning).
/// The first team is marines and the second is aliens. The first player in each team is the commander.
pub fn balanced_partitioning<'a>(
    players: &'a [String],
    marine_commander: Option<&String>,
    alien_commander: Option<&String>,
    score: impl Fn(&str) -> Skill,
) -> impl Iterator<Item = (Vec<&'a String>, Vec<&'a String>)> {
    // Compute the sums of all possible partitions in an array with 2^n elements.
    // Each possibility is encoded as a bit pattern (the index of the respective sum),
    // where a 0 indicates marines and a 1 indicates aliens.
    let marine_comm_idx = marine_commander.and_then(|comm| players.iter().position(|name| name == comm));
    let alien_comm_idx = alien_commander.and_then(|comm| players.iter().position(|name| name == comm));
    let n = 1 << players.len();
    let mut total_scores = (0usize..n)
        .flat_map(|p| (0..players.len()).flat_map(move |mc| (0..players.len()).map(move |ac| (p, mc, ac, 0.0))))
        .filter(|&(p, mc, ac, _)| {
            usize::abs_diff(players.len(), 2 * p.count_ones() as usize) <= 1 // player difference between teams is <= 1
                && marine_comm_idx.is_none_or(|i| i == mc) // mc matches wanted marine commander
                && alien_comm_idx.is_none_or(|i| i == ac) // ac matches wanted alien commander
                && (p >> mc) & 1 == 0 // marine commander is in marines team
                && (p >> ac) & 1 == 1 // alien commander is in aliens team
        })
        .collect::<Vec<_>>();
    for (i, player) in players.iter().enumerate() {
        let skill = score(player);
        for (p, mc, ac, score) in total_scores.iter_mut() {
            if i == *mc {
                // marine commander
                *score += skill.commander;
            } else if i == *ac {
                *score -= skill.commander;
            } else if (*p >> i) & 1 == 0 {
                // marines
                *score += skill.marine();
            } else {
                // aliens
                *score -= skill.alien();
            }
        }
    }

    total_scores.sort_by(|(_, _, _, score1), (_, _, _, score2)| f32::total_cmp(&score1.abs(), &score2.abs()));
    total_scores.into_iter().map(|(p, mc, ac, _)| {
        let mut marines = Vec::with_capacity(players.len() / 2);
        let mut aliens = Vec::with_capacity(players.len() / 2);
        for (i, player) in players.iter().enumerate() {
            if i == mc {
                marines.insert(0, player);
            } else if i == ac {
                aliens.insert(0, player);
            } else if (p >> i) & 1 == 0 {
                marines.push(player);
            } else {
                aliens.push(player);
            }
        }
        (marines, aliens)
    })
}

/// Analyzes the past games, sorted by length, in descending order.
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

pub enum Method {
    PastGames,
    Skill,
}

/// Print balanced team suggestions.
pub fn suggest_teams(games: Vec<GameSummary>, players: Vec<String>, marine_commander: Option<String>, alien_commander: Option<String>, method: Method) {
    println!("Team suggestions");
    println!("================");
    match method {
        Method::PastGames => {
            analyze_past_games(games, players, marine_commander, alien_commander).take(4).for_each(|game| {
                print_teams(
                    game.marines.players.keys(),
                    game.aliens.players.keys(),
                    |player| game.marines.is_commander(player),
                    |player| game.aliens.is_commander(player),
                );
            });
        }
        Method::Skill => {
            let skills = compute_skills(&games);
            balanced_partitioning(&players, marine_commander.as_ref(), alien_commander.as_ref(), |player| *skills.get(player).unwrap()).take(4).for_each(|(marines, aliens)| {
                print_teams(
                    marines.iter(),
                    aliens.iter(),
                    |player| marines.first().is_some_and(|name| name == player),
                    |player| aliens.first().is_some_and(|name| name == player),
                );
            });
        }
    }
}

fn print_teams<'a, T: Display + 'a, I: Iterator<Item = &'a T>>(
    marines: I,
    aliens: I,
    is_marine_commander: impl Fn(&T) -> bool,
    is_alien_commander: impl Fn(&T) -> bool,
) {
    println!();
    println!(
        "Marines: {}",
        helpers::format_with(marines, ", ", |f, player| if is_marine_commander(player) {
            write!(f, "[{}]", player)
        } else {
            write!(f, "{}", player)
        }),
    );
    println!(
        "Aliens: {}",
        helpers::format_with(aliens, ", ", |f, player| if is_alien_commander(player) {
            write!(f, "[{}]", player)
        } else {
            write!(f, "{}", player)
        }),
    );
}
