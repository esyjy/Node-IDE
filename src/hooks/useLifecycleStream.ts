import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type {
  Lifecycle,
  LifecycleEventPayload,
  NodeInstance,
} from "../types/graph";

export interface LifecycleStreamState {
  nodes: NodeInstance[];
  activeEdgeIds: Set<string>;
  patchNode: (id: string, patch: Partial<Pick<NodeInstance, "lifecycle" | "last_output">>) => void;
  setNodes: (nodes: NodeInstance[]) => void;
}

export function useLifecycleStream(baseNodes: NodeInstance[]): LifecycleStreamState {
  const [nodes, setNodesState] = useState<NodeInstance[]>(baseNodes);
  const [activeEdgeIds, setActiveEdgeIds] = useState<Set<string>>(new Set());
  const pendingPatches = useRef<Map<string, Partial<Pick<NodeInstance, "lifecycle" | "last_output">>>>(
    new Map(),
  );
  const rafId = useRef<number | null>(null);
  const edgeTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  useEffect(() => {
    setNodesState(baseNodes);
  }, [baseNodes]);

  const flushPatches = useCallback(() => {
    if (pendingPatches.current.size === 0) return;
    const patches = new Map(pendingPatches.current);
    pendingPatches.current.clear();
    setNodesState((prev) =>
      prev.map((node) => {
        const patch = patches.get(node.id);
        return patch ? { ...node, ...patch } : node;
      }),
    );
    rafId.current = null;
  }, []);

  const scheduleFlush = useCallback(() => {
    if (rafId.current !== null) return;
    rafId.current = requestAnimationFrame(flushPatches);
  }, [flushPatches]);

  const patchNode = useCallback(
    (id: string, patch: Partial<Pick<NodeInstance, "lifecycle" | "last_output">>) => {
      const existing = pendingPatches.current.get(id) ?? {};
      pendingPatches.current.set(id, { ...existing, ...patch });
      scheduleFlush();
    },
    [scheduleFlush],
  );

  const highlightEdge = useCallback((edgeId: string) => {
    setActiveEdgeIds((prev) => new Set(prev).add(edgeId));
    const existing = edgeTimers.current.get(edgeId);
    if (existing) clearTimeout(existing);
    const timer = setTimeout(() => {
      setActiveEdgeIds((prev) => {
        const next = new Set(prev);
        next.delete(edgeId);
        return next;
      });
      edgeTimers.current.delete(edgeId);
    }, 600);
    edgeTimers.current.set(edgeId, timer);
  }, []);

  useEffect(() => {
    const unsubs: Array<() => void> = [];

    void listen<LifecycleEventPayload>("node:lifecycle", (event) => {
      patchNode(event.payload.node_id, { lifecycle: event.payload.lifecycle });
    }).then((unlisten) => unsubs.push(unlisten));

    void listen<{ node_id: string; output: string }>("node:output", (event) => {
      patchNode(event.payload.node_id, { last_output: event.payload.output });
    }).then((unlisten) => unsubs.push(unlisten));

    void listen<{ edge_id: string }>("message:delivered", (event) => {
      highlightEdge(event.payload.edge_id);
    }).then((unlisten) => unsubs.push(unlisten));

    return () => {
      unsubs.forEach((fn) => fn());
      if (rafId.current !== null) cancelAnimationFrame(rafId.current);
      edgeTimers.current.forEach((timer) => clearTimeout(timer));
      edgeTimers.current.clear();
    };
  }, [highlightEdge, patchNode]);

  const setNodes = useCallback((next: NodeInstance[]) => {
    pendingPatches.current.clear();
    setNodesState(next);
  }, []);

  return { nodes, activeEdgeIds, patchNode, setNodes };
}

export type { Lifecycle };
