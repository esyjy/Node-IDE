import { useCallback, useState } from "react";

export interface StackMessage {
  id: string;
  text: string;
}

const MAX_MESSAGES = 2;

export function useMessageStack() {
  const [messages, setMessages] = useState<StackMessage[]>([]);

  const pushMessage = useCallback((text: string) => {
    setMessages((current) => {
      const next: StackMessage[] = [
        ...current,
        { id: crypto.randomUUID(), text },
      ];
      if (next.length <= MAX_MESSAGES) {
        return next;
      }
      return next.slice(next.length - MAX_MESSAGES);
    });
  }, []);

  const dismissMessage = useCallback((id: string) => {
    setMessages((current) => current.filter((message) => message.id !== id));
  }, []);

  return { messages, pushMessage, dismissMessage };
}
