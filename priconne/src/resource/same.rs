// Check if two posts are same.
// Definition of same:
// They have the same id (same resource), title and content.
// If a post is pushed, we don't consider they are same even if they have the same id.
// But such a post maybe updated, when updated, title, content or date may change.

use regex::Regex;

pub enum CompareResult {
    Same,
    Updated,
    Different,
}

// impl News {
//     pub fn compare(&self, other: &Self) -> CompareResult {
//         let same_id = self.id == other.id;
//         if !same_id {
//             return CompareResult::Different;
//         }
// 
//         let same_metadata = self.title != other.title
//             || self.display_title != other.display_title
//             || self.date != other.date;
// 
//         if same_metadata {
//             CompareResult::Same
//         } else {
//             CompareResult::Updated
//         }
//     }
// }

// impl Announce {
//     /// Two announce are same if they has same id
//     pub fn compare(&self, other: &Self) -> CompareResult {
//         todo!()
//     }
// }

/// Create mapped title that not changeed by square bracket or update information.
pub fn map_titie(title: &str) -> String {
    let title = title.trim();
    let regex = Regex::new(r#"^\s*(【.+?】)?\s*(.+?)\s*(\(.+更新\))?\s*$"#).unwrap();
    let title = regex.replace(title, "$2");

    title.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_titie() {
        assert_eq!(
            map_titie("「消耗體力時」主角EXP獲得量1.5倍活動！"),
            "「消耗體力時」主角EXP獲得量1.5倍活動！"
        );
        assert_eq!(
            map_titie("【活動】【喵喵】「消耗體力時」主角EXP獲得量1.5倍活動！"),
            "【喵喵】「消耗體力時」主角EXP獲得量1.5倍活動！"
        );
        assert_eq!(
            map_titie("【活動】「消耗體力時」主角EXP獲得量1.5倍活動！(1/1更新)"),
            "「消耗體力時」主角EXP獲得量1.5倍活動！"
        );
    }
}
