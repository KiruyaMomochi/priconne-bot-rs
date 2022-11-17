

use crate::{Error, Page};

#[derive(Debug)]
pub struct CartoonPage {
    pub id: String,
    pub image_src: String,
}

impl Page for CartoonPage {
    fn from_document(document: kuchiki::NodeRef) -> Result<(Self, kuchiki::NodeRef), Error> {
        let main_cartoon_node = document
            .select_first(".main_cartoon")
            .map_err(|_| Error::KuchikiError)?;

        let episode = main_cartoon_node
            .attributes
            .borrow()
            .get("data-current")
            .ok_or(Error::KuchikiError)?
            .to_owned();

        let image_node = main_cartoon_node
            .as_node()
            .select_first("img")
            .map_err(|_| Error::KuchikiError)?;
        let image_src = image_node
            .attributes
            .borrow()
            .get("src")
            .ok_or(Error::KuchikiError)?
            .to_owned();

        Ok((Self { id: episode, image_src }, main_cartoon_node.as_node().clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuchiki::traits::TendrilSink;
    use crate::Page;

    #[test]
    fn test_from_document() {
        let document = kuchiki::parse_html().one(r#"
<!DOCTYPE html>
<html lang="zh" xml:lang="zh-TW">
<head>
<meta charset="UTF-8">
<meta http-equiv="pragma" content="no-cache">
<meta http-equiv="cache-control" content="no-cache">
<meta http-equiv="expires" content="-1">
<meta name="format-detection" content="telephone=no">
<meta name="viewport" content="width=device-width,user-scalable=no,initial-scale=1.0,user-scalable=no,viewport-fit=cover">
<title></title>
<link rel="stylesheet" type="text/css" href="https://api-pc.so-net.tw/css/common.css?v=20191212132742" media="all">
<link rel="apple-touch-icon-precomposed" href="https://img-pc.so-net.tw/elements/image/ui/icon_bookmark.png?a635">
</head>

<body id="cartoon" >
<div id="top" class="page_container">
<a name="top" id="top"></a>
<div id="detail">
<div class="controls">
<a class="btn_next" href="https://api-pc.so-net.tw/cartoon/detail/255?rnd=291990601"><img src="https://img-pc.so-net.tw/elements/image/cartoon/arrow_r.png"></a>
<a class="btn_prev" href="https://api-pc.so-net.tw/cartoon/detail/253?rnd=112534687"><img src="https://img-pc.so-net.tw/elements/image/cartoon/arrow_l.png"></a>
</div>
<div class="main_cartoon" data-current="254">
<img src="https://img-pc.so-net.tw/elements/media/cartoon/image/2d31832e19fce6b8ca12ca61b99555a9.png">
</div>
<div id="pager_detail"></div>
</div>

<script type="text/template" id="tplPager">
<div class="prev_first"><a class="paging" data-pagesetid="<%= first %>" href="javascript:void(0);">first</a></div>
<div class="prev"><a class="paging" data-pagesetid="<%= older %>" href="javascript:void(0);">older</a></div>
<ul>
<% _.each(pager, function(item){ %>
<li><a class="page<% if(item.current){ %> current<% } %>" href="https://api-pc.so-net.tw/cartoon/detail/<%= item.pageId %>?rnd=396798834"><%= item.indexText %></a></li>
<% }); %>
</ul>
<div class="next"><a class="paging" data-pagesetid="<%= newer %>" href="javascript:void(0);">newer</a></div>
<div class="next_last"><a class="paging" data-pagesetid="<%= latest %>" href="javascript:void(0);">latest</a></div>
</script>
<script type="text/javascript">
window.detail_current_pager = 1;
</script>
<script type="text/template" id="tmpl-fail">
<figure class="network-fail">
<img src="https://img-pc.so-net.tw/elements/image//ui/parts/info_text_network_error.png" class="blank">
</figure>
</script>
</div>
</body>
</html>
        "#);

        let page = CartoonPage::from_document(document).unwrap().0;

        assert_eq!(page.id, "254");
        assert_eq!(
            page.image_src,
            "https://img-pc.so-net.tw/elements/media/cartoon/image/2d31832e19fce6b8ca12ca61b99555a9.png"
        );
    }
}
