use std::collections::{HashMap};

use crate::{error::Error, page::Page};

#[derive(Debug)]
pub struct Glossary(pub HashMap<String, String>);

impl Page for Glossary {
    fn from_document(document: kuchiki::NodeRef) -> Result<Self, Error> {
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

        Ok(Glossary(hash_map))
    }
}
