
pub trait PostSource<R>
where
    R: Resource,
{
    fn source(&self) -> Source;
}


pub trait MessageBuilder {
    /// Builds a message
    fn build_message(&self) -> String;
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AnnounceSource {
    pub api: String,
    pub id: i32,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewsSource {
    pub id: i32,
}

impl From<AnnounceSource> for Source {
    fn from(announce: AnnounceSource) -> Self {
        Source::Announce(announce)
    }
}

impl From<NewsSource> for Source {
    fn from(news: NewsSource) -> Self {
        Source::News(news)
    }
}

impl From<&Announce> for Source {
    fn from(announce: &Announce) -> Self {
        Source::Announce(announce.clone())
    }
}

impl From<&News> for Source {
    fn from(news: &News) -> Self {
        Source::News(news.clone())
    }
}


pub fn bson(&self) -> bson::Bson {
    match self {
        Source::Announce(announce) => bson::to_bson(announce).unwrap(),
        Source::News(news) => bson::to_bson(news).unwrap(),
    }
}

#[derive(Debug)]
pub enum PostKind {
    Announce(Announce, String),
    News(News),
}

impl PostKind {
    pub fn into_source(&self) -> sources::Source {
        match self {
            PostKind::Announce(announce, api) => sources::AnnounceSource{api: api.to_string(), id: announce.announce_id}.into(),
            PostKind::News(news) => sources::NewsSource{id: news.id}.into(),
        }
    }

    pub fn title(&self) -> String {
        match self {
            PostKind::Announce(announce, _) => announce.title.title.to_string(),
            PostKind::News(news) => news.title.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PostSources(pub Vec<sources::Source>);
pub fn has_announce(&self) -> bool {
    self.0.iter().any(|s| {
        if let sources::Source::Announce(_) = s {
            true
        } else {
            false
        }
    })
}

pub fn has_news(&self) -> bool {
    self.0.iter().any(|s| {
        if let sources::Source::News(_) = s {
            true
        } else {
            false
        }
    })
}

pub fn announce(&mut self) -> Option<&mut sources::Announce> {
    self.0.iter_mut().find_map(|s| {
        if let sources::Source::Announce(announce) = s {
            Some(announce)
        } else {
            None
        }
    })
}

pub fn upsert_announce(&mut self, announce: sources::Announce) {
    if let Some(announce_index) = self.0.iter().position(|s| {
        if let sources::Source::Announce(_) = s {
            true
        } else {
            false
        }
    }) {
        self.0[announce_index] = announce.into();
    } else {
        self.0.push(announce.into());
    }
}

impl PostSources {
    pub fn new(sources: Vec<sources::Source>) -> Self {
        let mut announce = HashMap::new();
        let mut news = Vec::new();
        for source in sources {
            match source {
                sources::Source::Announce(announcement) => {
                    announce.push(announcement)
                }
                sources::Source::News(news_) => {
                    news.push(news_)
                }
            }
        }

        Self {
            announce,
            news,
        }
    }

    pub fn announce(&mut self) -> Option<(&String, &mut i32)> {
        self.announce.iter_mut().next()
    }

    pub fn replace_announce(&mut self, announce: sources::AnnounceSource) {
        self.announce = vec![announce];
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum PostSource {
    Announce { api: String, id: i32 },
    News { id: i32 },
}

impl PostSource<Announce> for ApiClient {
    fn source(&self) -> crate::resource::post::sources::Source {
        crate::resource::post::sources::Source::Announce(self.api_server.id.clone())
    }
}

impl<R, Client> PostSource<R> for ResourceService<R, Client>
where
    Client: ResourceClient<R> + PostSource<R>,
    R: Resource + Sync,
{
    fn source(&self) -> crate::resource::post::sources::Source {
        self.client.source()
    }
}

impl<R, Client> PostSource<&R> for ResourceService<R, Client>
where
    Client: ResourceClient<R> + PostSource<R>,
    R: Resource,
{
    fn source(&self) -> crate::resource::post::sources::Source {
        self.client.source()
    }
}


pub mod sources {
    impl Soource {
        pub fn to_sources(self, id: i32) -> PostSources {
            match self {
                Source::Announce(api_id) => PostSources::new_announce(api_id, id),
                Source::News => PostSources::new_news(id),
            }
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PostSources {
    pub announce: HashMap<String, i32>,
    pub news: Option<i32>,
}

impl PostSources {
    pub fn has_announce(&self) -> bool {
        !self.announce.is_empty()
    }

    pub fn has_news(&self) -> bool {
        self.news.is_some()
    }

    pub fn new_news(id: i32) -> Self {
        Self {
            announce: HashMap::new(),
            news: Some(id),
        }
    }

    pub fn new_announce(api_id: String, id: i32) -> Self {
        Self {
            announce: HashMap::from([(api_id, id)]),
            news: None,
        }
    }

    pub fn matches(&self, source: &Source) -> Option<i32> {
        match source {
            Source::News => self.news,
            Source::Announce(server_id) => self.announce.get(server_id).map(|id| *id),
        }
    }
}