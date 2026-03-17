---
name: asset-store-contract
status: stub
version: 0.1
published_by: infrastructure (asset-store service)
consumed_by: engine/asset-pipeline, nodes (for serving to clients)
---

# Asset Store Contract

*Status: STUB*

## What This Contract Provides

- `get_asset(type_id, lod_tier, version)` → binary geometry blob or NOT_FOUND
- `get_asset_metadata(type_id)` → version number, tier count, total sizes per tier
- `upload_asset(type_id, lod_tier, binary_data)` → accepted or rejected
- `list_types()` → all registered object type IDs
- `check_version(type_id)` → current version number for type

## Notes

- GET operations are cacheable — assets are immutable at a given version
- Upload is restricted to authorized sources (world editor, build pipeline)
- Version increments on any geometry change — clients that cached an old version re-request
