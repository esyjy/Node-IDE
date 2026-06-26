import type { NodeInstance } from "../types/graph";
import { nodeKindLabel } from "../types/graph";

interface PropertyPanelProps {
  node: NodeInstance | null;
  onUpdate: (value: string, input: string) => void;
  onRemove: () => void;
}

export function PropertyPanel({ node, onRemove, onUpdate }: PropertyPanelProps) {
  if (!node) {
    return (
      <div className="property-panel">
        <h2>Properties</h2>
        <p className="muted">Select a node to edit its properties.</p>
      </div>
    );
  }

  const fieldValue =
    node.kind.kind === "constant" ? node.kind.value : node.kind.input;
  const isConstant = node.kind.kind === "constant";

  return (
    <div className="property-panel">
      <h2>{nodeKindLabel(node.kind)}</h2>
      <label className="field">
        <span>{isConstant ? "Value" : "Input"}</span>
        <input
          type="text"
          value={fieldValue}
          onChange={(event) => {
            if (isConstant) {
              onUpdate(event.target.value, "");
            } else {
              onUpdate("", event.target.value);
            }
          }}
        />
      </label>
      <button type="button" className="danger-btn" onClick={onRemove}>
        Remove node
      </button>
    </div>
  );
}
