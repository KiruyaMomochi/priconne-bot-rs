use crate::{Error, Page};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Glossary(pub HashMap<String, String>);

impl Page for Glossary {
    fn from_document(document: kuchiki::NodeRef) -> Result<(Self, kuchiki::NodeRef), Error> {
        let glossary_node = document
            .select_first(".glossary")
            .map_err(|_| Error::KuchikiError)?;

        let accordions = glossary_node
            .as_node()
            .select(".accordion")
            .map_err(|_| Error::KuchikiError)?;

        let mut hash_map = HashMap::<String, String>::new();

        for accordion in accordions {
            let term = accordion
                .as_node()
                .select_first(".acc-title span")
                .map_err(|_| Error::KuchikiError)?
                .text_contents()
                .trim()
                .to_owned();

            let description = accordion
                .as_node()
                .select_first(".acc-child dt")
                .map_err(|_| Error::KuchikiError)?
                .text_contents()
                .trim()
                .to_owned();

            hash_map.insert(term, description);
        }

        Ok((Glossary(hash_map), glossary_node.as_node().clone()))
    }
}

#[cfg(test)]
mod tests {
    use kuchiki::traits::TendrilSink;

    use super::*;
    use std::path::Path;

    #[test]
    fn test_glossary_from_document() {
        let path = Path::new("tests/glossary.html");
        let document = kuchiki::parse_html().from_utf8().from_file(path).unwrap();
        let glossary = Glossary::from_document(document).unwrap().0;

        assert_eq!(glossary.0.len(), 81);
        assert_eq!(glossary.0["熾炎戰鬼煉獄血盟暗黑團（The‧Order‧Of‧Gehenna‧Immortals）"], "修特帕魯在前世時所隸屬的暗黑騎士團。「暗黑騎士們在過去皆已墮落於罪惡當中。但是為了殲滅要顛覆世界的暗黑存在――《星之審判者》，他們決定使用相同的黑暗力量與其對抗。最後，在與星之審判者的戰鬥中，暗黑騎士們全數滅亡……」（節錄自『冥風戰記』）");
    }
}
