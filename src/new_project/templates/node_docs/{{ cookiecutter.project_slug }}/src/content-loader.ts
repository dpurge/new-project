import { marked } from "marked";
import YAML from "yaml";

export type DocumentMetadata = {
  title?: string;
  summary?: string;
  order?: number;
};

export type DocumentEntry = {
  slug: string;
  title: string;
  summary: string;
  order: number;
  pathSegments: string[];
  sectionSegments: string[];
  html: string;
};

export type NavigationNode = {
  name: string;
  slug: string;
  children: NavigationNode[];
  documents: DocumentEntry[];
};

const documentModules = import.meta.glob("./content/**/*.md", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

export const documents = Object.entries(documentModules)
  .map(([path, source]) => createDocument(path, source))
  .sort((left, right) => {
    if (left.order !== right.order) {
      return left.order - right.order;
    }

    return left.title.localeCompare(right.title);
  });

export const navigation = buildNavigationTree(documents);

export function findDocument(slug: string): DocumentEntry | undefined {
  return documents.find((document) => document.slug === slug);
}

export function defaultDocumentSlug(): string {
  return documents[0]?.slug ?? "";
}

function createDocument(path: string, source: string): DocumentEntry {
  const relativePath = path
    .replace(/^\.\/content\//u, "")
    .replace(/\.md$/u, "");
  const pathSegments = relativePath.split("/");
  const sectionSegments = pathSegments.slice(0, -1);
  const { metadata, markdown } = parseFrontMatter(source);
  const slug = pathSegments.join("/");

  return {
    slug,
    title: metadata.title ?? humanizeSegment(pathSegments[pathSegments.length - 1]),
    summary: metadata.summary ?? "",
    order: metadata.order ?? 100,
    pathSegments,
    sectionSegments,
    html: marked.parse(markdown) as string,
  };
}

function parseFrontMatter(source: string): {
  metadata: DocumentMetadata;
  markdown: string;
} {
  if (!source.startsWith("---\n") && !source.startsWith("---\r\n")) {
    return { metadata: {}, markdown: source };
  }

  const normalized = source.replace(/\r\n/g, "\n");
  const closingIndex = normalized.indexOf("\n---\n", 4);
  if (closingIndex === -1) {
    return { metadata: {}, markdown: source };
  }

  const frontMatter = normalized.slice(4, closingIndex);
  const markdown = normalized.slice(closingIndex + 5);
  const parsed = YAML.parse(frontMatter);

  return {
    metadata: isDocumentMetadata(parsed) ? parsed : {},
    markdown,
  };
}

function buildNavigationTree(allDocuments: DocumentEntry[]): NavigationNode[] {
  const roots: NavigationNode[] = [];

  for (const document of allDocuments) {
    let currentLevel = roots;
    let currentNode: NavigationNode | undefined;
    const traversedSegments: string[] = [];

    for (const segment of document.sectionSegments) {
      traversedSegments.push(segment);

      let node = currentLevel.find(
        (candidate) => candidate.slug === traversedSegments.join("/"),
      );
      if (!node) {
        node = {
          name: humanizeSegment(segment),
          slug: traversedSegments.join("/"),
          children: [],
          documents: [],
        };
        currentLevel.push(node);
      }

      currentNode = node;
      currentLevel = node.children;
    }

    if (currentNode) {
      currentNode.documents.push(document);
    } else {
      let rootNode = roots.find((node) => node.slug === "__root__");
      if (!rootNode) {
        rootNode = {
          name: "Documents",
          slug: "__root__",
          children: [],
          documents: [],
        };
        roots.push(rootNode);
      }
      rootNode.documents.push(document);
    }
  }

  return sortNavigation(roots);
}

function sortNavigation(nodes: NavigationNode[]): NavigationNode[] {
  return nodes
    .map((node) => ({
      ...node,
      children: sortNavigation(node.children),
      documents: [...node.documents].sort((left, right) => {
        if (left.order !== right.order) {
          return left.order - right.order;
        }

        return left.title.localeCompare(right.title);
      }),
    }))
    .sort((left, right) => left.name.localeCompare(right.name));
}

function humanizeSegment(segment: string): string {
  return segment
    .replace(/[-_]/g, " ")
    .replace(/\b\w/g, (character) => character.toUpperCase());
}

function isDocumentMetadata(value: unknown): value is DocumentMetadata {
  return typeof value === "object" && value !== null;
}
