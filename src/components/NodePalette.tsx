import type { NodeKindName } from "../hooks/useAppState";

interface NodePaletteProps {
  onAdd: (kind: NodeKindName) => void;
}

export function NodePalette({ onAdd }: NodePaletteProps) {
  return (
    <div className="palette-panel">
      <h2>Nodes</h2>
      <p className="palette-hint">Click to place on canvas</p>
      <button type="button" className="palette-card" onClick={() => onAdd("constant")}>
        <strong>Constant</strong>
        <span>Outputs text·single</span>
      </button>
      <button type="button" className="palette-card" onClick={() => onAdd("json_constant")}>
        <strong>JsonConstant</strong>
        <span>Outputs json·single (demo)</span>
      </button>
      <button type="button" className="palette-card" onClick={() => onAdd("echo")}>
        <strong>Echo</strong>
        <span>Outputs wired or panel input</span>
      </button>
    </div>
  );
}
