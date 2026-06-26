import {
  Background,
  Controls,
  ReactFlow,
  applyNodeChanges,
  type Connection,
  type Edge as FlowEdge,
  type Node,
  type NodeChange,
  type NodeTypes,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useCallback, useEffect, useRef, useState } from "react";
import { formatRejection, resolvePorts } from "../lib/protocol";
import type { Edge, NodeInstance } from "../types/graph";
import { WorkflowNode, type WorkflowNodeData } from "./WorkflowNode";

const nodeTypes: NodeTypes = {
  workflow: WorkflowNode,
};

function toFlowNodes(
  nodes: NodeInstance[],
  selectedId: string | null,
): Node<WorkflowNodeData>[] {
  return nodes.map((instance) => ({
    id: instance.id,
    type: "workflow",
    position: instance.position,
    data: { instance, selected: instance.id === selectedId },
    selected: instance.id === selectedId,
  }));
}

interface CanvasProps {
  nodes: NodeInstance[];
  edges: Edge[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onConnect: (connection: Connection) => void;
  onMoveNode: (id: string, x: number, y: number) => void;
  onRemoveEdge: (edgeId: string) => void;
  onError: (message: string) => void;
}

export function Canvas({
  nodes,
  edges,
  selectedId,
  onSelect,
  onConnect,
  onMoveNode,
  onRemoveEdge,
  onError,
}: CanvasProps) {
  const [flowNodes, setFlowNodes] = useState(() => toFlowNodes(nodes, selectedId));
  const lastRejectionKey = useRef<string | null>(null);

  useEffect(() => {
    setFlowNodes(toFlowNodes(nodes, selectedId));
  }, [nodes, selectedId]);

  const onNodesChange = useCallback((changes: NodeChange<Node<WorkflowNodeData>>[]) => {
    setFlowNodes((current) => applyNodeChanges(changes, current));
  }, []);

  const flowEdges: FlowEdge[] = edges.map((edge) => ({
    id: edge.id,
    source: edge.source_node_id,
    target: edge.target_node_id,
    sourceHandle: edge.source_port,
    targetHandle: edge.target_port,
  }));

  const isValidConnection = useCallback(
    (connection: Connection | FlowEdge) => {
      if (connection.sourceHandle !== "out" || connection.targetHandle !== "in") {
        return false;
      }
      if (!connection.source || !connection.target) {
        return false;
      }

      const sourceNode = nodes.find((n) => n.id === connection.source);
      const targetNode = nodes.find((n) => n.id === connection.target);
      if (!sourceNode || !targetNode) return false;

      const sourceDecl = sourceNode.port_decls?.out;
      const targetDecl = targetNode.port_decls?.in;
      if (!sourceDecl || !targetDecl) return false;

      const result = resolvePorts(sourceDecl, targetDecl);
      if (!result.compatible) {
        const key = `${connection.source}:${connection.target}:${result.reason}`;
        if (lastRejectionKey.current !== key) {
          lastRejectionKey.current = key;
          onError(formatRejection(result));
        }
        return false;
      }

      lastRejectionKey.current = null;
      return true;
    },
    [nodes, onError],
  );

  const handleConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      if (!isValidConnection(connection)) {
        return;
      }
      onConnect(connection);
    },
    [isValidConnection, onConnect],
  );

  const handleNodeDragStop = useCallback(
    (_event: MouseEvent | TouchEvent, node: Node<WorkflowNodeData>) => {
      onMoveNode(node.id, node.position.x, node.position.y);
    },
    [onMoveNode],
  );

  return (
    <div className="canvas-panel">
      <ReactFlow
        nodes={flowNodes}
        edges={flowEdges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onNodeDragStop={handleNodeDragStop}
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
