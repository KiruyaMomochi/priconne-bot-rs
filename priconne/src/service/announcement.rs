use crate::service::{AnnouncementPage, ResourceMetadata};
use async_trait::async_trait;

use crate::resource::announcement::AnnouncementResponse;

use super::resource::ResourceClient;

#[async_trait]
pub trait AnnouncementClient<M>:
    ResourceClient<M, Response = AnnouncementResponse<Self::Page>>
where
    M: ResourceMetadata,
{
    type Page: AnnouncementPage;
}

impl<M, T, P> AnnouncementClient<M> for T
where
    M: ResourceMetadata,
    T: ResourceClient<M, Response = AnnouncementResponse<P>>,
    P: AnnouncementPage,
{
    type Page = P;
}
