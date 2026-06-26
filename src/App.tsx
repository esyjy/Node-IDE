import { useEffect, useState } from "react";
import type { Connection } from "@xyflow/react";
import { getVersion } from "@tauri-apps/api/app";
import { ReactFlowProvider } from "@xyflow/react";
import { Canvas } from "./components/Canvas";
import { NodePalette } from "./components/NodePalette";
import { PropertyPanel } from "./components/PropertyPanel";
import { ResultPanel } from "./components/ResultPanel";
import { UpdateDialog } from "./components/UpdateDialog";
import { useAppState } from "./hooks/useAppState";
import type { GraphRunResult, RunResult } from "./types/graph";
import { nodeKindType } from "./types/graph";
import "./App.css";

function App() {
  const {
    state,
    error,
    loading,
    addNode,
    updateNode,
    removeNode,
    addEdge,
    removeEdge,
    moveNode,
    runNode,
    runGraph,
  } = useAppState();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [lastGraphRun, setLastGraphRun] = useState<GraphRunResult | null>(null);
  const [lastNodeRun, setLastNodeRun] = useState<RunResult | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [updateOpen, setUpdateOpen] = useState(false);
  const [appVersion, setAppVersion] = useState("…");

  const nodes = state?.nodes ?? [];
  const edges = state?.edges ?? [];
  const selectedNode = nodes.find((node) => node.id === selectedId) ?? null;

  useEffect(() => {
    void getVersion().then(setAppVersion).catch(() => setAppVersion("dev"));
  }, []);

  const showToast = (message: string) => {
    setToast(message);
    window.setTimeout(() => setToast(null), 3000);
  };

  const handleRunGraph = async () => {
    if (nodes.length === 0) {
      showToast("Add nodes first");
      return;
    }
    try {
      const result = await runGraph();
      setLastGraphRun(result);
      setLastNodeRun(null);
    } catch (err) {
      showToast(String(err));
    }
  };

  const handleRunNode = async () => {
    const target = selectedNode;
    if (!target) {
      showToast("Select a node to run");
      return;
    }
    try {
      const result = await runNode(target.id);
      setLastNodeRun(result);
      setLastGraphRun(null);
    } catch (err) {
      showToast(String(err));
    }
  };

  const handleConnect = (connection: Connection) => {
    if (!connection.source || !connection.target) return;
    void addEdge(
      connection.source,
      connection.sourceHandle ?? "out",
      connection.target,
      connection.targetHandle ?? "in",
    )
      .catch(showToast);
  };

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">
          <h1>Node-IDE</h1>
          <span className="version">v{appVersion}</span>
        </div>
        <div className="toolbar">
          <button type="button" className="primary-btn" onClick={() => void handleRunGraph()}>
            Run
          </button>
          <button type="button" onClick={() => void handleRunNode()}>
            Run node
          </button>
          <button type="button" onClick={() => setUpdateOpen(true)}>
            Check for updates
          </button>
        </div>
      </header>

      {loading && <p className="banner">Loading project…</p>}
      {error && <p className="banner error-text">{error}</p>}
      {toast && <p className="toast">{toast}</p>}

      <main className="workspace">
        <NodePalette
          onAdd={(kind) => {
            const offset = nodes.length * 40;
            void addNode(kind, 120 + offset, 120 + offset)
              .then((snapshot) => {
                const created = snapshot.nodes[snapshot.nodes.length - 1];
                if (created) setSelectedId(created.id);
              })
              .catch(showToast);
          }}
        />

        <ReactFlowProvider>
          <Canvas
            nodes={nodes}
            edges={edges}
            selectedId={selectedId}
            onSelect={setSelectedId}
            onConnect={handleConnect}
            onMoveNode={(id, x, y) => {
              void moveNode(id, x, y).catch(showToast);
            }}
            onRemoveEdge={(id) => {
              void removeEdge(id).catch(showToast);
            }}
            onError={showToast}
          />
        </ReactFlowProvider>

        <aside className="side-panel">
          <PropertyPanel
            node={selectedNode}
            onUpdate={(value, input) => {
              if (!selectedNode) return;
              const kind = nodeKindType(selectedNode.kind);
              void updateNode(selectedNode.id, kind, value, input).catch(showToast);
            }}
            onRemove={() => {
              if (!selectedNode) return;
              void removeNode(selectedNode.id)
                .then(() => {
                  setSelectedId(null);
                  setLastGraphRun(null);
                  setLastNodeRun(null);
                })
                .catch(showToast);
            }}
          />
          <ResultPanel
            nodes={nodes}
            edges={edges}
            lastGraphRun={lastGraphRun}
            lastNodeRun={lastNodeRun}
          />
        </aside>
      </main>

      <UpdateDialog open={updateOpen} onClose={() => setUpdateOpen(false)} />
    </div>
  );
}

export default App;
