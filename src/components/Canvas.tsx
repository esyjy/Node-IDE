import {
  Background,
  Controls,
  ReactFlow,
  type Connection,
  type Edge as FlowEdge,
  type Node,
  type NodeTypes,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useCallback, useMemo } from "react";
import type { Edge, NodeInstance } from "../types/graph";
import { WorkflowNode, type WorkflowNodeData } from "./WorkflowNode";

const nodeTypes: NodeTypes = {
  workflow: WorkflowNode,
};

interface CanvasProps {
  nodes: NodeInstance[];
  edges: Edge[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onConnect: (connection: Connection) => void;
  onRemoveEdge: (edgeId: string) => void;
  onError: (message: string) => void;
}

export function Canvas({
  nodes,
  edges,
  selectedId,
  onSelect,
  onConnect,
  onRemoveEdge,
  onError,
}: CanvasProps) {
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

  const flowEdges: FlowEdge[] = useMemo(
    () =>
      edges.map((edge) => ({
        id: edge.id,
        source: edge.source_node_id,
        target: edge.target_node_id,
        sourceHandle: edge.source_port,
        targetHandle: edge.target_port,
      })),
    [edges],
  );

  const isValidConnection = useCallback((connection: Connection | FlowEdge) => {
    return connection.sourceHandle === "out" && connection.targetHandle === "in";
  }, []);

  const handleConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      if (!isValidConnection(connection)) {
        onError("v2 connections must be out → in");
        return;
      }
      onConnect(connection);
    },
    [isValidConnection, onConnect, onError],
  );

  return (
    <div className="canvas-panel">
      <ReactFlow
        nodes={flowNodes}
        edges={flowEdges}
        nodeTypes={nodeTypes}
        fitView
        onNodeClick={(_, node) => onSelect(node.id)}
        onPaneClick={() => onSelect(null)}
        onConnect={handleConnect}
        isValidConnection={isValidConnection}
        onEdgesDelete={(deleted) => {
          for (const edge of deleted) {
            if (edge.id) onRemoveEdge(edge.id);
          }
        }}
        proOptions={{ hideAttribution: true }}
      >
        <Background gap={16} size={1} />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  );
}
