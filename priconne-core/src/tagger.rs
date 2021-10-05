use regex::Regex;

#[derive(Debug)]
pub struct Tagger {
    pub tag_rules: Vec<(Regex, String)>,
}

impl Tagger {
    pub fn tag<'a>(&'a self, title: &'a str) -> impl Iterator<Item = String> + 'a {
        self.tag_rules
            .iter()
            .filter(move |(regex, _tag)| regex.is_match(title))
            .map(|(_regex, tag)| tag.to_string())
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

pub fn map_titie(title: &str) -> String {
    let title = title.trim();
    let regex = Regex::new(r#"^\s*(【.+】)?\s*(.+)\s*(\(.+更新\))?\s*$"#).unwrap();
    let title = regex.replace(title, "$2");

    title.to_string()
}
