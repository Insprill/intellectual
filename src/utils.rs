pub fn pretty_format_num(num: i32) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f32 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{}K", num / 1_000)
    } else {
        format!("{num}")
    }
}

pub fn borrowed_u8_eq(a: &u8, b: &u8) -> bool {
    *a == *b
}

/// Gets the path part from a full URL.
/// Will return an empty string if invalid.
///
/// `https://github.com/Insprill` -> `Insprill`
/// `https://github.com/Insprill/` -> `Insprill/`
/// `https://github.com/` -> ` `
/// `github.com/Insprill` -> ` `
pub fn path_from_url(url: &str) -> String {
    let mut matches: u8 = 0;
    url.trim_start_matches(|c| {
        if matches == 3 {
            return false;
        } else if c == '/' {
            matches += 1;
        }
        true
    })
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_from_url_correct_paths() {
        assert_eq!(path_from_url("https://github.com/Insprill"), "Insprill");
        assert_eq!(path_from_url("https://github.com/Insprill/"), "Insprill/");
        assert_eq!(path_from_url("https://github.com/"), String::new());
        assert_eq!(path_from_url("github.com/Insprill"), String::new());
    }
}
