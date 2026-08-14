/// Light header normalization. Intentionally conservative: keep semantic tokens.
pub fn normalize_header(input: &str) -> String {
    let mut s = input.trim().replace(['\n', '\r', '\t'], " ");
    while s.contains("  ") {
        s = s.replace("  ", " ");
    }
    s = s.replace('*', "");
    s = s.replace(['：', ':'], "");

    const NOTES: &[&str] = &[
        "（必填）",
        "(必填)",
        "（必填）",
        "【必填】",
        "[必填]",
        "（选填）",
        "(选填)",
        "（可选）",
        "(可选)",
        "（必填项）",
        "(必填项)",
    ];
    for note in NOTES {
        s = s.replace(note, "");
    }

    s = s.trim().to_string();
    if s.ends_with("必填") && s.chars().count() > 2 {
        s = s.trim_end_matches("必填").trim().to_string();
    }

    s.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::normalize_header;

    #[test]
    fn trims_and_strips_required_markers() {
        assert_eq!(normalize_header(" 手机号码 "), "手机号码");
        assert_eq!(normalize_header("手机号码（必填）"), "手机号码");
        assert_eq!(normalize_header("手机号*"), "手机号");
        assert_eq!(normalize_header("联系电话："), "联系电话");
        assert_eq!(normalize_header("Name"), "name");
    }

    #[test]
    fn does_not_strip_meaningful_parentheses() {
        assert_eq!(normalize_header("手机号码（公司）"), "手机号码（公司）");
    }
}
