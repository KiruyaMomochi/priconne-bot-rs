use crate::{error::Error, page::Page};

#[derive(Debug)]
pub struct CartoonPage {
    pub episode: String,
    pub image_src: String,
}

impl Page for CartoonPage {
    fn from_document(document: kuchiki::NodeRef) -> Result<Self, crate::error::Error> {
        let main_cartoon_node = document
            .select_first(".main_cartoon")
            .map_err(|_| Error::KuchikiError)?;

        let episode = main_cartoon_node
            .attributes
            .borrow()
            .get("data-current")
            .ok_or(Error::KuchikiError)?
            .to_owned();

        let image_node = main_cartoon_node
            .as_node()
            .select_first("img")
            .map_err(|_| Error::KuchikiError)?;
        let image_src = image_node
            .attributes
            .borrow()
            .get("src")
            .ok_or(Error::KuchikiError)?
            .to_owned();

        Ok(Self { episode, image_src })
    }
}
