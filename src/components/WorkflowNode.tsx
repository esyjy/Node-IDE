import { Handle, Position, useNodeId, type NodeProps } from "@xyflow/react";
import { portLabel } from "../lib/protocol";
import { useNodesSnapshot } from "../context/NodesContext";
import type { Lifecycle } from "../types/graph";
import { lifecycleModeLabel, nodeKindLabel } from "../types/graph";
import type { WorkflowNodeData } from "./WorkflowNode.types";

function lifecycleClass(lifecycle: Lifecycle): string {
  return `lifecycle-badge lifecycle-${lifecycle}`;
}

function nodeShellClass(lifecycle: Lifecycle, mode: string): string {
  return `workflow-node workflow-node--${lifecycle} workflow-node-mode-${mode}`;
}

export function WorkflowNode({ data }: NodeProps) {
  const nodeData = data as WorkflowNodeData;
  const nodeId = useNodeId();
  const nodes = useNodesSnapshot();
  const instance =
    (nodeId ? nodes.find((node) => node.id === nodeId) : undefined) ??
    nodeData.instance;
  const inDecl = instance.port_decls?.in;
  const outDecl = instance.port_decls?.out;
  const mode = instance.lifecycle_mode ?? "ephemeral";

  return (
    <div
      className={`${nodeShellClass(instance.lifecycle, mode)} ${nodeData.selected ? "selected" : ""}`}
    >
      <div className="handle-group handle-in">
        {inDecl && <span className="port-label">{portLabel(inDecl)}</span>}
        <Handle type="target" position={Position.Left} id="in" />
      </div>
      <div className="workflow-node-header">
        <span className="workflow-node-title">{nodeKindLabel(instance.kind)}</span>
        <span className={`mode-badge mode-${mode}`} title={mode}>
          {lifecycleModeLabel(mode)}
        </span>
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
