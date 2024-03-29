use crate::error::Error;

use async_trait::async_trait;
use kuchikiki::{traits::TendrilSink, NodeRef};

/// A web page.
#[async_trait]
pub trait Page
where
    Self: Sized,
{
    /// Create a new [`Page`] from [`NodeRef`].
    fn from_document(document: NodeRef) -> Result<Self, Error>;

    /// Create a new [`Page`] from a [`String`] containing the HTML.
    fn from_html(html: String) -> Result<Self, Error> {
        let document = kuchikiki::parse_html().one(html);
        Self::from_document(document)
    }
}
