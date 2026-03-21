/**
 * Schema registry client — mirrors nexus-schema (Rust crate) in TypeScript.
 *
 * Schema IDs are FNV-1a 32-bit hashes of "<name>@<version>".
 * They are computed, not hardcoded. Two runtimes that hash the same string
 * get the same number without coordination.
 *
 * The two bootstrap IDs are the only hardcoded values — they define how to
 * find everything else.
 */

// Bootstrap IDs — hardcoded forever. Everything else is schemaId().
export const SCHEMA_UNTYPED  = 0  // legacy / unknown — decode by msg_type only
export const SCHEMA_REGISTRY = 1  // schema discovery — MSG_SCHEMA_QUERY/RESPONSE

// Schema discovery message types
export const MSG_SCHEMA_QUERY     = 0x0500  // C→S: "what is schema 0xABCD1234?"
export const MSG_SCHEMA_RESPONSE  = 0x0501  // S→C: SchemaDescriptor as JSON bytes
export const MSG_SCHEMA_NOT_FOUND = 0x0502  // S→C: schema_id unknown to this server

// ── FNV-1a 32-bit hash ────────────────────────────────────────────────────────

/** FNV-1a 32-bit hash over a UTF-8 string. */
export function fnv32(s: string): number {
  let hash = 0x811c9dc5  // offset basis
  for (let i = 0; i < s.length; i++) {
    hash ^= s.charCodeAt(i)
    // JavaScript bitwise ops are 32-bit signed; use >>> 0 to keep unsigned
    hash = Math.imul(hash, 0x01000193) >>> 0
  }
  return hash >>> 0
}

/**
 * Compute the stable schema_id for a name and version.
 * Mirrors `nexus_schema::schema_id(name, version)` exactly.
 *
 * @example
 * schemaId('spatial_manifest', '1.0')  // same number as Rust computes
 */
export function schemaId(name: string, version: string): number {
  return fnv32(`${name}@${version}`)
}

// ── Pre-computed IDs for known schemas ───────────────────────────────────────
// These are values, not constants — they are computed at module load time.
// If you add a new schema, add a line here. No other file needs to change.

export const SID_PHYSICS_BODY     = schemaId('physics_body',     '1.0')
export const SID_SPATIAL_MANIFEST = schemaId('spatial_manifest', '1.0')
export const SID_AGENT_TASK       = schemaId('agent_task',       '1.0')
export const SID_COMPUTER         = schemaId('computer',         '1.0')
export const SID_DISPLAY_FRAME    = schemaId('display_frame',    '1.0')
export const SID_FILE             = schemaId('file',             '1.0')

// ── SchemaDescriptor (parsed from MSG_SCHEMA_RESPONSE JSON) ──────────────────

export interface FieldDescriptor {
  name:        string
  type:        string
  description: string
}

export interface SchemaDescriptor {
  name:        string
  version:     string
  description: string
  fields:      FieldDescriptor[]
  encoding:    string
}

// ── Client-side registry — populated at runtime via MSG_SCHEMA_QUERY ─────────

const _cache = new Map<number, SchemaDescriptor>()

/** Store a descriptor received from the server. */
export function cacheSchema(id: number, descriptor: SchemaDescriptor): void {
  _cache.set(id, descriptor)
}

/** Look up a cached descriptor. Returns null if not yet fetched. */
export function getCachedSchema(id: number): SchemaDescriptor | null {
  return _cache.get(id) ?? null
}

/** Decode a MSG_SCHEMA_RESPONSE payload into a descriptor and cache it. */
export function decodeSchemaResponse(payload: ArrayBuffer): { id: number; descriptor: SchemaDescriptor } | null {
  if (payload.byteLength < 4) return null
  const view = new DataView(payload)
  const id   = view.getUint32(0, true)
  try {
    const json = new TextDecoder().decode(new Uint8Array(payload, 4))
    const descriptor = JSON.parse(json) as SchemaDescriptor
    cacheSchema(id, descriptor)
    return { id, descriptor }
  } catch {
    return null
  }
}
