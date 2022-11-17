use mongodb::{bson::{Document, doc}, options::{FindOneAndReplaceOptions, FindOneOptions}};
use resource::{information::Announce, message::SentMessage};

impl super::Db {
    pub async fn check_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let found_announce = self.find_announce(announce).await?;
        if let Some(found_announce) = found_announce {
            if found_announce.replace_time == announce.replace_time {
                return Ok(Some(found_announce));
            }
        }

        Ok(None)
    }

    pub async fn check_sent_announce(
        &self,
        announce: &Announce,
    ) -> Result<InformatonResult, mongodb::error::Error> {
        if let Some(announce) = self.check_announce(announce).await? {
            return Ok(InformatonResult::Announce(announce));
        }

        let found_sent = self.find_information_in_sent(announce).await?;
        if let Some(found_sent) = found_sent {
            if found_sent.update_time > announce.replace_time {
                self.upsert_announce(announce).await?;
                self.update_information_in_sent(&found_sent, announce).await?;
                return Ok(InformatonResult::SentAnnounce(found_sent));
            }
            return Ok(InformatonResult::SentNoAnnounce(found_sent));
        }

        Ok(InformatonResult::None)
    }

    pub async fn find_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let collection = self.announces();
        let filter = announce_filter(announce);
        let find_result = collection.find_one(filter.clone(), None).await?;
        Ok(find_result)
    }

    pub async fn upsert_announce(
        &self,
        announce: &Announce,
    ) -> Result<Option<Announce>, mongodb::error::Error> {
        let collection = self.announces();
        let filter = announce_filter(announce);
        let options = Some(FindOneAndReplaceOptions::builder().upsert(true).build());
        let replace_result = collection
            .find_one_and_replace(filter, announce, options)
            .await?;
        Ok(replace_result)
    }
}

pub enum InformatonResult {
    Announce(Announce),
    SentAnnounce(SentMessage),
    SentNoAnnounce(SentMessage),
    None,
}

impl InformatonResult {
    pub fn is_found(&self) -> bool {
        match self {
            InformatonResult::Announce(_) => true,
            InformatonResult::SentAnnounce(_) => true,
            InformatonResult::SentNoAnnounce(_) => false,
            InformatonResult::None => false,
        }
    }

    pub fn is_not_found(&self) -> bool {
        !self.is_found()
    }
}

fn announce_filter(announce: &Announce) -> Document {
    doc! {
        "announce_id": announce.announce_id,
    }
}
