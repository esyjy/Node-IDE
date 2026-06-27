import { createContext, useContext, type ReactNode } from "react";
import type { NodeInstance } from "../types/graph";

const NodesContext = createContext<NodeInstance[]>([]);

export function NodesProvider({
  nodes,
  children,
}: {
  nodes: NodeInstance[];
  children: ReactNode;
}) {
  return <NodesContext.Provider value={nodes}>{children}</NodesContext.Provider>;
}

export function useNodesSnapshot(): NodeInstance[] {
  return useContext(NodesContext);
}
