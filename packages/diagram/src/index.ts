// SignalCanvasLang/packages/diagram/src/index.ts
export { default as FlowDiagram }      from './FlowDiagram.vue'
export { default as DeviceNode }       from './DeviceNode.vue'
export { default as OrthogonalEdge }   from './OrthogonalEdge.vue'
export { default as PinTag }           from './PinTag.vue'
export { transformAstToFlow }          from './useAstToFlow'

export type {
  DeviceNodeData,
  PortHandle,
  FlowGraph,
  CompileResult,
  ParseError,
  Diagnostic,
} from './types'
