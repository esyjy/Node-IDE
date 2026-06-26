import { Handle, Position, type NodeProps } from "@xyflow/react";
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

  return (
    <div className={`workflow-node ${nodeData.selected ? "selected" : ""}`}>
      <Handle type="target" position={Position.Left} id="in" />
      <div className="workflow-node-header">
        <span className="workflow-node-title">{nodeKindLabel(instance.kind)}</span>
        <span className={lifecycleClass(instance.lifecycle)}>{instance.lifecycle}</span>
      </div>
      <div className="workflow-node-body">
        {instance.kind.kind === "constant" ? (
          <code>{instance.kind.value}</code>
        ) : (
          <code>{instance.kind.input || "(empty)"}</code>
        )}
      </div>
      {instance.last_output && (
        <div className="workflow-node-output">→ {instance.last_output}</div>
      )}
      <Handle type="source" position={Position.Right} id="out" />
    </div>
  );
}
