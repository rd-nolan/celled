use crate::domain::{MatchCandidate, SourceColumn, TemplateSchema};
use crate::embedding::{cosine_similarity, header_similarity};

/// Floor for first-row content inference. High enough that a random number
/// does not stick to an unrelated template header such as 姓名.
pub const CONTENT_THRESHOLD: f32 = 0.55;
const TOP_K: usize = 3;
const TOKEN_SUBSTRING_SCORE: f32 = 0.93;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentKind {
    Email,
    Date,
    Phone,
    LongId,
}

impl ContentKind {
    /// Generic type labels for the detected value — not column-name synonym pairs.
    pub fn type_tokens(self) -> &'static [&'static str] {
        match self {
            Self::Email => &["email", "mail", "邮箱", "电子邮件", "邮件"],
            Self::Date => &["date", "日期", "年月日", "time", "时间"],
            Self::Phone => &["phone", "mobile", "telephone", "电话", "手机"],
            Self::LongId => &["number", "编号", "identifier"],
        }
    }
}

pub fn first_sample(source: &SourceColumn) -> Option<&str> {
    source
        .sample_values
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
}

pub fn detect_kind(sample: &str) -> Option<ContentKind> {
    let sample = sample.trim();
    if sample.is_empty() {
        return None;
    }
    if looks_like_email(sample) {
        return Some(ContentKind::Email);
    }
    if looks_like_date(sample) {
        return Some(ContentKind::Date);
    }
    if looks_like_phone(sample) {
        return Some(ContentKind::Phone);
    }
    if looks_like_long_id(sample) {
        return Some(ContentKind::LongId);
    }
    None
}

pub fn combo_text(header: &str, sample: &str) -> Option<String> {
    let header = header.trim();
    if header.is_empty() || header.eq_ignore_ascii_case(sample) {
        None
    } else {
        Some(format!("{header} {sample}"))
    }
}

pub fn content_candidates(
    template: &TemplateSchema,
    template_embeddings: &[Vec<f32>],
    free_targets: &[usize],
    sample_vec: &[f32],
    combo_vec: Option<&[f32]>,
    tokens: &[&str],
    token_vecs: &[&[f32]],
) -> Vec<MatchCandidate> {
    let corpus: Vec<String> = template
        .columns
        .iter()
        .map(|col| col.normalized_name.clone())
        .collect();
    let mut scored: Vec<MatchCandidate> = template
        .columns
        .iter()
        .zip(template_embeddings.iter())
        .filter(|(col, _)| free_targets.contains(&col.index))
        .map(|(col, emb)| MatchCandidate {
            template_column_index: col.index,
            template_header: col.name.clone(),
            score: column_content_score(
                &col.normalized_name,
                emb,
                &corpus,
                sample_vec,
                combo_vec,
                tokens,
                token_vecs,
            ),
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(TOP_K);
    scored
}

fn column_content_score(
    header: &str,
    header_vec: &[f32],
    corpus: &[String],
    sample_vec: &[f32],
    combo_vec: Option<&[f32]>,
    tokens: &[&str],
    token_vecs: &[&[f32]],
) -> f32 {
    let mut score = cosine_similarity(sample_vec, header_vec).clamp(0.0, 1.0);
    if let Some(combo) = combo_vec {
        score = score.max(cosine_similarity(combo, header_vec).clamp(0.0, 1.0));
    }
    for (token, token_vec) in tokens.iter().zip(token_vecs.iter()) {
        if token_in_header(token, header) {
            score = score.max(TOKEN_SUBSTRING_SCORE);
        }
        score = score.max(header_similarity(
            token, token_vec, header, header_vec, corpus,
        ));
    }
    score
}

fn token_in_header(token: &str, header: &str) -> bool {
    let token = token.trim();
    let header = header.trim();
    if token.is_empty() || header.is_empty() {
        return false;
    }
    let min_len = if token.chars().all(|c| c.is_ascii()) {
        4
    } else {
        2
    };
    token.chars().count() >= min_len && header.contains(token)
}

fn looks_like_email(sample: &str) -> bool {
    if sample.chars().any(char::is_whitespace) || sample.len() > 254 {
        return false;
    }
    let Some((local, domain)) = sample.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() || local.contains('@') || domain.contains('@') {
        return false;
    }
    let Some((host, tld)) = domain.rsplit_once('.') else {
        return false;
    };
    !host.is_empty()
        && tld.len() >= 2
        && tld.chars().all(|c| c.is_ascii_alphabetic())
        && local
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-' | '%'))
}

fn looks_like_date(sample: &str) -> bool {
    let date_part = sample
        .split_once(['T', 't', ' '])
        .map(|(date, _)| date)
        .unwrap_or(sample);
    let sep = if date_part.matches('-').count() == 2 {
        '-'
    } else if date_part.matches('/').count() == 2 {
        '/'
    } else if date_part.matches('.').count() == 2 {
        '.'
    } else {
        return false;
    };
    let mut parts = date_part.split(sep);
    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    if year.len() != 4 {
        return false;
    }
    let Ok(y) = year.parse::<i32>() else {
        return false;
    };
    let Ok(m) = month.parse::<u32>() else {
        return false;
    };
    let Ok(d) = day.parse::<u32>() else {
        return false;
    };
    (1900..=2100).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d)
}

fn looks_like_phone(sample: &str) -> bool {
    let mut digits = String::new();
    for c in sample.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if matches!(c, '+' | '-' | ' ' | '(' | ')' | '.') {
            continue;
        } else {
            return false;
        }
    }
    let rest = if digits.starts_with("86") && digits.len() == 13 {
        &digits[2..]
    } else {
        digits.as_str()
    };
    rest.len() == 11 && rest.starts_with('1')
}

fn looks_like_long_id(sample: &str) -> bool {
    let len = sample.chars().count();
    (8..=32).contains(&len) && sample.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::{
        detect_kind, looks_like_date, looks_like_email, looks_like_phone, token_in_header,
        ContentKind,
    };
    use crate::embedding::{cosine_similarity, EmbeddingProvider, MockEmbeddingProvider};

    #[test]
    fn detects_generic_email_date_and_phone() {
        assert_eq!(detect_kind("a@b.com"), Some(ContentKind::Email));
        assert_eq!(detect_kind("2024-01-02"), Some(ContentKind::Date));
        assert_eq!(detect_kind("2024/1/2"), Some(ContentKind::Date));
        assert_eq!(detect_kind("13812345678"), Some(ContentKind::Phone));
        assert_eq!(detect_kind("20230101001"), Some(ContentKind::LongId));
        assert_eq!(detect_kind("42"), None);
        assert_eq!(detect_kind("张三"), None);
    }

    #[test]
    fn email_and_date_checks_reject_noise() {
        assert!(!looks_like_email("not-an-email"));
        assert!(!looks_like_email("a@b"));
        assert!(!looks_like_date("2024-13-40"));
        assert!(!looks_like_date("01-02-2024"));
        assert!(!looks_like_phone("12345"));
        assert!(!looks_like_phone("20230101001"));
    }

    #[test]
    fn email_tokens_prefer_email_header_over_name() {
        assert!(token_in_header("邮箱", "邮箱"));
        assert!(token_in_header("email", "email"));
        assert!(token_in_header("日期", "出生日期"));
        assert!(!token_in_header("邮箱", "日期"));
        assert!(!token_in_header("id", "valid"));
    }

    #[test]
    fn email_token_embedding_matches_email_header() {
        let provider = MockEmbeddingProvider;
        let vectors = provider
            .embed(&["邮箱".into(), "姓名".into(), "日期".into()])
            .unwrap();
        let to_self = cosine_similarity(&vectors[0], &vectors[0]);
        let to_name = cosine_similarity(&vectors[0], &vectors[1]);
        let to_date = cosine_similarity(&vectors[0], &vectors[2]);
        assert!(to_self > 0.99);
        assert!(to_self > to_name);
        assert!(to_self > to_date);
    }
}
