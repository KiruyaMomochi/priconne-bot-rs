use kuchiki::NodeRef;

use crate::{Error, Page};

use super::News;

#[derive(Debug)]
pub struct NewsList {
    pub news_list: Vec<News>,
    pub current_page: i32,
    pub prev_href: Option<String>,
    pub next_href: Option<String>,
}

impl Page for NewsList {
    fn from_document(document: NodeRef) -> Result<(Self, NodeRef), Error> {
        let pagging_node = document
            .select_first(".paging")
            .map_err(|_| Error::KuchikiError)?;
        let pagging_node = pagging_node.as_node();

        let page = node_to_page(pagging_node)?;
        let prev_page_href = node_to_prev_href(pagging_node);
        let next_page_href = node_to_next_href(pagging_node);

        let dl_node = document
            .select_first("article")
            .map_err(|_| Error::KuchikiError)?
            .as_node()
            .select_first("dl")
            .map_err(|_| Error::KuchikiError)?;
        let dl_node = dl_node.as_node();
        let result = node_to_news_list(dl_node)?;

        Ok((Self {
            current_page: page,
            news_list: result,
            next_href: next_page_href,
            prev_href: prev_page_href,
        }, document))
    }
}

fn node_to_news_list(dl_node: &NodeRef) -> Result<Vec<News>, Error> {
    let children: Vec<_> = dl_node.children().skip(1).step_by(2).collect();
    let chunks = children.chunks(2);
    let mut result = Vec::<News>::new();
    let news = chunks
        .take_while(|chunk| chunk.len() == 2)
        .map(|chunk| News::from_nodes(&chunk[0], &chunk[1]));
    for news in news {
        result.push(news?);
    }
    Ok(result)
}

fn node_to_next_href(pagging_node: &NodeRef) -> Option<String> {
    let next_page_node = pagging_node.select_first("a[title=下一頁]");
    
    next_page_node.ok().and_then(|last_page_node| {
        last_page_node
            .attributes
            .borrow()
            .get("href")
            .map(|s| s.to_owned())
    })
}

fn node_to_prev_href(pagging_node: &NodeRef) -> Option<String> {
    let prev_page_node = pagging_node.select_first("a[title=上一頁]");
    
    prev_page_node.ok().and_then(|first_page_node| {
        first_page_node
            .attributes
            .borrow()
            .get("href")
            .map(|s| s.to_owned())
    })
}

fn node_to_page(pagging_node: &NodeRef) -> Result<i32, Error> {
    let page = pagging_node
        .select_first(".active")
        .map_err(|_| Error::KuchikiError)?
        .as_node()
        .first_child()
        .ok_or(Error::KuchikiError)?
        .into_text_ref()
        .ok_or(Error::KuchikiError)?
        .borrow()
        .parse::<i32>()?;
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuchiki::traits::TendrilSink;
    use std::path::Path;

    #[test]
    fn test_from_document() {
        let path = Path::new("tests/news_list.html");
        let document = kuchiki::parse_html()
            .from_utf8()
            .from_file(path)
            .unwrap();

        let (result, _) = NewsList::from_document(document).unwrap();
        println!("{:#?}", result);

        assert_eq!(result.current_page, 1);
        assert_eq!(result.news_list.len(), 10);
        assert_eq!(
            result.next_href,
            Some("news?page=2".to_owned())
        );
        assert_eq!(result.prev_href, None);
    }
}
