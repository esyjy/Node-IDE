import {
  Background,
  Controls,
  ReactFlow,
  type Node,
  type NodeTypes,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useMemo } from "react";
import type { NodeInstance } from "../types/graph";
import { WorkflowNode, type WorkflowNodeData } from "./WorkflowNode";

const nodeTypes: NodeTypes = {
  workflow: WorkflowNode,
};

interface CanvasProps {
  nodes: NodeInstance[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
}

export function Canvas({ nodes, selectedId, onSelect }: CanvasProps) {
  const flowNodes: Node<WorkflowNodeData>[] = useMemo(
    () =>
      nodes.map((instance) => ({
        id: instance.id,
        type: "workflow",
        position: instance.position,
        data: { instance, selected: instance.id === selectedId },
        selected: instance.id === selectedId,
      })),
    [nodes, selectedId],
  );

  return (
    <div className="canvas-panel">
      <ReactFlow
        nodes={flowNodes}
        edges={[]}
        nodeTypes={nodeTypes}
        fitView
        onNodeClick={(_, node) => onSelect(node.id)}
        onPaneClick={() => onSelect(null)}
        proOptions={{ hideAttribution: true }}
      >
        <Background gap={16} size={1} />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  );
}
