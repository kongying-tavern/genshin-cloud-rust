/// 转义 LIKE 通配符（% _ \），防止输入被当作模糊匹配通配符放大（PG 默认 ESCAPE 为反斜杠）。
pub fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn escape_like_escapes_wildcards() {
        // 三个通配符字符各自被转义
        assert_eq!(escape_like(r"a%b_c\d"), r"a\%b\_c\\d");
        // 空串原样返回
        assert_eq!(escape_like(""), "");
        // 不含通配符的输入原样返回
        assert_eq!(escape_like("abc-123"), "abc-123");
    }
}
