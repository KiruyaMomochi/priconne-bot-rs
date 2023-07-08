use html5ever::{local_name, namespace_url, ns, QualName};
use kuchikiki::{ElementData, NodeData, NodeRef};

/// Trim leading space in nodes, returns true if nodes isn't empty.
///
/// Rescursively traverses the tree and detatch any empty string nodes,
/// `<br>` nodes or empty `<br>`/`<div>`/`<p>` nodes. Trim beginning space of
/// the first non-empty text node.
pub fn trim_leading_whitespace(sliblings: kuchikiki::iter::Siblings) -> bool {
    for slibling in sliblings {
        match slibling.data() {
            NodeData::Element(element_data) => match element_data.name.local.clone() {
                // Detach `<br/>`
                local_name!("br") => slibling.detach(),
                local_name!("div") | local_name!("span") | local_name!("p") => {
                    // Recurse into children
                    let chilren_not_empty = trim_leading_whitespace(slibling.children());

                    // If children is not empty, we are done
                    if chilren_not_empty {
                        return true;
                    }

                    // If children is empty, detach the node
                    if slibling.children().next().is_none() {
                        slibling.detach();
                    }
                }
                // image does not count as content
                // image itself won't be detached because we detatch the parent node
                // only if it doesn't have any children
                local_name!("img") | local_name!("figure") => continue,
                // Other nodes means we have content
                _ => return true,
            },
            NodeData::Text(text) => {
                // For text nodes, trim leading space

                let mut value = text.borrow_mut();
                *value = value.trim_start().to_string();

                // If text is empty, detach the node
                if value.is_empty() {
                    slibling.detach();
                } else {
                    return true;
                }
            }
            _ => continue,
        }
    }

    false
}

/// Trim trailing space in nodes, returns true if nodes is not empty.
pub fn trim_trailing_whitespace(sliblings: kuchikiki::iter::Siblings) -> bool {
    for slibling in sliblings.rev() {
        match slibling.data() {
            NodeData::Element(element_data) => match element_data.name.local.clone() {
                // Detach `<br/>`
                local_name!("br") => slibling.detach(),
                local_name!("div") | local_name!("span") | local_name!("p") => {
                    // Recurse into children
                    let chilren_not_empty = trim_trailing_whitespace(slibling.children());

                    // If children is not empty, we are done
                    if chilren_not_empty {
                        return true;
                    }

                    // If children is empty, detach the node
                    if slibling.children().next().is_none() {
                        slibling.detach();
                    }
                }
                // image does not count as content
                // image itself won't be detached because we detatch the parent node
                // only if it doesn't have any children
                local_name!("img") | local_name!("figure") => continue,
                // Other nodes means we have content
                _ => return true,
            },
            NodeData::Text(text) => {
                // For text nodes, trim trailing space

                let mut value = text.borrow_mut();
                *value = value.trim_end().to_string();

                // If text is empty, detach the node
                if value.is_empty() {
                    slibling.detach();
                } else {
                    return true;
                }
            }
            _ => continue,
        }
    }

    false
}

/// Returns true if the node ends with a newline.
fn is_end_with_linebreak(node: &NodeRef) -> bool {
    let descendants = node.inclusive_descendants();

    // Iterate backwards until we find a text or `<br>`
    for node in descendants.rev() {
        // If we find a text node, check if it ends with a newline
        if let Some(text) = node.as_text() {
            return text
                .borrow()
                .trim_end_matches([' ', '　', '\t'])
                .ends_with('\n');
        }
        // If we find a `<br>` node, return true
        if let Some(element) = node.as_element() {
            if element.name.local == local_name!("br") {
                return true;
            }
            if element.name.local == local_name!("img") {
                return false;
            }
        }
    }

    // If we didn't find a text or `<br>` node, return false
    false
}

/// Returns true if the node starts with a newline.
fn is_start_with_linebreak(node: &NodeRef) -> bool {
    let descendants = node.inclusive_descendants();

    // Iterate until we find a text or `<br>`
    for node in descendants {
        // If we find a text node, check if it starts with a newline
        if let Some(text) = node.as_text() {
            return text
                .borrow()
                .trim_start_matches([' ', '　', '\t'])
                .starts_with('\n');
        }
        // If we find a `<br>` node, return true
        if let Some(element) = node.as_element() {
            if element.name.local == local_name!("br") {
                return true;
            }
            if element.name.local == local_name!("img") {
                return false;
            }
        }
    }

    // If we didn't find a text or `<br>` node, return false
    false
}

/// Remove all nodes matching predicate and pull their children to the parent.
pub fn pull_children<P>(sliblings: kuchikiki::iter::Siblings, predicate: P)
where
    P: Fn(&ElementData) -> bool,
{
    pull_children_inner(sliblings, &predicate)
}

/// Pull first `<image>` or `<figure>` to the root.
pub fn pull_first_image(node: &NodeRef) {
    if let Some(slibing) = node.first_child() {
        if let Some(image) = pull_first_image_inner(slibing) {
            node.prepend(image);
        }
    }
}

fn pull_first_image_inner(node: NodeRef) -> Option<NodeRef> {
    if let Some(element) = node.as_element() {
        match element.name.local.clone() {
            local_name!("div") | local_name!("p") if node.first_child().is_some() => {
                return pull_first_image_inner(node.first_child().unwrap())
            }
            local_name!("img") => return Some(node),
            local_name!("figure") => return Some(node),
            _ => return None,
        }
    }

    None
}

fn pull_children_inner<P>(sliblings: kuchikiki::iter::Siblings, predicate: &P)
where
    P: Fn(&ElementData) -> bool,
{
    for slibling in sliblings {
        if let Some(element) = slibling.as_element() {
            if predicate(element) {
                pull_children_inner(slibling.children(), predicate);
                // Move children to the parent
                for child in slibling.children() {
                    slibling.insert_before(child);
                }
                // Detach the div
                slibling.detach();
            }
        }
    }
}

/// Remove all `<div>` and `<span>` nodes and pull their children to the parent.
pub fn remove_div_span(sliblings: kuchikiki::iter::Siblings) {
    pull_children(sliblings, |element| {
        element.name.local == local_name!("div") || element.name.local == local_name!("span")
    });
}

/// Remove all `<div>` nodes and pull their children to the parent.
pub fn remove_div(sliblings: kuchikiki::iter::Siblings) {
    pull_children(sliblings, |element| {
        element.name.local == local_name!("div")
    });
}

/// Replace last-level `<div>` nodes with `<p>` nodes.
pub fn replace_div_with_p(sliblings: kuchikiki::iter::Siblings) -> bool {
    let mut ret = false;

    // Create a new element: https://github.com/kuchikiki-rs/kuchikiki/issues/60
    for slibling in sliblings {
        if let Some(element) = slibling.as_element() {
            if element.name.local == local_name!("p") {
                ret = true;
            }

            let child_replace = replace_div_with_p(slibling.children());
            if child_replace {
                ret = true
            }

            if element.name.local == local_name!("div") && !child_replace {
                let p =
                    NodeRef::new_element(QualName::new(None, ns!(html), local_name!("p")), None);
                for child in slibling.children() {
                    p.append(child);
                }
                slibling.insert_before(p);
                slibling.detach();
                ret = true;
            }
        }
    }

    ret
}

/// Insert a newline between `<div>` nodes.
pub fn insert_br_between_div(node: &NodeRef) -> u32 {
    let mut count = 0;

    for child in node.children() {
        if let Some(preceding) = child.preceding_siblings().next() {
            // println!("{:?}", preceding);
            if is_end_with_linebreak(&preceding) {
                continue;
            }
            if is_start_with_linebreak(&child) {
                continue;
            }

            match preceding.data() {
                NodeData::Element(element) => {
                    if let local_name!("div") = element.name.local.clone() {
                        count += 1;
                        child.insert_before(NodeRef::new_element(
                            QualName::new(None, ns!(html), local_name!("br")),
                            vec![],
                        ));
                    }
                }
                _ => continue,
            }
        }
    }
    count
}

/// Fix so-net news page.
pub fn fix_sonet_news(section_node: NodeRef) -> NodeRef {
    // First, trim leading whitespace
    trim_leading_whitespace(section_node.children());

    // Then we expect a `<h4>` node with style "display: none;"
    // If we find this node, detach it
    let first_child = section_node.first_child();
    if let Some(child) = first_child {
        if let Some(element) = child.as_element() {
            if let local_name!("h4") = element.name.local.clone() {
                if let Some(style) = element.attributes.borrow().get(local_name!("style")) {
                    if style == "display: none;" {
                        child.detach();
                    }
                }
            }
        }
    }

    section_node
}

/// Wrap `<img>` into `<figure>` and `<figcaption>` if not already wrapped.
pub fn wrap_imgs(section_node: &NodeRef) {
    if let Ok(children) = section_node.select(":not(figure) img") {
        for img in children {
            let figure = NodeRef::new_element(
                QualName::new(None, ns!(html), local_name!("figure")),
                vec![],
            );
            img.as_node().insert_before(figure);
            let figure = img.as_node().previous_sibling().unwrap();

            let figcaption = NodeRef::new_element(
                QualName::new(None, ns!(html), local_name!("figcaption")),
                vec![],
            );
            figure.append(img.as_node().clone());
            figure.append(figcaption);
        }
    }
}

pub fn optimize_for_telegraph(node: NodeRef) -> NodeRef {
    // Fix for So-net news
    let node = fix_sonet_news(node);

    // We do not replace `<div>` with `<p>` because so-net use `<div>` to
    // break lines. (WTF)
    // http://www.princessconnect.so-net.tw/news/newsDetail/1460

    // Insert a new line between other `<div>` nodes
    insert_br_between_div(&node);

    // Remove `<div>`
    remove_div(node.children());

    // Trim leading and trailing whitespace to remove empty `<p>` nodes
    // This should be done before pulling children to the parent.
    trim_leading_whitespace(node.children());
    trim_trailing_whitespace(node.children());

    // Wrap `<img>`
    wrap_imgs(&node);

    // Pull images to the parent
    pull_first_image(&node);

    node
}

#[cfg(test)]
mod tests {
    use html5ever::tendril::TendrilSink;

    use super::*;

    fn information_node() -> NodeRef {
        let message = r#"<div class="messages"><div></div><br/>
<div><img class="fr-dib fr-draggable"
    src="https://img-pc.so-net.tw/elements/media/announce/image/6228b3d3f7117619dcb937af445cbbc6.png"></div>
<div><span style="font-size: 18px;">2022/07/11
    16:00起，下述角色的★6才能開花之姿將會登場。<br><br>■能夠以★6才能開花之姿登場的角色<br>・貪吃佩可（夏日）<br>・可可蘿（夏日）<br>・凱留（夏日）<br>※角色名無特定排序<br>此外，限定角色在進行★6才能開花時所必須消耗的道具如下所述<br><br>■必須的道具一覽<br>・各角色的記憶碎片×25<br>・各角色的純淨的記憶碎片×50<br>・公主寶珠×100<br>※預計於今後登場的限定角色也將預計需消耗相同數量的消耗道具。<br><br>■能夠入手純淨的記憶碎片的冒險<br>・貪吃佩可（夏日）<br>主線冒險
    30-2(VERY HARD)「弗泰拉斷崖・北部」<br><br>・可可蘿（夏日）<br>主線冒險 30-3(VERY HARD)「弗泰拉斷崖・北部」<br><br>・凱留（夏日）<br>主線冒險 31-1(VERY
    HARD)「弗泰拉斷崖・南部」<br><br>※若已讓上述角色才能開花至★5，且裝備上專用裝備的話，也能夠於女神的秘石商店中購買。<br><br>■注意事項<br>1.
    若要將角色「★6才能開花」則需要一定數量的「各角色的記憶碎片」「各角色的純淨記憶碎片」「公主寶珠」。<br>2. 若要將角色「★6才能開花」則必須通關「解放冒險」。<br>3.
    本公告中所刊載的相關內容，有可能不經預告逕行調整。<br><br>※So-net
    營運團隊保有活動最終修改與詮釋之權利，實際內容請以遊戲內資訊為準。活動舉辦日期與內容，均有可能未先告知而逕行調整，確切詳情以實際開放時所述為主。</span></div>
</div>"#;

        kuchikiki::parse_html()
            .one(message)
            .select_first(".messages")
            .unwrap()
            .as_node()
            .clone()
    }

    fn news_node_raw() -> NodeRef {
        let news = r#"    <article class="news_con">
<h2>
    2021.08.24<span class="ac01">活動</span>
            </h2>
<h3>【轉蛋】《精選轉蛋》新角色「克蘿依（聖學祭）」登場！機率UP活動舉辦預告！</h3>
<section>
    <h4 style="display: none;">超異域公主連結☆Re：Dive</h4>
    <p>
        <div><img class="fr-dib fr-draggable" src="https://img-pc.so-net.tw/elements/media/announce/image/9d37e97cc30fd18010a0132064cf5dee.png" alt="" /></div>

                <div>2021/12/20 12:00起，為12月戰隊競賽的模式變更期間。</div>
<div> </div>
<div>■關於模式的變更以及選擇</div>
<div>模式變更為能夠事前變更「戰隊模式」「單人模式」的功能。</div>
<div>並且只有在以下變更期間內，才能由戰隊的「隊長」以及「副隊長」執行。</div>
<div>※變更期間內能夠不限次數地變更模式。</div>
<div> </div>
<div>■模式變更期間</div>
<div>2021/12/20 12:00 ～ 2021/12/25 11:59</div>
<div>※戰隊競賽的模式僅能夠於變更期間內進行變更。</div>
<div>※戰隊模式適用於隸屬同一戰隊的所有成員。</div>
<div>※期間內若未選擇模式，系統則會自動選擇與前次戰隊競賽相同的模式。</div>
<div>※若前次沒有選擇模式，系統則會自動選擇戰隊模式。</div>
<div> </div>
<div>■訓練模式解放期間</div>
<div>2021/12/24 12:00 ～ 2021/12/31 23:59</div>
<div>※訓練模式是一個能夠於戰隊競賽舉辦前，與預定登場的怪物進行測試戰鬥的功能。</div>
<div>※選擇顯示於戰隊競賽主畫面左下方的圖示，即可進行訓練模式。</div>
<div> </div>
<div>■戰隊競賽舉辦期間</div>
<div>2021/12/27 05:00 ～ 2021/12/31 23:59</div>
<div> </div>
<div>■變更方法</div>
<div>能夠依照以下步驟進行設定。</div>
<div>・選擇主頁面的［戰隊］。</div>
<div>・在戰隊競賽的模式變更期間內，選擇［變更］。</div>
<div>・在「模式設定」中選擇［戰隊模式］或［單人模式］。</div>
<div>・選擇［變更］。</div>
<div>・戰隊競賽的模式即會被設定。而在設定後，戰隊成員便能夠在戰隊主畫面，或者是於初次移動至戰隊競賽的主畫面時，接收到最新設定內容的通知。</div>
<div> </div>
<div>■各模式可獲得的獎勵</div>
<div>在戰隊模式的排名獎勵以及單人模式的討伐獎勵中，僅有記憶碎片與前次有所不同。</div>
<div>本次可獲得的記憶碎片為「依里的記憶碎片」。</div>
<div> </div>
<div>■注意事項</div>
<div>1. 想參加戰隊競賽，必須先有所屬戰隊。</div>
<div>戰隊將於通關主線冒險 3-1(NORMAL)後開放。</div>
<div>2. 戰隊競賽的活動期間內，無法進行戰隊的退出、驅逐、解散與模式變更。</div>
<div>3. 戰隊的詳細介紹，請點擊遊戲內的底部選單的［選單］，於［幫助］裡的「戰隊」中進行確認。</div>
<div>4. 戰隊競賽的詳細介紹，請點擊遊戲內的底部選單的［選單］，於［幫助］裡的「戰隊競賽」中進行確認。</div>
<div>5. 戰隊模式的選擇期間、戰隊競賽的舉辦期間及其相關內容，有可能不經預告逕行調整。</div>
<div>6. 若更新未正常反映，請先回到遊戲標題後，再次進行確認。</div>
<div> </div>
<div>※So-net 營運團隊保有活動最終修改與詮釋之權利，實際內容請以遊戲內資訊為準。活動舉辦日期與內容，均有可能未先告知而逕行調整，確切詳情以實際開放時所述為主。</div>
            <div>新角色「克蘿依（聖學祭）」將於精選轉蛋、白金轉蛋及★3必中白金轉蛋中登場！<br />在精選轉蛋中「克蘿依（聖學祭）」的出現機率將獲得提升。<br />此外，舉辦期間內只要在精選轉蛋或★3必中白金轉蛋中獲得對象角色的話，作為「贈品」即可獲得該角色的記憶碎片！<br />舉辦期間內，每次在精選轉蛋或★3必中白金轉蛋中獲得對象角色時，皆能夠獲得贈品。<br />即使透過角色交換Pt來獲得角色，也能夠獲得贈品。<br />※獲得的記憶碎片將不會發送至禮物盒中，而是會直接加算至持有數中。<br />※即使於精選轉蛋、★3必中白金轉蛋以外的轉蛋獲得對象角色，也無法獲得贈品。<br />※角色交換Pt無法繼承至除本次外的轉蛋中。<br />現在「復刻限定角色 獎勵轉蛋」也正同時舉辦中！<br />※紡希（萬聖節）不會出現在「精選轉蛋」中。<br /><br />■角色交換Pt共通的轉蛋<br />進行下列轉蛋即可獲得「克蘿依（聖學祭）」的角色交換Pt。<br />・白金轉蛋<br />・新手衝刺轉蛋<br />・★3必中白金轉蛋<br /><br />■精選轉蛋舉辦期間<br />2021/08/25 16:00 ～ 2021/09/01 15:59<br /><br />■可獲得贈品的對象角色<br />克蘿依（聖學祭）<br /><br />■贈品內容<br />克蘿依（聖學祭）的記憶碎片×100<br /><br />■注意事項<br />1. 「克蘿依（聖學祭）」於精選轉蛋的舉辦期間結束後，也有可能接著在白金轉蛋中出現。<br />2. 「克蘿依（聖學祭）」有可能於新手衝刺轉蛋中登場。<br />※新手衝刺轉蛋中出現的角色，將與開始遊戲的日子所舉辦的轉蛋中所出現的角色相同。<br />3. 關於精選轉蛋、★3必中白金轉蛋、白金轉蛋以及新手衝刺轉蛋的出現機率，請點擊底部選單［轉蛋］中的［詳細］，於［出現機率］分頁內進行確認。<br />4. 2021/08/25 15:59以前停留在轉蛋頁面，並於2021/08/25 16:00以後進行轉蛋的話，有可能會發生錯誤。<br />上述為在轉蛋更新時進行轉蛋所導致的錯誤。此錯誤並不會消耗寶石。<br />5. 「克蘿依（聖學祭）」「克蘿依」為不同角色，因此能夠編組進同一隊伍中。<br />關於同名角色的隊伍編組詳細，請點擊遊戲中底部選單的［選單］，於［幫助］內的「戰鬥」進行確認。<br />6. 精選轉蛋、★3必中白金轉蛋、白金轉蛋以及新手衝刺轉蛋的舉辦期間及其相關內容，有可能不經預告逕行調整。<br />7. 若在超過舉辦期間後才獲得該角色，則無法獲得贈品。<br /><br />※So-net 營運團隊保有活動最終修改與詮釋之權利，實際內容請以遊戲內資訊為準。活動舉辦日期與內容，均有可能未先告知而逕行調整，確切詳情以實際開放時所述為主。</div>
    </p>
</section>
<!-- 頁碼 --> 
<div class="paging">
    <ol>
        <li><a href="javascript: window.history.back();" title="回上一頁">回上一頁</a></li>
        <li><a href="/news" title="回列表首頁">回列表首頁</a></li>
    </ol>
</div>
</article>"#;

        let node = kuchikiki::parse_html()
            .one(news)
            .select_first(".news_con>section")
            .unwrap();

        node.as_node().clone()
    }

    fn news_node_fixed() -> NodeRef {
        fix_sonet_news(news_node_raw())
    }

    const CHECK_OUTPUT: bool = false;
    fn check_output(node: &NodeRef) {
        if CHECK_OUTPUT {
            let mut stdout = std::io::stdout();
            node.serialize(&mut stdout).unwrap();
        }
    }

    #[test]
    fn test_trim_leading_whitespace() -> Result<(), Box<dyn std::error::Error>> {
        let node = news_node_fixed();
        trim_leading_whitespace(node.children());
        check_output(&node);

        Ok(())
    }

    #[test]
    fn test_pull_children() -> Result<(), Box<dyn std::error::Error>> {
        let node = information_node();

        pull_children(node.children(), |element| {
            element.name.local == local_name!("div") || element.name.local == local_name!("span")
        });
        check_output(&node);

        Ok(())
    }

    #[test]
    fn test_fix_sonet_news() -> Result<(), Box<dyn std::error::Error>> {
        let node = news_node_raw();
        let node = fix_sonet_news(node);
        check_output(&node);

        Ok(())
    }

    #[test]
    fn test_wrap_imgs() -> Result<(), Box<dyn std::error::Error>> {
        let node = news_node_fixed();
        wrap_imgs(&node);
        check_output(&node);

        Ok(())
    }

    #[test]
    fn test_pull_first_image() -> Result<(), Box<dyn std::error::Error>> {
        let node = news_node_fixed();
        pull_first_image(&node);
        check_output(&node);

        Ok(())
    }

    #[test]
    fn test_replace_div_with_p() -> Result<(), Box<dyn std::error::Error>> {
        let node = news_node_fixed();
        replace_div_with_p(node.children());
        check_output(&node);

        Ok(())
    }

    #[test]
    fn test_optimize_for_telegraph() -> Result<(), Box<dyn std::error::Error>> {
        let node = news_node_raw();
        let node = optimize_for_telegraph(node);
        check_output(&node);

        Ok(())
    }
}
