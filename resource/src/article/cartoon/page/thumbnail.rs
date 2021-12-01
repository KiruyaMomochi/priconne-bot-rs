use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Thumbnail {
    #[serde(deserialize_with = "utils::string_or_i32")]
    pub id: i32,
    pub episode: String,
    pub current_page_id: i32,
    pub page_set: i32,
    pub title: String,
    pub thumbnail: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ThumbnailList(pub Option<Vec<Thumbnail>>);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_thumbnail() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
  "id": "258",
  "episode": "257",
  "current_page_id": 0,
  "page_set": 0,
  "title": "很擅長料理的人",
  "thumbnail": "https://img-pc.so-net.tw/elements/media/cartoon/image/fbd6340391b4421719a47ab1ae38e6cf.png"
}"#;
        let cjson = r#"{
  "id": 258,
  "episode": "257",
  "current_page_id": 0,
  "page_set": 0,
  "title": "很擅長料理的人",
  "thumbnail": "https://img-pc.so-net.tw/elements/media/cartoon/image/fbd6340391b4421719a47ab1ae38e6cf.png"
}"#;
        let thumbnail = Thumbnail{
            thumbnail: "https://img-pc.so-net.tw/elements/media/cartoon/image/fbd6340391b4421719a47ab1ae38e6cf.png".to_owned(),
            current_page_id: 0,
            episode: "257".to_owned(),
            id: 258,
            page_set: 0,
            title: "很擅長料理的人".to_owned()
        };

        let result: Thumbnail = serde_json::from_str(json)?;
        assert_eq!(result, thumbnail);

        let result = serde_json::to_string_pretty(&thumbnail)?;
        assert_eq!(result, cjson);

        Ok(())
    }

    #[test]
    fn test_thumbnail_list() -> Result<(), Box<dyn std::error::Error>> {
        let null_json = r#"null"#;
        let _: ThumbnailList = serde_json::from_str(null_json)?;

        let some_json = r#"[{"id":"244","episode":"243","current_page_id":2,"page_set":0,"title":"\u5373\u8208\u5e03\u4e01\u6f14\u594f","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/c366b598b9739c985955216f6f68ec4e.png"},{"id":"243","episode":"242","current_page_id":2,"page_set":0,"title":"\u7d91\u7d81\u73a9\u6cd5","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/101a3373565a88217356bb1e294cea63.png"},{"id":"242","episode":"241","current_page_id":2,"page_set":0,"title":"\u4e0d\u7531\u81ea\u4e3b\u5730","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/b7dc24bf8d9b00dedbca225b30015086.png"},{"id":"241","episode":"240","current_page_id":2,"page_set":0,"title":"\u5440\u554a\u6355\u9b5a\u6cd5","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/07aa8368ec16daa1c851c9fb3839a44f.png"},{"id":"240","episode":"239","current_page_id":2,"page_set":0,"title":"\u5efa\u570b\u7d00\u5ff5\u65e5","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/2c20054ca094d8ac08204eab8db83b75.png"},{"id":"239","episode":"238","current_page_id":2,"page_set":0,"title":"\u76db\u5927\u4e0d\u5df2\u7684\u6539\u826f","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/fb89809f71e586c995f8a0c73fbcae57.png"},{"id":"238","episode":"237","current_page_id":2,"page_set":0,"title":"\u6700\u73cd\u611b\u7684\u5730\u65b9","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/fef4ebe5a078773a3d8f555ec180d74e.png"},{"id":"237","episode":"236","current_page_id":2,"page_set":0,"title":"\u7e8c\u7bc7\u30fc\u9234\u8393\u7684\u624d\u80fd","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/ab846f3e63178692f88d6ff78a65e095.png"},{"id":"236","episode":"235","current_page_id":2,"page_set":0,"title":"\u7c89\u7d72\u670d\u52d9","thumbnail":"https:\/\/img-pc.so-net.tw\/elements\/media\/cartoon\/image\/d9bd8b986a35a24f269575626bab7487.png"}]"#;
        let _: ThumbnailList = serde_json::from_str(some_json)?;

        Ok(())
    }
}
