import type { Edge, GraphRunResult, NodeInstance, RunResult } from "../types/graph";
import { sinkNodeId } from "../types/graph";

interface ResultPanelProps {
  nodes: NodeInstance[];
  edges: Edge[];
  lastGraphRun: GraphRunResult | null;
  lastNodeRun: RunResult | null;
}

export function ResultPanel({
  nodes,
  edges,
  lastGraphRun,
  lastNodeRun,
}: ResultPanelProps) {
  const sinkId = sinkNodeId(nodes, edges);
  const sinkNode = sinkId ? nodes.find((n) => n.id === sinkId) : null;

  const graphOutput =
    lastGraphRun && sinkId
      ? lastGraphRun.node_results.find((r) => r.node_id === sinkId)?.output
      : null;

  return (
    <div className="result-panel">
      <h2>Result</h2>
      {lastNodeRun ? (
        <div className="result-box">
          <div className="result-meta">
            <span>Run node · {lastNodeRun.lifecycle}</span>
          </div>
          <pre>{lastNodeRun.output}</pre>
        </div>
      ) : graphOutput ? (
        <div className="result-box">
          <div className="result-meta">
            <span>Run graph · sink: {sinkNode?.kind.kind ?? "node"}</span>
          </div>
          <pre>{graphOutput}</pre>
        </div>
      ) : sinkNode?.last_output ? (
        <div className="result-box">
          <div className="result-meta">
            <span>Last output · {sinkNode.lifecycle}</span>
          </div>
          <pre>{sinkNode.last_output}</pre>
        </div>
      ) : (
        <p className="muted">Run the graph or a single node to see output here.</p>
      )}
      {lastGraphRun && lastGraphRun.deliveries.length > 0 && (
        <p className="muted delivery-hint">
          {lastGraphRun.deliveries.length} message(s) delivered on edges
        </p>
      )}
    </div>
  );
}
