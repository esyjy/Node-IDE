import type { NodeInstance } from "../types/graph";

export interface WorkflowNodeData {
  instance: NodeInstance;
  selected: boolean;
  [key: string]: unknown;
}
