# TODO

## When updating lopdf: re-check the object-stream save workaround

`pdfuse-merging/src/data.rs` (`merge_documents`) saves with:

```rust
let options = SaveOptions {
    use_object_streams: false,
    use_xref_streams: true,
    ..Default::default()
};
```

`use_object_streams` is forced **off** to work around a bug in lopdf 0.39's
writer. With object streams enabled, `create_xref_steam` iterates
`1..xref.size+1` using the `xref.size` captured *before* the object-stream
container objects and the cross-reference-stream object are created. Those
higher-id objects therefore get no xref entry and the emitted `/Index` is
truncated. Because the catalog is packed into an object stream, a strict reader
(including lopdf itself) cannot find it and reports zero pages. `pdfind` /
`pypdf` (via brute-force recovery) surfaced this.

Cost of the workaround: the object *structure* (dictionaries, page tree) is no
longer compressed into object streams. Stream payloads (page content, images,
fonts) are still compressed by `compress()`, so the file-size impact is small.

### Status: FIXED upstream in lopdf 0.45.0

Verified by reproduction (a 200-page doc saved with `use_object_streams: true`
round-trips to 200 pages on 0.45.0, and the file is ~half the size). The fix is
in `writer.rs::create_xref_steam`, which now iterates `1..=xref.max_id()`
instead of the stale `xref.size`:

```rust
// Iterate over the actual highest entry instead of `xref.size`:
for obj_id in 1..=xref.max_id() {
```

The project currently pins lopdf 0.39.0 (see `Cargo.toml`).

### Action on lopdf upgrade

1. Bump lopdf to >= 0.45 (mind API changes across 0.39 -> 0.45; check the merge,
   bookmark, and renumber paths still compile and behave).
2. Set `use_object_streams: true` again in `merge_documents` to restore object
   structure compression.
3. Regenerate a multi-page merged PDF and confirm the page count with `pdfind`
   and a second reader before removing this note.
