use crate::{database::PostCollection, resource::{post::sources, information::Announce}, Error};

pub struct PostService {
    pub post_collection: PostCollection,
}

impl PostService {
    pub fn new(post_collection: PostCollection) -> Self {
        Self { post_collection }
    }

    pub async fn handle_new_announce(
        &self,
        announce: &Announce,
        api_id: String,
    ) -> Result<(), Error> {
        let source = sources::Source::Announce(api_id.clone());
        let post = self
            .post_collection
            .find(&announce.title.title, announce.announce_id, &source)
            .await?;

        if post.is_none() {
            let (information, node) = self
                .information_service
                .client
                .information(announce.announce_id)
                .await?;
            let telegraph = self
                .telegraph
                .create_page_doms(&information.title, iter::once(node), false)
                .await?;

            // let sender = Post::from_announce(announce, &information, api_id.clone(), &telegraph);
            let sender = self.message_builder.build_message_announce(
                announce,
                &information,
                api_id.clone(),
                &telegraph,
            );
            let post = sender.send(self.bot.clone(), self.chat_id.clone()).await?;
            self.post_collection.upsert(post).await?;
        }
        let post = post.unwrap();

        let action = self.need_update_send_announce(announce, &api_id, post);
        match action {
            Action::Send => todo!(),
            Action::Edit => todo!(),
            Action::UpdateOnly => todo!(),
            Action::None => todo!(),
        }

        Ok(())
    }
}