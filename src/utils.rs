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

/// Gets the path part from a URL. Will panic if the URL doesn't have any '/'.
pub fn path_from_url(url: &str) -> String {
    url.splitn(4, '/').last().unwrap().to_owned()
}
