import { useState } from "react";
import { ReactFlowProvider } from "@xyflow/react";
import { Canvas } from "./components/Canvas";
import { NodePalette } from "./components/NodePalette";
import { PropertyPanel } from "./components/PropertyPanel";
import { ResultPanel } from "./components/ResultPanel";
import { UpdateDialog } from "./components/UpdateDialog";
import { useAppState } from "./hooks/useAppState";
import type { RunResult } from "./types/graph";
import { nodeKindType } from "./types/graph";
import "./App.css";

function App() {
  const { state, error, loading, addNode, updateNode, removeNode, runNode } = useAppState();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [lastRun, setLastRun] = useState<RunResult | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [updateOpen, setUpdateOpen] = useState(false);

  const nodes = state?.nodes ?? [];
  const selectedNode = nodes.find((node) => node.id === selectedId) ?? nodes[0] ?? null;

  const showToast = (message: string) => {
    setToast(message);
    window.setTimeout(() => setToast(null), 3000);
  };

  const handleRun = async () => {
    const target = selectedNode ?? nodes[0];
    if (!target) {
      showToast("Add a node first");
      return;
    }
    try {
      const result = await runNode(target.id);
      setLastRun(result);
    } catch (err) {
      showToast(String(err));
    }
  };

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">
          <h1>Node-IDE</h1>
          <span className="version">v0.1.1</span>
        </div>
        <div className="toolbar">
          <button type="button" className="primary-btn" onClick={() => void handleRun()}>
            Run
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
          hasNode={nodes.length > 0}
          onAdd={(kind) => {
            void addNode(kind).then((snapshot) => {
              const created = snapshot.nodes[0];
              if (created) setSelectedId(created.id);
            }).catch(showToast);
          }}
          onToast={showToast}
        />

        <ReactFlowProvider>
          <Canvas
            nodes={nodes}
            selectedId={selectedId}
            onSelect={setSelectedId}
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
                  setLastRun(null);
                })
                .catch(showToast);
            }}
          />
          <ResultPanel nodes={nodes} lastRun={lastRun} />
        </aside>
      </main>

      <UpdateDialog open={updateOpen} onClose={() => setUpdateOpen(false)} />
    </div>
  );
}

export default App;
