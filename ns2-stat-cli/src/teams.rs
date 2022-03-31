use ns2_stat::NS2Stats;

/// Computes the absolute difference of `a` and `b`.
fn abs_diff(a: usize, b: usize) -> usize {
    if a < b {
        b - a
    } else {
        a - b
    }
}

/// Suggests teams by solving the [balanced partitioning problem](https://en.wikipedia.org/wiki/Balanced_number_partitioning).
/// The `n_suggestions` best suggestions are returned.
fn balanced_partitioning<S: AsRef<str>>(players: &[S], score: impl Fn(&str) -> f32, n_suggestions: usize) -> impl Iterator<Item = (Vec<&str>, Vec<&str>)> {
    // Compute the sums of all possible partitions in an array with 2^n elements.
    // Each possibility is encoded as a bit pattern (the index of the respective sum),
    // where a 0 indicates the 1st team and a 1 indicates the 2nd team.
    let n = 1 << (players.len() - 1); // without loss of generality, the last player always goes into the 1st team, so we can save a bit
    let mut total_scores: Vec<(usize, f32)> = (0..n).map(|p| (p, 0.0)).collect();
    for (i, player) in players.iter().enumerate() {
        let player_score = score(player.as_ref());
        for p in 0..n {
            if (p >> i) & 1 == 0 {
                total_scores[p].1 += player_score;
            } else {
                total_scores[p].1 -= player_score;
            }
        }
    }

    total_scores.sort_by_key(|(_, score)| (score.abs() * 1000.0) as u32);
    total_scores
        .into_iter()
        .map(|(p, _)| p)
        .filter(|p| abs_diff(players.len(), 2 * p.count_ones() as usize) <= 1) // the player difference between two teams has to be <= 1
        .take(n_suggestions)
        .map(|p| {
            let mut team1 = Vec::with_capacity(players.len() / 2);
            let mut team2 = Vec::with_capacity(players.len() / 2);
            for (i, player) in players.iter().enumerate() {
                if (p >> i) & 1 == 0 {
                    team1.push(player.as_ref());
                } else {
                    team2.push(player.as_ref());
                }
            }
            (team1, team2)
        })
}

/// Print balanced team suggestions.
pub fn suggest_teams<S: AsRef<str>>(stats: NS2Stats, players: &[S]) {
    let mut unknown_player = false;
    for player in players {
        let player = player.as_ref();
        if !stats.users.contains_key(player) {
            eprintln!("Error: unknown player `{}`", player);
            unknown_player = true;
        }
    }
    if unknown_player {
        std::process::exit(1);
    }

    println!("Team suggestions");
    println!("================");
    balanced_partitioning(players, |p| stats.users[p].kd, 4).for_each(|(team1, team2)| {
        println!();
        println!("Team 1: {}", team1.join(", "));
        println!("Team 2: {}", team2.join(", "));
    });
}
