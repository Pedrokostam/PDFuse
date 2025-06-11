use std::error::Error;

use lopdf::{Bookmark, Document, Object, ObjectId};
use rayon::vec;

pub(crate) enum BookmarkError {}

struct Output {
    bookmarks: Vec<Bookmark>,
    next_sibling: Option<ObjectId>,
}

pub fn extract_bookmarks(doc: &Document) -> Result<Vec<lopdf::Bookmark>, lopdf::Error> {
    let dereference = |obj: &Object| obj.as_reference().and_then(|x| doc.get_object(x));

    let catalog: ObjectId = doc.trailer.get(b"Root").and_then(Object::as_reference)?;
    let catalog_dict: &lopdf::Dictionary = doc.get_object(catalog).and_then(Object::as_dict)?;
    let outline_root = catalog_dict
        .get(b"Outlines")
        .and_then(dereference)
        .and_then(Object::as_dict); //.and_then(dereference);
                                    // for x in catalog_dict.get(b"Outlines"){
                                    //     println!("{}",String::from_utf8_lossy(x));
                                    // }
    let mut bookmarks = vec![];
    let mut current_node = outline_root
        .and_then(|x| x.get(b"First"))
        .and_then(Object::as_reference)
        .ok();
    while let Some(node) = current_node {
        let current_node_result = read_bookmark_entry(node, doc).inspect_err(|x| println!("{x}"));
        match current_node_result {
            Err(_) => break,
            Ok(Output {
                bookmarks: bookmarks_node,
                next_sibling,
            }) => {
                bookmarks.extend(bookmarks_node);
                current_node = next_sibling;
            }
        }
    }
    Ok(bookmarks)
}

fn read_bookmark_entry<'a>(entry_id: ObjectId, doc: &'a Document) -> Result<Output, lopdf::Error> {
    let dereference = |obj: &Object| obj.as_reference().and_then(|x| doc.get_object(x));
    let entry = doc.get_object(entry_id)?;
    let dict = entry.as_dict().inspect_err(|x| println!("{x}"))?;
    let id = entry_id.0;
    let title = String::from_utf8_lossy(
        dict.get(b"Title")
            .and_then(dereference)
            .and_then(Object::as_str)
            .inspect_err(|x| println!("{x}"))?,
    )
    .to_string();
    // let parent = dict.get(b"Parent").and_then(Object::as_reference)?;
    let format = dict
        .get(b"F")
        .and_then(dereference)
        .and_then(Object::as_i64).unwrap_or(0) as u32;
    // let color: Option<Vec<f32>> = dict
    //     .get(b"C").unwrap_or_else(|_|Object::Array(vec![0,0,0]))
    //     .as_array().ok()
    //     .map(|x|{x.iter().map(|f| f.as_f32().unwrap_or(0.0)).collect()});
    let color = vec![0,0,0];
    let page = dict.get(b"A").and_then(dereference)?;
    let next = dict.get(b"Next").and_then(Object::as_reference);
    let first = dict.get(b"First").and_then(Object::as_reference);
    // let last = dict.get(b"Last").and_then(dereference);

    let mut current_child: Option<ObjectId> = first.ok();
    let mut bookmarks: Vec<Bookmark> = vec![];
    while let Some(child) = current_child {
        let current_child_output = read_bookmark_entry(child, doc);
        match current_child_output {
            Err(_) => break,
            Ok(Output {
                bookmarks: children_bookmarks,
                next_sibling,
            }) => {
                bookmarks.extend(children_bookmarks);
                current_child = next_sibling;
            }
        }
    }
    let current_bookmark = Bookmark {
        title,
        format,
        color: [0.0, 0.0, 0.0],//color.try_into().unwrap_or([0.0, 0.0, 0.0]),
        page:(0,0),
        id,
        children: bookmarks.iter().map(|x| x.id).collect(),
    };
    bookmarks.insert(0, current_bookmark);
    return Ok(Output {
        bookmarks,
        next_sibling: next.ok(),
    });
}

// fn parse_outline_node(
//     doc: &Document,
//     obj_id: ObjectId,
// ) -> Result<Vec<lopdf::Bookmark>, lopdf::Error> {
//     let mut result = Vec::new();
//     let dict = doc.get_object(obj_id)?.as_dict()?;

//     if let Ok(mut current_id) = dict.get(b"First").and_then(Object::as_reference) {
//         loop {
//             let item = doc.get_object(current_id)?.as_dict()?;

//             let title = item
//                 .get(b"Title")
//                 .and_then(Object::as_str)
//                 .map(String::from_utf8_lossy);
//             let dest = item
//                 .get(b"Dest")
//                 .or_else(|| item.get(b"A"))
//                 .map(|o| format!("{:?}", o));

//             let children = match item.get(b"First").and_then(Object::as_reference).copied() {
//                 Some(child_id) => parse_outline_node(doc, child_id)?,
//                 None => vec![],
//             };
//             let current_bookmark = lopdf::Bookmark{
//                 children: todo!(),
//                 title,
//                 format: todo!(),
//                 color: todo!(),
//                 page: todo!(),
//                 id: todo!(),

// ;            }
//             result.push(Bookmark {
//                 title,
//                 dest,
//                 children,
//             });

//             match item.get(b"Next").and_then(Object::as_reference).copied() {
//                 Some(next_id) => current_id = next_id,
//                 None => break,
//             }
//         }
//     }

//     Ok(result)
// }
