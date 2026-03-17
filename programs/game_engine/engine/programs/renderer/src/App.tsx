/**
 * App — top-level component.
 *
 * NexusCanvas (GfxContext boundary) wraps NexusScene.
 * Nothing outside NexusCanvas touches renderer/platform settings.
 */
import { NexusCanvas } from './gfx/NexusCanvas'
import { NexusScene } from './components/NexusScene'

export default function App() {
  return (
    <NexusCanvas>
      <NexusScene />
    </NexusCanvas>
  )
}
