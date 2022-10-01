pub fn pretty_format_num(num: i32) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f32 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{}K", num / 1_000)
    } else {
        format!("{}", num)
    }
}
