import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppStateSnapshot,
  ConnectionValidation,
  GraphRunResult,
  Lifecycle,
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
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();

    const unsubs: Array<() => void> = [];

    void listen<{ node_id: string; lifecycle: Lifecycle }>("node:lifecycle", () => {
      void refresh();
    }).then((unlisten) => unsubs.push(unlisten));

    void listen("node:output", () => {
      void refresh();
    }).then((unlisten) => unsubs.push(unlisten));

    void listen("message:delivered", () => {
      void refresh();
    }).then((unlisten) => unsubs.push(unlisten));

    return () => {
      unsubs.forEach((fn) => fn());
    };
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

  const runNode = useCallback(async (id: string) => {
    const result = await invoke<RunResult>("run_node", { id });
    await refresh();
    return result;
  }, [refresh]);

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
