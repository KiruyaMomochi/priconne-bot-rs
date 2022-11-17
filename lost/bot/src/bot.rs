use priconne_core::RegexTagger;

pub struct Bot<C: Clone + Send> {
    pub client: C,
    pub database: db::Db,
    pub telegraph: telegraph_rs::Telegraph,
    pub bot: teloxide::adaptors::AutoSend<teloxide::Bot>,
    pub tagger: RegexTagger,
}
