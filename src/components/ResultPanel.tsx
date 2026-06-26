import type { NodeInstance, RunResult } from "../types/graph";

interface ResultPanelProps {
  nodes: NodeInstance[];
  lastRun: RunResult | null;
}

export function ResultPanel({ nodes, lastRun }: ResultPanelProps) {
  const active = nodes[0];

  return (
    <div className="result-panel">
      <h2>Result</h2>
      {lastRun ? (
        <div className="result-box">
          <div className="result-meta">
            <span>Status: {lastRun.lifecycle}</span>
          </div>
          <pre>{lastRun.output}</pre>
        </div>
      ) : active?.last_output ? (
        <div className="result-box">
          <div className="result-meta">
            <span>Status: {active.lifecycle}</span>
          </div>
          <pre>{active.last_output}</pre>
        </div>
      ) : (
        <p className="muted">Run a node to see output here.</p>
      )}
    </div>
  );
}
