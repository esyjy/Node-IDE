import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UpdateInfo } from "../types/graph";

interface UpdateDialogProps {
  open: boolean;
  onClose: () => void;
}

export function UpdateDialog({ open, onClose }: UpdateDialogProps) {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [status, setStatus] = useState<string>("idle");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) return;

    setInfo(null);
    setError(null);
    setStatus("checking");
    setBusy(true);

    void invoke<UpdateInfo>("check_for_updates")
      .then((result) => {
        setInfo(result);
        setStatus(result.available ? "ready" : "idle");
      })
      .catch((err) => {
        setError(String(err));
        setStatus("error");
      })
      .finally(() => setBusy(false));

    const unsubs: Array<() => void> = [];
    void listen<{ phase: string; message?: string }>("update:status", (event) => {
      setStatus(event.payload.phase);
      if (event.payload.message) {
        setError(event.payload.message);
      }
    }).then((unlisten) => unsubs.push(unlisten));

    return () => unsubs.forEach((fn) => fn());
  }, [open]);

  const install = async () => {
    setBusy(true);
    setError(null);
    try {
      await invoke("install_update");
    } catch (err) {
      setError(String(err));
      setBusy(false);
    }
  };

  if (!open) return null;

  return (
    <div className="dialog-overlay" role="presentation" onClick={onClose}>
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="update-title"
        onClick={(event) => event.stopPropagation()}
      >
        <h2 id="update-title">Check for updates</h2>
        <p className="muted">Status: {status}</p>
        {info && (
          <div className="update-info">
            <p>Current: v{info.current_version}</p>
            {info.latest_version && <p>Latest: v{info.latest_version}</p>}
            {info.notes && <pre className="update-notes">{info.notes}</pre>}
          </div>
        )}
        {error && <p className="error-text">{error}</p>}
        <div className="dialog-actions">
          <button type="button" onClick={onClose} disabled={busy}>
            Close
          </button>
          {info?.available && (
            <button type="button" className="primary-btn" onClick={() => void install()} disabled={busy}>
              Install and restart
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
