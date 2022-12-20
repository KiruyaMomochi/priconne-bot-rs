use linked_hash_set::LinkedHashSet;
use regex::Regex;
use crate::utils::SplitPrefix;

/// A tagger that can tag a text with a list of strings.
pub trait Tagger {
    /// Tags `text` with a list of strings.
    fn tag(&self, text: &str) -> Vec<String>;
}

#[derive(Clone, Debug)]
/// A tagger that use regular expression as rules, and can tag a text with an iterator of strings.
pub struct RegexTagger {
    pub tag_rules: Vec<(Regex, String)>,
}

impl RegexTagger {
    /// Use given regular expressions to tag `text`.
    pub fn tag_iter<'a>(&'a self, title: &'a str) -> impl Iterator<Item = String> + 'a {
        self.tag_rules
            .iter()
            .filter(move |(regex, _tag)| regex.is_match(title))
            .map(|(_regex, tag)| tag.to_string())
    }

    pub fn tag_title<'a>(&self, title: &'a str) -> LinkedHashSet<String> {
        let mut title = title;
        let mut tags = LinkedHashSet::new();

        if let Some((category, base_title)) = title.split_prefix('【', '】') {
            title = base_title;
            tags.insert(category.to_string());
        }
    
        tags.extend(self.tag_iter(title));
        tags.extend(extract_tag(&title));
        tags
    }
}

impl Tagger for RegexTagger {
    /// Use given regular expressions to tag `text`.
    fn tag(&self, text: &str) -> Vec<String> {
        self.tag_iter(text).collect()
    }
}

#[macro_export]
macro_rules! tag_rule {
    ($regex:expr => $tag:expr) => {
        (regex::Regex::new($regex).unwrap(), $tag.to_string())
    };
    ($tag:expr) => {
        (regex::Regex::new($tag).unwrap(), $tag.to_string())
    };
}

#[macro_export]
macro_rules! tagger {
    ($($regex:expr $(=> $tag:expr)?),*) => {
        $crate::message::Tagger {
            tag_rules: vec![
            $(
                $crate::tag_rule!($regex $(=> $tag)?),
            )*
        ]
        }
    };
}


/// Returns contents quoted by parenthesis, and their positions.
///
/// Nesting is not supported, and nesting same parenthesis may cause
/// unexpected results.
pub fn extract_quote(string: &str) -> Vec<(usize, String)> {
    const QUOTES: [(char, char); 5] = [
        ('【', '】'),
        ('「', '」'),
        ('（', '）'),
        ('(', ')'),
        ('《', '》'),
    ];

    let mut end_char = None;
    let mut start_byte = 0;
    let mut result = Vec::new();

    for (current_byte, ch) in string.char_indices() {
        if end_char == None {
            for &(start, end) in &QUOTES {
                if ch == start {
                    // start_idx = idx;
                    start_byte = current_byte + ch.len_utf8();
                    end_char = Some(end);
                    break;
                }
            }
        } else if ch == end_char.unwrap() {
            end_char = None;
            let quote = string[start_byte..current_byte].to_string();
            result.push((start_byte, quote));
        }
    }

    result
}

/// Extract tags from title quoted by parenthesis, and their positions. 
/// 
/// Nesting is not supported, and nesting same parenthesis may cause
/// unexpected results. This function differs from [extract_quote] in that
/// it's result is trimmed so that it does not contain whitespace or other
/// punctuation.
pub fn extract_tag(string: &str) -> Vec<String> {
    let mut quotes = extract_quote(string);
    for (_, quote) in quotes.iter_mut() {
        if let Some(wh) = quote.find(char::is_whitespace) {
            *quote = quote[..wh].to_string();
        }
        if let Some(br) = quote.find(['（', '(']) {
            *quote = quote[..br].to_string();
        }
    }

    quotes.into_iter().map(|(_, quote)| quote).collect()
}

pub fn message_title(title: &str) -> (Option<String>, String) {
    if let Some((category, base_title)) = title.split_prefix('【', '】') {
        (Some(category.to_string()), base_title.to_string())
    }
    else {
        (None, title.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_quote() {
        let quotes = extract_quote(
            "【轉蛋】《公主祭典 獎勵轉蛋》★3「蘭法（新年）」期間限定角色登場！舉辦預告！",
        );
        assert_eq!(
            quotes,
            vec![
                (3, "轉蛋".to_string()),
                (15, "公主祭典 獎勵轉蛋".to_string()),
                (50, "蘭法（新年）".to_string())
            ]
        );
    }

    #[test]
    fn test_extract_tag() {
        let quotes = extract_tag(
            "【轉蛋】《公主祭典 獎勵轉蛋》★3「蘭法（新年）」期間限定角色登場！舉辦預告！",
        );
        assert_eq!(
            quotes,
            vec![
                "轉蛋".to_string(),
                "公主祭典".to_string(),
                "蘭法".to_string()
            ]
        );
    }
}
