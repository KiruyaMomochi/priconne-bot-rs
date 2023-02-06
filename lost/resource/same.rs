// Check if two posts are same.
// Definition of same:
// They have the same id (same resource), title and content.
// If a post is pushed, we don't consider they are same even if they have the same id.
// But such a post maybe updated, when updated, title, content or date may change.

pub enum CompareResult {
    Same,
    Updated,
    Different,
}

impl News {
    pub fn compare(&self, other: &Self) -> CompareResult {
        let same_id = self.id == other.id;
        if !same_id {
            return CompareResult::Different;
        }

        let same_metadata = self.title != other.title
            || self.display_title != other.display_title
            || self.date != other.date;

        if same_metadata {
            CompareResult::Same
        } else {
            CompareResult::Updated
        }
    }
}

impl Announce {
    /// Two announce are same if they has same id
    pub fn compare(&self, other: &Self) -> CompareResult {
        todo!()
    }
}
