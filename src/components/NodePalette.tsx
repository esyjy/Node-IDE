interface NodePaletteProps {
  hasNode: boolean;
  onAdd: (kind: "constant" | "echo") => void;
  onToast: (message: string) => void;
}

export function NodePalette({ hasNode, onAdd, onToast }: NodePaletteProps) {
  const handleAdd = (kind: "constant" | "echo") => {
    if (hasNode) {
      onToast("v1 allows only one node on the canvas");
      return;
    }
    onAdd(kind);
  };

  return (
    <div className="palette-panel">
      <h2>Nodes</h2>
      <p className="palette-hint">Click to place on canvas (max 1 in v1)</p>
      <button type="button" className="palette-card" onClick={() => handleAdd("constant")}>
        <strong>Constant</strong>
        <span>Outputs a fixed value</span>
      </button>
      <button type="button" className="palette-card" onClick={() => handleAdd("echo")}>
        <strong>Echo</strong>
        <span>Outputs the input field value</span>
      </button>
    </div>
  );
}
