use crate::error::Error;

use async_trait::async_trait;
use kuchiki::{traits::TendrilSink, NodeRef};

#[async_trait]
pub trait Page
where
    Self: Sized,
{
    fn from_document(document: NodeRef) -> Result<Self, Error>;

    fn from_html(html: String) -> Result<Self, Error> {
        let document = kuchiki::parse_html().one(html);
        Self::from_document(document)
    }
}
