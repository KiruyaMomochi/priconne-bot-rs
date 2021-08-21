pub fn cartoon_thumbnail_href(num: i32) -> String {
    format!("cartoon/thumbnail_list/{num}", num = num)
}

pub fn cartoon_pager_top_href(current_page_id: i32, page_set: i32) -> String {
    format!(
        "cartoon/pager/0/{current_page_id}/{page_set}",
        current_page_id = current_page_id,
        page_set = page_set
    )
}

pub fn cartoon_pager_detail_href(current_page_id: i32, page_set: i32) -> String {
    format!(
        "cartoon/pager/1/{current_page_id}/{page_set}",
        current_page_id = current_page_id,
        page_set = page_set
    )
}

pub fn cartoon_detail_href(id: i32) -> String {
    format!("cartoon/detail/{id}", id = id)
}
