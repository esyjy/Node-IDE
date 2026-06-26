import { Handle, Position, type NodeProps } from "@xyflow/react";
import { portLabel } from "../lib/protocol";
import type { Lifecycle, NodeInstance } from "../types/graph";
import { nodeKindLabel } from "../types/graph";

export interface WorkflowNodeData {
  instance: NodeInstance;
  selected: boolean;
  [key: string]: unknown;
}

function lifecycleClass(lifecycle: Lifecycle): string {
  return `lifecycle-badge lifecycle-${lifecycle}`;
}

export function WorkflowNode({ data }: NodeProps) {
  const nodeData = data as WorkflowNodeData;
  const { instance } = nodeData;
  const inDecl = instance.port_decls?.in;
  const outDecl = instance.port_decls?.out;

  return (
    <div className={`workflow-node ${nodeData.selected ? "selected" : ""}`}>
      <div className="handle-group handle-in">
        {inDecl && <span className="port-label">{portLabel(inDecl)}</span>}
        <Handle type="target" position={Position.Left} id="in" />
      </div>
      <div className="workflow-node-header">
        <span className="workflow-node-title">{nodeKindLabel(instance.kind)}</span>
        <span className={lifecycleClass(instance.lifecycle)}>{instance.lifecycle}</span>
      </div>
      <div className="workflow-node-body">
        {instance.kind.kind === "echo" ? (
          <code>{instance.kind.input || "(empty)"}</code>
        ) : (
          <code>{instance.kind.value}</code>
        )}
      </div>
      {instance.last_output && (
        <div className="workflow-node-output">→ {instance.last_output}</div>
      )}
      <div className="handle-group handle-out">
        <Handle type="source" position={Position.Right} id="out" />
        {outDecl && <span className="port-label">{portLabel(outDecl)}</span>}
      </div>
    </div>
  );
}
