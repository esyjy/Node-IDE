import type { StackMessage } from "../hooks/useMessageStack";

interface MessageStackProps {
  messages: StackMessage[];
  onDismiss: (id: string) => void;
}

export function MessageStack({ messages, onDismiss }: MessageStackProps) {
  if (messages.length === 0) {
    return null;
  }

  return (
    <div className="message-stack" role="status" aria-live="polite">
      {messages.map((message) => (
        <div key={message.id} className="toast">
          <p className="toast-text">{message.text}</p>
          <button
            type="button"
            className="toast-dismiss"
            aria-label="Dismiss message"
            onClick={() => onDismiss(message.id)}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
