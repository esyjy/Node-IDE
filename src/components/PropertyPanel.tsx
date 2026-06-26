import { HOW_OPTIONS, WHAT_OPTIONS, portLabel, whatId, howId } from "../lib/protocol";
import type { NodeInstance } from "../types/graph";
import { nodeKindLabel, nodePorts } from "../types/graph";

interface PropertyPanelProps {
  node: NodeInstance | null;
  onUpdate: (value: string, input: string) => void;
  onUpdatePorts: (portDecls: Record<string, { what: string; how: string }>) => void;
  onRemove: () => void;
}

export function PropertyPanel({ node, onRemove, onUpdate, onUpdatePorts }: PropertyPanelProps) {
  if (!node) {
    return (
      <div className="property-panel">
        <h2>Properties</h2>
        <p className="muted">Select a node to edit its properties.</p>
      </div>
    );
  }

  const isEcho = node.kind.kind === "echo";
  const fieldValue =
    node.kind.kind === "echo" ? node.kind.input : node.kind.value;

  const ports = nodePorts(node.kind);

  const updatePort = (portId: string, what: string, how: string) => {
    onUpdatePorts({ [portId]: { what, how } });
  };

  return (
    <div className="property-panel">
      <h2>{nodeKindLabel(node.kind)}</h2>
      <label className="field">
        <span>{isEcho ? "Input" : "Value"}</span>
        <input
          type="text"
          value={fieldValue}
          onChange={(event) => {
            if (isEcho) {
              onUpdate("", event.target.value);
            } else {
              onUpdate(event.target.value, "");
            }
          }}
        />
      </label>

      <h3 className="panel-subheading">Port declarations</h3>
      {ports.map((portId) => {
        const decl = node.port_decls?.[portId];
        const what = decl ? whatId(decl) : "text";
        const how = decl ? howId(decl) : "single";
        return (
          <div key={portId} className="port-decl-editor">
            <strong className="port-decl-name">{portId}</strong>
            {decl && <span className="muted port-decl-summary">{portLabel(decl)}</span>}
            <label className="field">
              <span>What</span>
              <select
                value={what}
                onChange={(e) => updatePort(portId, e.target.value, how)}
              >
                {WHAT_OPTIONS.map((opt) => (
                  <option key={opt} value={opt}>
                    {opt}
                  </option>
                ))}
              </select>
            </label>
            <label className="field">
              <span>How</span>
              <select
                value={how}
                onChange={(e) => updatePort(portId, what, e.target.value)}
              >
                {HOW_OPTIONS.map((opt) => (
                  <option key={opt} value={opt}>
                    {opt}
                  </option>
                ))}
              </select>
            </label>
          </div>
        );
      })}

      <button type="button" className="danger-btn" onClick={onRemove}>
        Remove node
      </button>
    </div>
  );
}
