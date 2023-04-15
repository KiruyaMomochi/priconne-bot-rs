use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Icon {
    Activity = 1,
    Gotcha = 2,
    Special = 3,
    Update = 4,
    Maintaince = 5,
    Info = 6,
    Statement = 7,
}

impl Icon {
    pub fn from_classname(classname: &str) -> Option<Self> {
        match classname {
            "icon_1" => Some(Icon::Activity),
            "icon_2" => Some(Icon::Gotcha),
            "icon_3" => Some(Icon::Special),
            "icon_4" => Some(Icon::Update),
            "icon_5" => Some(Icon::Maintaince),
            "icon_6" => Some(Icon::Info),
            "icon_7" => Some(Icon::Statement),
            _ => None,
        }
    }

    pub fn to_tag(&self) -> &str {
        match self {
            Icon::Activity => "活動",
            Icon::Gotcha => "轉蛋",
            Icon::Special => "特別活動",
            Icon::Update => "更新",
            Icon::Maintaince => "維護",
            Icon::Info => "最新情報",
            Icon::Statement => "問題說明",
        }
    }
}
