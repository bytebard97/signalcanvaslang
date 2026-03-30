// SignalCanvasLang/packages/diagram/src/types.ts
import type { Node, Edge } from '@vue-flow/core'

export interface PortHandle {
  /** Globally unique handle ID: "{instanceName}-{portName}-{source|target}" */
  id: string
  /** Display label, e.g. "Dante_In[1..32]" */
  name: string
  /** Port range notation if this port covers multiple channels, e.g. "[1..32]" */
  range?: string
}

export interface DeviceNodeData {
  // Set by adapter (useAstToFlow / useProbeToFlow)
  instanceName: string
  templateName: string
  category: string
  manufacturer: string
  model: string
  location: string
  inputPorts: PortHandle[]
  outputPorts: PortHandle[]
  // Injected by FlowDiagram before setNodes
  connectedPortIds?: Set<string>
  mode?: 'wires' | 'netnames'
  portTags?: Record<string, Array<{ label: string; edgeId: string }>>
  drillable?: boolean
}

export interface FlowGraph {
  nodes: Node[]
  edges: Edge[]
}

// Full CompileResult shape matching the WASM compiler output.
// Moved here from usePatchlangCompiler.ts so useAstToFlow no longer
// depends on a file outside the package.

export interface ParseError {
  message: string
  span: { start: number; end: number }
  hint?: string | null
}

export interface Diagnostic {
  code: string
  message: string
  severity: 'error' | 'warning' | 'info'
  layer?: string
  span?: { start: number; end: number }
}

export interface CompileResult {
  success: boolean
  program: unknown | null
  errors: ParseError[]
  diagnostics: Diagnostic[]
  rawJson: string
}
