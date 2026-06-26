export type Lifecycle = "created" | "running" | "done" | "failed";

export type NodeKind =
  | { kind: "constant"; value: string }
  | { kind: "echo"; input: string };

export interface Position {
  x: number;
  y: number;
}

export interface NodeInstance {
  id: string;
  kind: NodeKind;
  lifecycle: Lifecycle;
  last_output: string | null;
  position: Position;
}

export interface AppStateSnapshot {
  schema_version: number;
  nodes: NodeInstance[];
  project_path: string;
}

export interface RunResult {
  node_id: string;
  output: string;
  lifecycle: Lifecycle;
}

export interface UpdateInfo {
  available: boolean;
  current_version: string;
  latest_version: string | null;
  notes: string | null;
}

export function nodeKindLabel(kind: NodeKind): string {
  return kind.kind === "constant" ? "Constant" : "Echo";
}

export function nodeKindType(kind: NodeKind): "constant" | "echo" {
  return kind.kind;
}
