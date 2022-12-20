struct Information {
    ...
    ...
    ...
}

impl Resource for Information {
    ...
    ...
}

impl PageResource for Information {
    type page = InformationPage;
}

struct InformationPage {
    url: Url
    page: NodeRef,
    content_node: NodeRef,
}

impl Page for InformationPage {
    fn from_document...
}

impl PostPage for InformationPage {
    Data = InformationData;

    fn title() -> Result<String, Error>;
    fn date() -> Option<DateTime<FixedOffset>>;
    fn data() -> Self::Data;

    fn telegraph() -> (title, content);

    fn insight() -> PostData;
}