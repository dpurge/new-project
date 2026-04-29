import { For, Show, createMemo, createSignal } from "solid-js";

type ChatRole = "system" | "user" | "assistant";

type ChatMessage = {
  role: ChatRole;
  content: string;
};

export default function App() {
  const [messages, setMessages] = createSignal<ChatMessage[]>([
    {
      role: "assistant",
      content: "Connected to `/v1/chat/completions`. Ask a question to start a streamed reply.",
    },
  ]);
  const [prompt, setPrompt] = createSignal("");
  const [errorMessage, setErrorMessage] = createSignal("");
  const [isStreaming, setIsStreaming] = createSignal(false);

  const canSend = createMemo(() => prompt().trim().length > 0 && !isStreaming());

  const submitPrompt = async (event: SubmitEvent) => {
    event.preventDefault();
    if (!canSend()) {
      return;
    }

    const content = prompt().trim();
    const nextMessages = [...messages(), { role: "user" as const, content }];
    setMessages([...nextMessages, { role: "assistant", content: "" }]);
    setPrompt("");
    setErrorMessage("");
    setIsStreaming(true);

    try {
      const response = await fetch("/v1/chat/completions", {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify({
          stream: true,
          messages: nextMessages,
        }),
      });

      if (!response.ok) {
        throw new Error(await response.text());
      }

      if (!response.body) {
        throw new Error("The chat response did not include a readable stream.");
      }

      await consumeSse(response.body, (delta) => {
        if (!delta) {
          return;
        }

        setMessages((current) => {
          const updated = [...current];
          const lastIndex = updated.length - 1;
          updated[lastIndex] = {
            role: "assistant",
            content: `${updated[lastIndex]?.content ?? ""}${delta}`,
          };
          return updated;
        });
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Streaming request failed.";
      setErrorMessage(message);
      setMessages((current) => {
        const updated = [...current];
        const lastIndex = updated.length - 1;
        if (updated[lastIndex]?.role === "assistant" && !updated[lastIndex]?.content) {
          updated[lastIndex] = {
            role: "assistant",
            content: "The request failed before a response arrived.",
          };
        }
        return updated;
      });
    } finally {
      setIsStreaming(false);
    }
  };

  return (
    <main class="shell">
      <section class="hero">
        <p class="kicker">Streaming Chat</p>
        <h1>{{ cookiecutter.site_title }}</h1>
        <p class="summary">
          Static SolidJS frontend, runtime proxied to an OpenAI-compatible backend such as `llm-api`.
        </p>
      </section>

      <section class="chat-panel">
        <header class="chat-header">
          <div>
            <p class="label">Transport</p>
            <strong>OpenAI-style SSE over `/v1/chat/completions`</strong>
          </div>
          <span class={`status${isStreaming() ? " live" : ""}`}>
            {isStreaming() ? "Streaming" : "Idle"}
          </span>
        </header>

        <div class="messages" aria-live="polite">
          <For each={messages()}>
            {(message) => (
              <article class={`bubble ${message.role}`}>
                <p class="role">{message.role}</p>
                <p class="content">{message.content}</p>
              </article>
            )}
          </For>
        </div>

        <Show when={errorMessage()}>
          {(message) => <p class="error">{message()}</p>}
        </Show>

        <form class="composer" onSubmit={submitPrompt}>
          <label class="composer-label" for="prompt">
            Prompt
          </label>
          <textarea
            id="prompt"
            rows="5"
            value={prompt()}
            onInput={(event) => setPrompt(event.currentTarget.value)}
            placeholder="Ask the model for an explanation, summary, or draft."
          />
          <div class="composer-actions">
            <p>Requests stream directly into the last assistant bubble.</p>
            <button type="submit" disabled={!canSend()}>
              {isStreaming() ? "Streaming..." : "Send"}
            </button>
          </div>
        </form>
      </section>
    </main>
  );
}

async function consumeSse(
  stream: ReadableStream<Uint8Array>,
  onDelta: (delta: string) => void,
) {
  const reader = stream.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }

    buffer += decoder.decode(value, { stream: true });
    const records = buffer.split("\n\n");
    buffer = records.pop() ?? "";

    for (const record of records) {
      for (const line of record.split("\n")) {
        if (!line.startsWith("data:")) {
          continue;
        }

        const payload = line.slice(5).trim();
        if (!payload || payload === "[DONE]") {
          continue;
        }

        const parsed = JSON.parse(payload) as {
          choices?: Array<{
            delta?: {
              content?: string;
            };
            message?: {
              content?: string;
            };
          }>;
        };

        const choice = parsed.choices?.[0];
        onDelta(choice?.delta?.content ?? choice?.message?.content ?? "");
      }
    }
  }
}
