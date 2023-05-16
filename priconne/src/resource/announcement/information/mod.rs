mod page;
use async_trait::async_trait;
pub use page::*;

use crate::{
    service::{
        api::ApiClient,
        resource::{MemorizedResourceClient, MetadataFindResult, ResourceClient},
        AnnouncementService,
    },
    Error,
};

use super::{sources::AnnouncementSource, AnnouncementResponse};


#[async_trait]
impl AnnouncementService<Announce> for MemorizedResourceClient<Announce, ApiClient> {
    type Page = InformationPage;

    fn source(&self) -> AnnouncementSource {
        AnnouncementSource::Api(self.client.api_server.id.clone())
    }
    async fn collect_latest_announcements(
        &self,
    ) -> Result<Vec<MetadataFindResult<Announce>>, Error> {
        self.latests().await
    }
    async fn fetch_response(
        &self,
        metadata: &Announce,
    ) -> Result<AnnouncementResponse<Self::Page>, Error> {
        self.fetch(metadata).await
    }
}

#[cfg(test)]
mod tests {
    use kuchiki::{traits::TendrilSink, NodeRef};

    // #[tokio::test]
    // pub async fn test_news_page() -> Result<(), Box<dyn std::error::Error>> {
    //     let client = crate::Client::new(
    //         "http://www.princessconnect.so-net.tw/",
    //         "https://api-pc.so-net.tw/",
    //     )?;

    //     let ph = telegraph_rs::Telegraph::new("redive-test")
    //         .author_url("https://t.me/pcrtw")
    //         .access_token("02b8a200015d5d9a6301fda3d086ad378b63f491fa3407cfe6921c809e53")
    //         .create()
    //         .await?;

    //     // println!()

    //     let (page, content) = client.information(1807).await?;

    //     // let mut stdout = std::io::stdout();
    //     // content.serialize(&mut stdout).unwrap();

    //     let img = content.select_first("img").unwrap();
    //     {
    //         let mut attrs = img.attributes.borrow_mut();
    //         let src = attrs.get_mut("src").unwrap();
    //         *src = "https://telegra.ph/file/5ba3115fd260b31c2c6b2.png".to_string();
    //     }

    //     // utils::replace_div_with_p(content.children());
    //     // utils::remove_div_span(content.children());
    //     // // utils::insert_br_between_div(&content);
    //     // utils::wrap_imgs(&content);
    //     // utils::pull_first_image(&content);

    //     let content = utils::optimize_for_telegraph(content);

    //     let mut stdout = std::io::stdout();
    //     content.serialize(&mut stdout).unwrap();

    //     // let art = ph.create_page_doms("test", content.children(), false).await?;
    //     // println!("{}", art.url);

    //     Ok(())
    // }

    #[tokio::test]
    pub async fn test_temp() {
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

            let node = kuchiki::parse_html()
                .one(news)
                .select_first(".news_con>section")
                .unwrap();

            node.as_node().clone()
        }

        let ph = telegraph_rs::Telegraph::new("redive-test")
            .author_url("https://t.me/pcrtw")
            .access_token("02b8a200015d5d9a6301fda3d086ad378b63f491fa3407cfe6921c809e53")
            .create()
            .await
            .unwrap();

        let node = news_node_raw();
        let node = crate::utils::optimize_for_telegraph(node);
        // let mut stdout = std::io::stdout();
        // node.serialize(&mut stdout).unwrap();

        let page = ph
            .create_page_doms("test", node.children(), false)
            .await
            .unwrap();
        println!("{}", page.url);
    }
}
