use crate::message::Tagger;

pub struct Bot<C: Clone + Send> {
    pub(crate) client: C,
    pub(crate) mongo_database: mongodb::Database,
    pub(crate) telegraph: telegraph_rs::Telegraph,
    pub(crate) bot: teloxide::adaptors::AutoSend<teloxide::Bot>,
    pub(crate) tagger: Tagger,
}
