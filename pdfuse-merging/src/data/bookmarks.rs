use std::error::Error;

use lopdf::{Bookmark, Document, Object, ObjectId};

pub(crate) enum BookmarkError{

}

fn extract_bookmarks(doc: &Document) -> Result<Vec<lopdf::Bookmark>, lopdf::Error> {
    let catalog: ObjectId = doc.trailer.get(b"Root").and_then(Object::as_reference)?;
    let catalog_dict: &lopdf::Dictionary = doc.get_object(catalog).and_then(Object::as_dict)?;
    let mut bookmarks: Vec<lopdf::Bookmark> = vec![];
    if let Ok(outlines_ref) = catalog_dict.get(b"Outlines") {
        if let Ok(outlines_id) = outlines_ref.as_reference() {
            if let Ok(mut marks) = parse_outline_node(doc, outlines_id){
                bookmarks.append(&mut marks);
            }
        };
    };
    Ok(bookmarks)
}

fn read_bookmark_entry(entry:&Object,doc: &Document,bookmarks:&mut Vec<Bookmark>)->Result<bool, lopdf::Error>{
    let dict = entry.as_dict()?;
    let id = entry.as_reference()?.0;
    let title = String::from_utf8_lossy(dict.get(b"Title").and_then(Object::as_str)?).to_string();
    // let parent = dict.get(b"Parent").and_then(Object::as_reference)?;
    let format = dict.get(b"F").and_then(Object::as_i64)? as u32;
    let color :Vec<f32> = dict.get(b"C").and_then(Object::as_array)?.iter().map(|f|f.as_f32().unwrap_or(0.0)).collect();
    let page  = dict.get(b"Dest").and_then(Object::as_reference)?;
    let next = dict.get(b"Next").and_then(Object::as_reference).and_then(|r|doc.get_object(r));
    let first = dict.get(b"First").and_then(Object::as_reference).and_then(|r|doc.get_object(r));
    let last = dict.get(b"Last").and_then(Object::as_reference).and_then(|r|doc.get_object(r));
    
    if let Ok(child) = first{
        loop{
            if read_bookmark_entry(child, doc, bookmarks).is_err(){
                break;
            }
        }
        
    }
    let current_bookmark = Bookmark{
        title,
        format,
        color:color.try_into().unwrap_or_else(|_|[0.0,0.0,0.0]),
        page,
        id,
        children: [],
    };
    return Ok(next.is_ok());
}

fn parse_outline_node(
    doc: &Document,
    obj_id: ObjectId,
) -> Result<Vec<lopdf::Bookmark>, lopdf::Error> {
    let mut result = Vec::new();
    let dict = doc.get_object(obj_id)?.as_dict()?;

    if let Ok(mut current_id) = dict.get(b"First").and_then(Object::as_reference) {
        loop {
            let item = doc.get_object(current_id)?.as_dict()?;

            let title = item
                .get(b"Title")
                .and_then(Object::as_str)
                .map(String::from_utf8_lossy);
            let dest = item
                .get(b"Dest")
                .or_else(|| item.get(b"A"))
                .map(|o| format!("{:?}", o));

            let children = match item.get(b"First").and_then(Object::as_reference).copied() {
                Some(child_id) => parse_outline_node(doc, child_id)?,
                None => vec![],
            };
            let current_bookmark = lopdf::Bookmark{
                children: todo!(),
                title,
                format: todo!(),
                color: todo!(),
                page: todo!(),
                id: todo!(),

;            }
            result.push(Bookmark {
                title,
                dest,
                children,
            });

            match item.get(b"Next").and_then(Object::as_reference).copied() {
                Some(next_id) => current_id = next_id,
                None => break,
            }
        }
    }

    Ok(result)
}