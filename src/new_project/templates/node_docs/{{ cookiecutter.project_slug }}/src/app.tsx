import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

import {
  defaultDocumentSlug,
  documents,
  findDocument,
  navigation,
  type DocumentEntry,
  type NavigationNode,
} from "./content-loader";

export default function App() {
  const [currentSlug, setCurrentSlug] = createSignal(resolveInitialSlug());

  const activeDocument = createMemo(() => {
    const slug = currentSlug();
    return findDocument(slug) ?? documents[0];
  });

  onMount(() => {
    const updateFromHash = () => {
      const nextSlug = readSlugFromHash();
      if (nextSlug) {
        setCurrentSlug(nextSlug);
      }
    };

    window.addEventListener("hashchange", updateFromHash);
    onCleanup(() => {
      window.removeEventListener("hashchange", updateFromHash);
    });
  });

  return (
    <div class="app-shell">
      <aside class="sidebar">
        <div class="brand">
          <p class="eyebrow">Docs</p>
          <h1>{{ cookiecutter.site_title }}</h1>
          <p class="summary">
            Markdown documents with YAML front matter, organized by folders.
          </p>
        </div>

        <nav class="nav-tree" aria-label="Document navigation">
          <For each={navigation}>{(node) => <NavigationSection node={node} activeSlug={currentSlug()} />}</For>
        </nav>
      </aside>

      <main class="content-pane">
        <Show
          when={activeDocument()}
          fallback={<div class="empty-state">No documents were found.</div>}
        >
          {(document) => <DocumentView document={document()} />}
        </Show>
      </main>
    </div>
  );
}

function NavigationSection(props: {
  node: NavigationNode;
  activeSlug: string;
}) {
  return (
    <section class="nav-section">
      <h2>{props.node.name}</h2>
      <ul>
        <For each={props.node.documents}>
          {(document) => <NavigationLink document={document} activeSlug={props.activeSlug} />}
        </For>
      </ul>
      <For each={props.node.children}>
        {(child) => <NavigationSection node={child} activeSlug={props.activeSlug} />}
      </For>
    </section>
  );
}

function NavigationLink(props: {
  document: DocumentEntry;
  activeSlug: string;
}) {
  const href = `#/${props.document.slug}`;

  return (
    <li>
      <a
        href={href}
        class={props.document.slug === props.activeSlug ? "active" : ""}
      >
        <span>{props.document.title}</span>
        <Show when={props.document.summary}>
          <small>{props.document.summary}</small>
        </Show>
      </a>
    </li>
  );
}

function DocumentView(props: { document: DocumentEntry }) {
  return (
    <article class="document">
      <header class="document-header">
        <p class="document-path">{props.document.pathSegments.join(" / ")}</p>
        <h1>{props.document.title}</h1>
        <Show when={props.document.summary}>
          <p class="document-summary">{props.document.summary}</p>
        </Show>
      </header>

      <div class="markdown-body" innerHTML={props.document.html} />
    </article>
  );
}

function resolveInitialSlug(): string {
  const fromHash = readSlugFromHash();
  return fromHash || defaultDocumentSlug();
}

function readSlugFromHash(): string {
  return window.location.hash.replace(/^#\//u, "");
}
