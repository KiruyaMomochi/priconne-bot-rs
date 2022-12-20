#[derive(Debug)]
pub enum UpdateEvent {
    News(News),
    Announce(Announce),
    Cartoon(Thumbnail),
}


enum Action {
    None,
    UpdateOnly,
    Edit,
    Send,
}

impl Action {
    pub fn send(&self) -> bool {
        match self {
            Action::Send => true,
            _ => false,
        }
    }
}