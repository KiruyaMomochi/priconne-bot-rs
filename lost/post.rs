impl Post {
    pub fn from_information(
        information: &super::article::information::InformationPage,
        telegraph_url: &str,
        api_name: &str,
        api_id: i32,
    ) -> Self {
        let mut tags = LinkedHashSet::new();
        if let Some(icon) = information.icon {
            tags.insert(icon.to_tag().to_string());
        }
        for tag in crate::extract_tag(&information.title) {
            tags.insert_if_absent(tag);
        }

        Self::new(
            information.title.clone(),
            tags,
            information.date.map(|d| d.with_timezone(&chrono::Utc)),
            vec![sources::AnnounceSource {
                api: api_name.to_string(),
                id: api_id,
            }
            .into()],
            information.events.clone(),
            telegraph_url.to_string(),
        )
    }

    pub fn from_announce(
        announce: &Announce,
        information: &InformationPage,
        api_id: String,
        telegraph: &telegraph_rs::Page,
    ) -> NewMessage {
        let mut tags = LinkedHashSet::new();
        Self::insert_tags_announce(&mut tags, information);

        let time = information
            .date
            .map_or(announce.replace_time, |d| d.with_timezone(&chrono::Utc));

        NewMessage {
            title: information.title.clone(),
            source: Source::Announce(api_id),
            id: announce.announce_id,
            create_time: time,
            history: None,
            tags,
            events: information.events,
            telegraph: Some(telegraph.url.to_string()),
            silent: false,
        }

        Self::new(
            information.title.clone(),
            tags,
            time,
            PostSources {
                announce: HashMap::from([(api_id, announce.announce_id)]),
                news: None,
            },
            information.events.clone(),
            telegraph.to_string(),
        )
    }

    fn insert_tags_announce(tags: &mut LinkedHashSet<String>, information: &InformationPage) {
        if let Some(icon) = information.icon {
            tags.insert_if_absent(icon.to_tag().to_string());
        }
        for tag in crate::extract_tag(&information.title) {
            tags.insert_if_absent(tag);
        }
    }

}