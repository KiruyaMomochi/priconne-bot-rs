use priconne_core::Tagger;

pub struct Bot<C: Clone + Send> {
    pub client: C,
    pub mongo_database: mongodb::Database,
    pub telegraph: telegraph_rs::Telegraph,
    pub bot: teloxide::adaptors::AutoSend<teloxide::Bot>,
    pub tagger: Tagger,
}
