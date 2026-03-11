pub fn fuzzy_match<'a>(
    token: &str,
    candidates: &'a [&'a str],
    threshold: usize,
) -> Option<&'a str> {
    let normalized = token.to_lowercase();

    if let Some(exact) = candidates.iter().find(|c| normalized == c.to_lowercase()) {
        return Some(*exact);
    }

    if let Some(prefix) = candidates.iter().find(|c| {
        normalized.starts_with(&c.to_lowercase()) || c.to_lowercase().starts_with(&normalized)
    }) {
        return Some(*prefix);
    }

    if let Some(sub) = candidates
        .iter()
        .find(|c| normalized.contains(&c.to_lowercase()) || c.to_lowercase().contains(&normalized))
    {
        return Some(*sub);
    }

    let mut best: Option<(&str, usize)> = None;
    for candidate in candidates {
        let dist = levenshtein(&normalized, &candidate.to_lowercase());
        match best {
            Some((_, best_dist)) if dist >= best_dist => {}
            _ => best = Some((candidate, dist)),
        }
    }

    best.and_then(|(candidate, dist)| {
        if dist <= threshold {
            Some(candidate)
        } else {
            None
        }
    })
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b_chars.len() + 1]; a_chars.len() + 1];

    for (i, row) in dp.iter_mut().enumerate().take(a_chars.len() + 1) {
        row[0] = i;
    }
    for j in 0..=b_chars.len() {
        dp[0][j] = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[a_chars.len()][b_chars.len()]
}
