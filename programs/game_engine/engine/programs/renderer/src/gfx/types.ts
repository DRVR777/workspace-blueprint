/**
 * GfxContext — platform abstraction boundary.
 *
 * The NexusCanvas component implements this contract by wrapping R3F's Canvas.
 * Nothing outside gfx/ may reference WebGL, WebGPU, or Three.js renderer
 * objects directly — all platform specifics are owned here.
 */
export interface GfxContext {
  /** Backing canvas element */
  readonly canvas: HTMLCanvasElement
  /** Human-readable backend name: "WebGL2" | "WebGPU" */
  readonly backend: string
  /** Device pixel ratio in use */
  readonly dpr: number
}

/** Props forwarded from app layer to NexusCanvas */
export interface NexusCanvasProps {
  children: React.ReactNode
  /** Override DPR for performance testing. Default: window.devicePixelRatio (capped at 2) */
  dpr?: number | [number, number]
}
