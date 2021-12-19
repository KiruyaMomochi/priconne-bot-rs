use kuchiki::{NodeData, NodeRef};
use html5ever::{local_name, ns, namespace_url, QualName};

/// Trim leading space strings from a slibling node
pub fn trim_leading_whitespace(sliblings: kuchiki::iter::Siblings) -> bool {
    for slibling in sliblings {
        match slibling.data() {
            NodeData::Element(element_data) => match element_data.name.local {
                local_name!("br") => slibling.detach(),
                local_name!("div") => {
                    if trim_leading_whitespace(slibling.children()) {
                        return true;
                    }
                    if slibling.children().next().is_none() {
                        slibling.detach();
                    }
                }
                local_name!("p") => {
                    if trim_leading_whitespace(slibling.children()) {
                        return true;
                    }
                    if slibling.children().next().is_none() {
                        slibling.detach();
                    }
                }
                _ => return true,
            },
            NodeData::Text(text) => {
                let mut value = text.borrow_mut();

                *value = value.trim_start().to_string();

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

fn is_end_with_linebreak(node: &NodeRef) -> bool {
    let descendants = node.inclusive_descendants();

    for node in descendants.rev() {
        if let Some(text) = node.as_text() {
            if text.borrow().trim_end().ends_with("\n") {
                return true;
            }
            return false;
        }
        if let Some(element) = node.as_element() {
            if element.name.local == local_name!("br") {
                return true;
            }
        }
    }

    return false;
}

fn is_start_with_linebreak(node: &NodeRef) -> bool {
    let descendants = node.inclusive_descendants();

    for node in descendants {
        if let Some(text) = node.as_text() {
            if text.borrow().trim_start().starts_with("\n") {
                return true;
            }
            return false;
        }
        if let Some(element) = node.as_element() {
            if element.name.local == local_name!("br") {
                return true;
            }
            if element.name.local == local_name!("img") {
                return false;
            }
        }
    }

    return false;
}

pub fn insert_br_after_div(node: &NodeRef) {
    for child in node.children().clone() {
        if let Some(preceding) = child.preceding_siblings().next() {
            if is_end_with_linebreak(&preceding) {
                continue;
            }
            if is_start_with_linebreak(&child) {
                continue;
            }

            if let Some(preceding_element) = preceding.as_element() {
                if let local_name!("div") = preceding_element.name.local {
                    child.insert_before(NodeRef::new_element(
                        QualName::new(None, ns!(html), local_name!("br")),
                        vec![],
                    ));
                }
            }
        }
    }
}