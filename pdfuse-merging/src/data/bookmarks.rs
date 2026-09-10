use lopdf::{Bookmark, Dictionary, Document, Object, ObjectId};

/// A bookmark extracted from a source document's `/Outlines` tree, with its
/// destination already resolved to a page object id in the document's current
/// object-id space.
#[derive(Debug, Clone)]
pub(crate) struct SourceBookmark {
    pub title: String,
    pub format: u32,
    pub color: [f32; 3],
    pub page: ObjectId,
    pub children: Vec<SourceBookmark>,
}

/// Guard against cyclic `/Next`/`/First` chains in malformed outlines.
const MAX_OUTLINE_NODES: usize = 100_000;

/// Walk the document's `/Outlines` tree and return the top-level bookmarks with
/// resolved destination pages. `fallback_page` is used for any entry whose
/// destination cannot be resolved so the bookmark still lands somewhere.
///
/// Call after any object renumbering that should apply to the returned page
/// ids: the ids are read straight out of `doc`'s current object space.
pub(crate) fn extract_bookmarks(doc: &Document, fallback_page: ObjectId) -> Vec<SourceBookmark> {
    let Some(root) = outline_root(doc) else {
        return vec![];
    };
    let Ok(first) = root.get(b"First").and_then(Object::as_reference) else {
        return vec![];
    };
    let mut budget = MAX_OUTLINE_NODES;
    read_siblings(doc, first, fallback_page, &mut budget)
}

/// Add extracted bookmarks to `output` under `parent`, preserving hierarchy.
/// `add_bookmark` assigns fresh ids, so children are attached to the id their
/// parent was just given.
pub(crate) fn add_to_document(
    output: &mut Document,
    items: &[SourceBookmark],
    parent: Option<u32>,
) {
    for item in items {
        let id = output.add_bookmark(
            Bookmark::new(item.title.clone(), item.color, item.format, item.page),
            parent,
        );
        add_to_document(output, &item.children, Some(id));
    }
}

fn outline_root(doc: &Document) -> Option<&Dictionary> {
    let catalog = doc.trailer.get(b"Root").and_then(Object::as_reference).ok()?;
    let catalog = doc.get_object(catalog).and_then(Object::as_dict).ok()?;
    let outlines = catalog.get(b"Outlines").ok()?;
    doc.dereference(outlines).ok()?.1.as_dict().ok()
}

fn read_siblings(
    doc: &Document,
    first: ObjectId,
    fallback: ObjectId,
    budget: &mut usize,
) -> Vec<SourceBookmark> {
    let mut out = vec![];
    let mut current = Some(first);
    while let Some(id) = current {
        if *budget == 0 {
            break;
        }
        *budget -= 1;

        let Ok(dict) = doc.get_object(id).and_then(Object::as_dict) else {
            break;
        };
        let title = dict
            .get(b"Title")
            .and_then(|o| doc.dereference(o).map(|(_, v)| v))
            .and_then(Object::as_str)
            .map(decode_text_string)
            .unwrap_or_default();
        let format = dict.get(b"F").and_then(Object::as_i64).unwrap_or(0) as u32;
        let page = resolve_destination(doc, dict).unwrap_or(fallback);
        let children = dict
            .get(b"First")
            .and_then(Object::as_reference)
            .ok()
            .map(|child| read_siblings(doc, child, fallback, budget))
            .unwrap_or_default();

        out.push(SourceBookmark {
            title,
            format,
            color: [0.0, 0.0, 0.0],
            page,
            children,
        });
        current = dict.get(b"Next").and_then(Object::as_reference).ok();
    }
    out
}

/// Resolve an outline item's destination to a page object id, following either
/// its `/Dest` or a `/A` GoTo action's `/D`, including named destinations.
fn resolve_destination(doc: &Document, item: &Dictionary) -> Option<ObjectId> {
    if let Ok(dest) = item.get(b"Dest") {
        if let Some(page) = destination_to_page(doc, dest, MAX_OUTLINE_NODES) {
            return Some(page);
        }
    }
    let action = item.get(b"A").ok()?;
    let action = doc.dereference(action).ok()?.1.as_dict().ok()?;
    let dest = action.get(b"D").ok()?;
    destination_to_page(doc, dest, MAX_OUTLINE_NODES)
}

/// A destination is one of: an explicit `[pageRef /Fit ...]` array, a name or
/// string naming an entry in the document's destination tables, or a dict with
/// a `/D` entry holding one of the former.
fn destination_to_page(doc: &Document, dest: &Object, budget: usize) -> Option<ObjectId> {
    if budget == 0 {
        return None;
    }
    let dest = doc.dereference(dest).ok()?.1;
    match dest {
        Object::Array(array) => array.first()?.as_reference().ok(),
        Object::Dictionary(dict) => {
            destination_to_page(doc, dict.get(b"D").ok()?, budget - 1)
        }
        Object::Name(name) => {
            let target = lookup_named_destination(doc, name)?;
            destination_to_page(doc, &target, budget - 1)
        }
        Object::String(bytes, _) => {
            let target = lookup_named_destination(doc, bytes)?;
            destination_to_page(doc, &target, budget - 1)
        }
        _ => None,
    }
}

/// Look up a named destination via the catalog's legacy `/Dests` dictionary and
/// the `/Names` → `/Dests` name tree. Returns an owned copy of the target.
fn lookup_named_destination(doc: &Document, key: &[u8]) -> Option<Object> {
    let catalog = doc.trailer.get(b"Root").and_then(Object::as_reference).ok()?;
    let catalog = doc.get_object(catalog).and_then(Object::as_dict).ok()?;

    if let Ok(dests) = catalog.get(b"Dests").and_then(|d| doc.dereference(d).map(|(_, v)| v)) {
        if let Ok(dict) = dests.as_dict() {
            if let Ok(value) = dict.get(key) {
                return Some(value.clone());
            }
        }
    }

    let names = catalog.get(b"Names").and_then(|n| doc.dereference(n).map(|(_, v)| v)).ok()?;
    let dests_tree = names.as_dict().ok()?.get(b"Dests").ok()?;
    search_name_tree(doc, dests_tree, key, MAX_OUTLINE_NODES)
}

/// Search a PDF name tree node for `key`, recursing through `/Kids`. Node
/// leaves hold `/Names` as a flat `[key value key value ...]` array.
fn search_name_tree(doc: &Document, node: &Object, key: &[u8], budget: usize) -> Option<Object> {
    if budget == 0 {
        return None;
    }
    let node = doc.dereference(node).ok()?.1.as_dict().ok()?;

    if let Ok(names) = node.get(b"Names").and_then(Object::as_array) {
        for pair in names.chunks_exact(2) {
            let name = doc.dereference(&pair[0]).ok()?.1;
            if name.as_str().map(|n| n == key).unwrap_or(false) {
                return Some(pair[1].clone());
            }
        }
    }

    if let Ok(kids) = node.get(b"Kids").and_then(Object::as_array) {
        for kid in kids {
            if let Some(found) = search_name_tree(doc, kid, key, budget - 1) {
                return Some(found);
            }
        }
    }
    None
}

/// Decode a PDF text string: UTF-16BE when it carries the BOM, otherwise treat
/// the bytes as Latin-1 (a superset-compatible best effort for PDFDocEncoding).
fn decode_text_string(bytes: &[u8]) -> String {
    if let [0xFE, 0xFF, rest @ ..] = bytes {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        bytes.iter().map(|&b| b as char).collect()
    }
}
