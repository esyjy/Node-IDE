import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppStateSnapshot,
  ConnectionValidation,
  GraphRunResult,
  LifecycleMode,
  PortDeclaration,
  RunResult,
} from "../types/graph";

export type NodeKindName = "constant" | "json_constant" | "echo";

export function useAppState() {
  const [state, setState] = useState<AppStateSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const snapshot = await invoke<AppStateSnapshot>("get_app_state");
      setState(snapshot);
      setError(null);
      return snapshot;
    } catch (err) {
      setError(String(err));
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const addNode = useCallback(
    async (kind: NodeKindName, x?: number, y?: number) => {
      const snapshot = await invoke<AppStateSnapshot>("add_node", {
        request: {
          kind,
          value:
            kind === "constant"
              ? "hello"
              : kind === "json_constant"
                ? "{}"
                : undefined,
          input: kind === "echo" ? "" : undefined,
          x,
          y,
        },
      });
      setState(snapshot);
      return snapshot;
    },
    [],
  );

  const updateNode = useCallback(
    async (
      id: string,
      kind: NodeKindName,
      value?: string,
      input?: string,
    ) => {
      const snapshot = await invoke<AppStateSnapshot>("update_node", {
        request: { id, kind, value, input },
      });
      setState(snapshot);
      return snapshot;
    },
    [],
  );

  const updateNodePorts = useCallback(
    async (id: string, portDecls: Record<string, { what: string; how: string }>) => {
      const snapshot = await invoke<AppStateSnapshot>("update_node_ports", {
        request: { id, port_decls: portDecls },
      });
      setState(snapshot);
      return snapshot;
    },
    [],
  );

  const updateNodeMode = useCallback(async (id: string, mode: LifecycleMode) => {
    const snapshot = await invoke<AppStateSnapshot>("update_node_mode", {
      request: { id, mode },
    });
    setState(snapshot);
    return snapshot;
  }, []);

  const startNode = useCallback(async (id: string) => {
    const snapshot = await invoke<AppStateSnapshot>("start_node", { id });
    setState(snapshot);
    return snapshot;
  }, []);

  const stopNode = useCallback(async (id: string) => {
    const snapshot = await invoke<AppStateSnapshot>("stop_node", { id });
    setState(snapshot);
    return snapshot;
  }, []);

  const removeNode = useCallback(async (id: string) => {
    const snapshot = await invoke<AppStateSnapshot>("remove_node", { id });
    setState(snapshot);
    return snapshot;
  }, []);

  const addEdge = useCallback(
    async (
      sourceNodeId: string,
      sourcePort: string,
      targetNodeId: string,
      targetPort: string,
    ) => {
      const snapshot = await invoke<AppStateSnapshot>("add_edge", {
        request: {
          source_node_id: sourceNodeId,
          source_port: sourcePort,
          target_node_id: targetNodeId,
          target_port: targetPort,
        },
      });
      setState(snapshot);
      return snapshot;
    },
    [],
  );

  const validateConnection = useCallback(
    async (
      sourceNodeId: string,
      sourcePort: string,
      targetNodeId: string,
      targetPort: string,
    ) => {
      return invoke<ConnectionValidation>("validate_connection", {
        request: {
          source_node_id: sourceNodeId,
          source_port: sourcePort,
          target_node_id: targetNodeId,
          target_port: targetPort,
        },
      });
    },
    [],
  );

  const removeEdge = useCallback(async (id: string) => {
    const snapshot = await invoke<AppStateSnapshot>("remove_edge", { id });
    setState(snapshot);
    return snapshot;
  }, []);

  const moveNode = useCallback(async (id: string, x: number, y: number) => {
    const snapshot = await invoke<AppStateSnapshot>("move_node", { id, x, y });
    setState(snapshot);
    return snapshot;
  }, []);

  const runNode = useCallback(
    async (id: string) => {
      const result = await invoke<RunResult>("run_node", { id });
      await refresh();
      return result;
    },
    [refresh],
  );

  const runGraph = useCallback(async () => {
    const result = await invoke<GraphRunResult>("run_graph");
    await refresh();
    return result;
  }, [refresh]);

  return {
    state,
    error,
    loading,
    refresh,
    addNode,
    updateNode,
    updateNodePorts,
    updateNodeMode,
    startNode,
    stopNode,
    removeNode,
    addEdge,
    validateConnection,
    removeEdge,
    moveNode,
    runNode,
    runGraph,
  };
}

export type { PortDeclaration };
