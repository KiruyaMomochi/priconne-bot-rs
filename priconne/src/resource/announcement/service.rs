use crate::{
    client::{MemorizedResourceClient, MetadataFindResult, ResourceClient, ResourceResponse},
    database::AnnouncementCollection,
    insight::{AnnouncementInsight, AnnouncementPage, EventPeriod},
    resource::{sources::AnnouncementSource, Announcement, ResourceMetadata},
    service::{PriconneService, ResourceService},
    Error,
};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tracing::{debug, trace};

use crate::resource::announcement::AnnouncementResponse;

/// Clients that can fetch [`AnnouncementResponse`] need to implement this trait
/// to privide what source they are fetching from.
pub trait AnnouncementClient<M>:
    ResourceClient<M, Response = AnnouncementResponse<Self::Page>>
where
    M: ResourceMetadata,
{
    // TODO: is it possible to provide a default implementation?
    type Page: AnnouncementPage;
    fn source(&self) -> AnnouncementSource;
}

/// Auto-implemented extension for [`MemorizedResourceClient`] that implements [`AnnouncementClient`]
#[async_trait]
pub trait MemorizedAnnouncementClient<M: ResourceMetadata>
where
    Self: Sync,
{
    type Page: AnnouncementPage;

    fn source(&self) -> AnnouncementSource;
    fn announcement_collection(&self, priconne: &PriconneService) -> AnnouncementCollection {
        AnnouncementCollection(priconne.database.collection("announcement"))
    }
    async fn collect_latest_announcements(&self) -> Result<Vec<MetadataFindResult<M>>, Error>;
    async fn fetch_response(&self, metadata: &M)
        -> Result<AnnouncementResponse<Self::Page>, Error>;
}

#[async_trait]
impl<M, Client> MemorizedAnnouncementClient<M> for MemorizedResourceClient<M, Client>
where
    M: ResourceMetadata,
    Client: AnnouncementClient<M> + ResourceClient<M>,
    Client::Page: AnnouncementPage,
{
    type Page = Client::Page;

    fn source(&self) -> AnnouncementSource {
        self.client.source()
    }
    async fn collect_latest_announcements(&self) -> Result<Vec<MetadataFindResult<M>>, Error> {
        self.latests().await
    }
    async fn fetch_response(
        &self,
        metadata: &M,
    ) -> Result<<Self as ResourceClient<M>>::Response, Error> {
        self.fetch(metadata).await
    }
}

#[async_trait]
impl<M, T> ResourceService<MetadataFindResult<M>> for T
where
    M: ResourceMetadata,
    T: MemorizedAnnouncementClient<M>,
{
    /// Collect latest metadata
    async fn collect_latests(
        &self,
        _priconne: &PriconneService,
    ) -> Result<Vec<MetadataFindResult<M>>, Error> {
        self.collect_latest_announcements().await
    }

    /// Add a new information resource to post collection, extract data and send if needed
    /// This is the main entry point of the service
    async fn work(
        &self,
        priconne: &PriconneService,
        metadata: MetadataFindResult<M>,
    ) -> Result<(), Error>
    where
        M: 'async_trait,
    {
        let source = self.source();

        let announcement = self
            .announcement_collection(priconne)
            .find_resource(metadata.item(), &source)
            .await?;

        let mut decision = AnnouncementDecision::new(source.clone(), metadata, announcement);

        let Some(metadata) = decision.should_request() else {return Ok(());};

        // ask client to get full article
        // maybe other things like thumbnail for cartoon, todo
        let (mut insight, events, content) = {
            let response = self.fetch_response(metadata).await?;
            let (insight, events) = priconne.extractor.extract_announcement(&response);
            let extra = Some(serde_json::to_string_pretty(&insight.extra)?);

            (insight, events, response.telegraph_content(extra)?)
        };

        // extract data
        // TODO: telegraph patch in utils
        let telegraph = priconne
            .telegraph
            .create_page(&insight.title, &content.unwrap(), false)
            .await?;

        insight.telegraph_url = Some(telegraph.url);

        trace!("{insight:?}");
        if let Some(announcement) = decision.update_announcement(insight, events) {
            self.announcement_collection(priconne)
                .upsert(announcement)
                .await?;
        };

        if let Some(announcement) = decision.send_post_and_continue() {
            let message = priconne.chat_manager.send_post(announcement).await?;
        };

        Ok(())
    }
}

/// Use information about a resource to find action to take
#[derive(Debug)]
pub struct AnnouncementDecision<R: ResourceMetadata> {
    /// Action to take
    pub action: Action,
    /// Source of item
    pub source: AnnouncementSource,
    /// Item in database before fetch
    pub resource: MetadataFindResult<R>,
    /// Post item
    pub announcement: Option<Announcement>,
}

/// Action to take
#[derive(Debug)]
pub enum Action {
    /// Do nothing, just return
    None,
    /// Update post, but do not send or edit message
    UpdateOnly,
    /// Update post and send message
    Send,
    /// Update post and edit message
    Edit,
}

impl<R: ResourceMetadata + Debug> AnnouncementDecision<R> {
    pub fn should_request(&self) -> Option<&R> {
        match self.action {
            Action::None => None,
            _ => Some(self.resource.item()),
        }
    }

    pub fn update_announcement<E>(
        &mut self,
        insight: AnnouncementInsight<E>,
        events: Vec<EventPeriod>,
    ) -> Option<&Announcement>
    where
        E: Serialize + DeserializeOwned,
    {
        match self.announcement.as_mut() {
            Some(post) => {
                post.push(insight, events);
            }
            None => self.announcement = Some(Announcement::new(insight, events)),
        };
        self.announcement.as_ref()
    }

    pub fn send_post_and_continue(&self) -> Option<&Announcement> {
        match self.action {
            Action::Send => self.announcement.as_ref(),
            _ => None,
        }
    }
}

/// TODO: random write, may all wrong
impl<R: ResourceMetadata + Debug> AnnouncementDecision<R> {
    pub fn new(
        source: AnnouncementSource,
        find_result: MetadataFindResult<R>,
        announcement: Option<Announcement>,
    ) -> Self {
        Self {
            action: Self::get_action(&source, &find_result, &announcement),
            source,
            resource: find_result,
            announcement,
        }
    }

    fn get_action(
        source: &AnnouncementSource,
        resource: &MetadataFindResult<R>,
        post: &Option<Announcement>,
    ) -> Action {
        let resource = resource.item();
        if post.is_none() {
            debug!("old post is None. creating a new one");
            return Action::Send;
        }
        let post = post.as_ref().unwrap();

        // find resource with same source
        let same_source = post.data.iter().any(|p| &p.source == source);
        if !same_source {
            debug!("old post does not contain current source. update the post without posting");
            return Action::UpdateOnly;
        };

        // find resource with same id
        let found_resource = post
            .data
            .iter()
            .rev()
            .find(|p| &p.source == source && p.id == resource.id());
        if found_resource.is_none() {
            debug!("old post has a different source id. edit the old message");
            return Action::Edit;
        }
        let found_resource = found_resource.unwrap();

        let update_time = found_resource.update_time.or(found_resource.create_time);
        if update_time.is_some() && resource.update_time() > update_time.unwrap() {
            debug!("old post has same source, but current one is newer. edit the old message");
            return Action::Edit;
        }

        debug!("old post has same source, but current one is newer. edit the old message");
        Action::None
    }
}
