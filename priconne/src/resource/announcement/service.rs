use crate::{
    client::{MemorizedResourceClient, MetadataFindResult, ResourceClient, ResourceResponse},
    database::AnnouncementCollection,
    insight::AnnouncementPage,
    resource::{sources::AnnouncementSource, Announcement, ResourceMetadata},
    service::{PriconneService, ResourceService},
    Error,
};
use async_trait::async_trait;

use std::fmt::Debug;
use tracing::{debug, instrument, trace};

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
/// In other words, if you [`memorized`](ResourceClient::memorize) a [`AnnouncementClient`], that
/// client implements [`MemorizedAnnouncementClient`] automatically. Beyond that, it makes
/// [`ResourceService`] implemented.
///
/// Note that there are two different collections: one for metadata and one for all announcements.
#[async_trait]
trait MemorizedAnnouncementClient<M: ResourceMetadata>
where
    Self: Sync,
{
    type Page: AnnouncementPage;

    fn source(&self) -> AnnouncementSource;
    fn announcement_collection(&self, priconne: &PriconneService) -> AnnouncementCollection {
        AnnouncementCollection(priconne.database.collection("announcement"))
    }
    /// Fetch latest announcements and compare with metadata collection
    async fn collect_latest_metadatas(&self) -> Result<Vec<MetadataFindResult<M>>, Error>;
    /// Upsert the metadata into the metadata collection
    async fn upsert_metadata(&self, metadata: &M) -> Result<Option<M>, Error>;
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
    async fn collect_latest_metadatas(&self) -> Result<Vec<MetadataFindResult<M>>, Error> {
        self.latests().await
    }
    async fn upsert_metadata(&self, metadata: &M) -> Result<Option<M>, Error> {
        self.upsert(metadata).await
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
    #[instrument(skip_all, fields(source = %self.source()))]
    async fn collect_latests(
        &self,
        _priconne: &PriconneService,
    ) -> Result<Vec<MetadataFindResult<M>>, Error> {
        self.collect_latest_metadatas().await
    }

    /// Add a new information resource to post collection, extract data and send if needed
    /// This is the main entry point of the service
    #[instrument(skip_all, fields(
        source = %self.source(),
        metadata.id = metadata.item().id(),
        metadata.title = metadata.item().title()))]
    async fn work(
        &self,
        priconne: &PriconneService,
        metadata: MetadataFindResult<M>,
    ) -> Result<(), Error>
    where
        M: 'async_trait,
    {
        let source = self.source();
        let announcements = self.announcement_collection(priconne);

        let found = announcements
            .find_resource(metadata.item(), &source)
            .await?;

        let decision = AnnouncementDecision::new(&source, &metadata, &found);

        if !decision.should_request() {
            return Ok(());
        };

        // ask client to get full article
        // maybe other things like thumbnail for cartoon, todo
        let (mut insight, content) = {
            let response = self.fetch_response(metadata.item()).await?;
            let insight = priconne.extractor.extract_announcement(&response);
            let extra = Some(serde_json::to_string_pretty(&insight.extra)?);

            (insight, response.telegraph_content(extra)?)
        };

        // extract data
        if decision.should_telegraph() {
            // TODO: telegraph patch in utils
            let telegraph = priconne
                .telegraph
                .create_page(&insight.title, &content.unwrap(), false)
                .await?;

            insight.telegraph_url = Some(telegraph.url);
        }

        trace!("{insight:?}");
        let announcement = Announcement::new(insight, found);

        if decision.send_post_and_continue() {
            let message = priconne
                .chat_manager
                .send_announcement(&announcement)
                .await?;
            trace!("message sent: {:?}", message.url());
        };

        // TODO: Graceful Shutdown
        self.upsert_metadata(metadata.item()).await?;
        announcements.upsert(&announcement).await?;

        Ok(())
    }

    fn dry_work(&self, metadata: MetadataFindResult<M>) {
        tracing::info!(
            "dry_run: work {} {}",
            metadata.item().kind(),
            metadata.item().title()
        )
    }
}

/// Use information about a resource to find action to take
#[derive(Debug)]
pub struct AnnouncementDecision {
    /// Action to take
    pub action: Action,
    /// Source of item
    pub source: AnnouncementSource,
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

impl AnnouncementDecision {
    pub fn should_request(&self) -> bool {
        !matches!(self.action, Action::None)
    }

    pub fn send_post_and_continue(&self) -> bool {
        matches!(self.action, Action::Send)
    }

    pub fn should_telegraph(&self) -> bool {
        !matches!(self.action, Action::Send)
    }
}

// TODO: random write, may all wrong
impl AnnouncementDecision {
    pub fn new<R: ResourceMetadata + Debug>(
        source: &AnnouncementSource,
        find_result: &MetadataFindResult<R>,
        found: &Option<Announcement>,
    ) -> Self {
        Self {
            action: Self::get_action(source, find_result, found),
            source: source.clone(),
        }
    }

    #[instrument(skip(resource, post))]
    fn get_action<R: ResourceMetadata + Debug>(
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

        debug!("no action needed");
        Action::None
    }
}
