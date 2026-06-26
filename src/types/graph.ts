export type Lifecycle = "created" | "running" | "done" | "failed";

export type NodeKind =
  | { kind: "constant"; value: string }
  | { kind: "json_constant"; value: string }
  | { kind: "echo"; input: string };

export interface Position {
  x: number;
  y: number;
}

export type { ConnectionValidation, PortDeclaration } from "../lib/protocol";
import type { PortDeclaration } from "../lib/protocol";

export interface NodeInstance {
  id: string;
  kind: NodeKind;
  lifecycle: Lifecycle;
  last_output: string | null;
  position: Position;
  port_decls: Record<string, PortDeclaration>;
}

export interface Edge {
  id: string;
  source_node_id: string;
  source_port: string;
  target_node_id: string;
  target_port: string;
}

export interface MessageEnvelope {
  source_node_id: string;
  source_port: string;
  sequence: number;
  payload: string;
}

export interface MessageDelivery {
  edge_id: string;
  envelope: MessageEnvelope;
}

export interface AppStateSnapshot {
  schema_version: number;
  nodes: NodeInstance[];
  edges: Edge[];
  project_path: string;
}

export interface RunResult {
  node_id: string;
  output: string;
  lifecycle: Lifecycle;
}

export interface GraphRunResult {
  node_results: RunResult[];
  deliveries: MessageDelivery[];
}

export interface UpdateInfo {
  available: boolean;
  current_version: string;
  latest_version: string | null;
  notes: string | null;
}

export function nodeKindLabel(kind: NodeKind): string {
  switch (kind.kind) {
    case "constant":
      return "Constant";
    case "json_constant":
      return "JsonConstant";
    case "echo":
      return "Echo";
  }
}

export function nodeKindType(kind: NodeKind): "constant" | "json_constant" | "echo" {
  return kind.kind;
}

export function nodePorts(kind: NodeKind): string[] {
  switch (kind.kind) {
    case "constant":
    case "json_constant":
      return ["out"];
    case "echo":
      return ["in", "out"];
  }
}

export function sinkNodeId(nodes: NodeInstance[], edges: Edge[]): string | null {
  const sources = new Set(edges.map((e) => e.source_node_id));
  const sinks = nodes.filter((n) => !sources.has(n.id));
  if (sinks.length === 1) return sinks[0].id;
  if (sinks.length > 1) return sinks[sinks.length - 1].id;
  return nodes[nodes.length - 1]?.id ?? null;
}
