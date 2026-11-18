
import { useEffect, useState } from "react";

import CodeBlockLowlight, {
  type CodeBlockLowlightOptions,
} from "@tiptap/extension-code-block-lowlight";

import {
  NodeViewContent,
  NodeViewWrapper,
  ReactNodeViewRenderer,
  type NodeViewProps,
} from "@tiptap/react";

import mermaid from "mermaid";

import {
  AlertCircle,
  Eye,
  LoaderCircle,
  Pencil,
  Workflow,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { converter, formatRgb } from "culori";

const toRgb = converter("rgb");

/**
 * Mermaid uses global configuration.
 * Serialize rendering to prevent different diagrams from changing
 * each other's theme configuration during concurrent renders.
 */
let renderQueue: Promise<unknown> = Promise.resolve();
let renderCounter = 0;

function roundMermaidNodes(svg: string, radius = 8): string {
  const parser = new DOMParser();
  const document = parser.parseFromString(svg, "image/svg+xml");

  document.querySelectorAll("g.node > rect").forEach((rect) => {
    rect.setAttribute("rx", String(radius));
    rect.setAttribute("ry", String(radius));
  });

  return new XMLSerializer().serializeToString(document);
}

function toMermaidColor(value: string): string {
  const color = toRgb(value);

  if (!color) {
    throw new Error(`Invalid theme color: ${value}`);
  }

  return formatRgb(color);
}

/**
 * Resolve actual browser colors from your Tailwind semantic classes.
 *
 * This works with CSS variables whether they use HSL, OKLCH,
 * RGB, or another browser-supported color representation.
 */
function getMermaidThemeVariables() {
  const styles = getComputedStyle(document.documentElement);
  const get = (name: string) =>
    toMermaidColor(styles.getPropertyValue(name).trim());

  return {
    background: get("--card"),

    primaryColor: get("--primary"),
    primaryTextColor: get("--primary-foreground"),
    primaryBorderColor: get("--border"),

    secondaryColor: get("--secondary"),
    secondaryTextColor: get("--secondary-foreground"),
    secondaryBorderColor: get("--border"),

    tertiaryColor: get("--muted"),
    tertiaryTextColor: get("--muted-foreground"),
    tertiaryBorderColor: get("--border"),

    mainBkg: get("--card"),
    nodeBorder: get("--border"),
    lineColor: get("--muted-foreground"),

    // Explicit flowchart text colors
    nodeTextColor: get("--foreground"),

    clusterBkg: get("--muted"),
    clusterBorder: get("--border"),

    titleColor: get("--foreground"),
    edgeLabelBackground: get("--card"),

    noteBkgColor: get("--secondary"),
    noteTextColor: get("--secondary-foreground"),
    noteBorderColor: get("--border"),

    fontFamily: "Inter Variable, sans-serif",
    fontSize: "14px",
    rectRadius: "20px",
  };
}

function renderMermaid(
  source: string,
  themeVariables: ReturnType<typeof getMermaidThemeVariables>,
) {
  const id = `mermaid-render-${++renderCounter}`;

  const task = renderQueue.then(async () => {
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      theme: "base",
      themeVariables,
      flowchart: {
        curve: "basis",
        padding: 12,
        htmlLabels: true,
      },
    });

    return mermaid.render(id, source);
  });

  renderQueue = task.then(
    () => undefined,
    () => undefined,
  );

  return task;
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

function MermaidCodeBlockView({ node }: NodeViewProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [svg, setSvg] = useState("");
  const [renderError, setRenderError] = useState<string | null>(null);
  const [isRendering, setIsRendering] = useState(false);
  const [themeRevision, setThemeRevision] = useState(0);

  const language =
    typeof node.attrs.language === "string"
      ? node.attrs.language.toLowerCase()
      : "";

  const isMermaid = language === "mermaid";
  const source = node.textContent;

  /**
   * Watch the document theme.
   * Tailwind's dark class or theme-related root attributes can change.
   */
  useEffect(() => {
    const root = document.documentElement;

    const observer = new MutationObserver(() => {
      setThemeRevision((revision) => revision + 1);
    });

    observer.observe(root, {
      attributes: true,
      attributeFilter: ["class", "style", "data-theme"],
    });

    return () => observer.disconnect();
  }, []);

  /**
   * Render Mermaid whenever its source or theme changes.
   */
  useEffect(() => {
    if (!isMermaid || isEditing) {
      return;
    }

    if (!source.trim()) {
      setSvg("");
      setRenderError(null);
      setIsRendering(false);
      return;
    }

    let cancelled = false;

    async function render() {
      setIsRendering(true);
      setRenderError(null);

      try {
        const themeVariables = getMermaidThemeVariables();
        const result = await renderMermaid(source, themeVariables);

        if (cancelled) return;

        setSvg(roundMermaidNodes(result.svg, 20));
        setRenderError(null);
      } catch (error) {
        if (cancelled) return;

        setSvg("");
        setRenderError(getErrorMessage(error));
      } finally {
        if (!cancelled) {
          setIsRendering(false);
        }
      }
    }

    render();

    return () => {
      cancelled = true;
    };
  }, [source, isMermaid, isEditing, themeRevision]);

  /**
   * Normal code blocks retain their normal editable appearance.
   */
  if (!isMermaid) {
    return (
      <NodeViewWrapper className="my-5 min-w-0 max-w-full">
        <pre className="max-w-full overflow-x-auto rounded-xl border border-border bg-muted p-4 text-sm leading-6">
          <NodeViewContent />
        </pre>
      </NodeViewWrapper>
    );
  }

  return (
    <NodeViewWrapper className="my-6 min-w-0 max-w-full">
      <div className="overflow-hidden rounded-xl border border-border bg-card">
        {/* Diagram header */}
        <div className="flex items-center justify-between gap-3 border-b border-border bg-muted/40 px-4 py-2.5">
          <div className="flex min-w-0 items-center gap-2">
            <Workflow className="h-4 w-4 shrink-0 text-muted-foreground" />

            <span className="text-sm font-medium">Mermaid diagram</span>

            {!isEditing && !renderError && (
              <span className="rounded-md bg-secondary px-2 py-0.5 text-[10px] font-medium text-secondary-foreground">
                Preview
              </span>
            )}

            {isEditing && (
              <span className="rounded-md bg-secondary px-2 py-0.5 text-[10px] font-medium text-secondary-foreground">
                Editing source
              </span>
            )}
          </div>

          <button
            type="button"
            onClick={() => setIsEditing((value) => !value)}
            className="inline-flex shrink-0 items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
          >
            {isEditing ? (
              <>
                <Eye className="h-3.5 w-3.5" />
                Preview
              </>
            ) : (
              <>
                <Pencil className="h-3.5 w-3.5" />
                Edit source
              </>
            )}
          </button>
        </div>

        {/* Keep NodeViewContent mounted so ProseMirror retains its content DOM. */}
        <pre
          className={
            isEditing
              ? "max-w-full overflow-x-auto bg-muted/30 p-4 text-sm leading-6"
              : "hidden"
          }
        >
          <NodeViewContent />
        </pre>

        {/* Rendered diagram */}
        {!isEditing && (
          <div className="min-w-0 p-3 sm:p-5">
            {isRendering ? (
              <div className="flex min-h-32 items-center justify-center gap-2 text-sm text-muted-foreground">
                <LoaderCircle className="h-4 w-4 animate-spin" />
                Rendering diagram...
              </div>
            ) : renderError ? (
              <div
                role="alert"
                className="rounded-lg border border-destructive/30 bg-destructive/5 p-4"
              >
                <div className="flex items-start gap-3">
                  <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-destructive" />

                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium text-foreground">
                      Couldn't render this diagram
                    </p>

                    <p className="mt-1 text-xs text-muted-foreground">
                      There is a problem with the Mermaid syntax. Your source
                      is safe; edit it to fix the diagram.
                    </p>

                    <details className="mt-3">
                      <summary className="cursor-pointer text-xs font-medium text-muted-foreground hover:text-foreground">
                        Show error details
                      </summary>

                      <pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap wrap-break-word rounded-md bg-muted p-3 text-xs text-muted-foreground">
                        {renderError}
                      </pre>
                    </details>

                    <button
                      type="button"
                      onClick={() => setIsEditing(true)}
                      className="mt-3 inline-flex items-center gap-1.5 text-xs font-medium text-primary hover:underline"
                    >
                      <Pencil className="h-3.5 w-3.5" />
                      Edit Mermaid source
                    </button>
                  </div>
                </div>
              </div>
            ) : !source.trim() ? (
              <button
                type="button"
                onClick={() => setIsEditing(true)}
                className="flex min-h-32 w-full flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border text-sm text-muted-foreground transition-colors hover:border-primary/50 hover:bg-muted/40 hover:text-foreground"
              >
                <Workflow className="h-6 w-6" />
                <span>Add Mermaid diagram source</span>
              </button>
            ) : (
              <div
                className={cn(
                  "mermaid-preview min-w-0 max-w-full overflow-x-auto",
                  "[&_svg]:mx-auto [&_svg]:block [&_svg]:h-auto [&_svg]:max-w-full",
                )}
                dangerouslySetInnerHTML={{ __html: svg }}
              />
            )}
          </div>
        )}
      </div>
    </NodeViewWrapper>
  );
}

export const CodeBlockLowlightMermaid =
  CodeBlockLowlight.extend<CodeBlockLowlightOptions>({
    addAttributes() {
      const parentAttributes = this.parent?.() ?? {};

      return {
        ...parentAttributes,

        id: {
          default: null,

          parseHTML: (element) => element.getAttribute("data-id"),

          renderHTML: (attributes) =>
            attributes.id
              ? { "data-id": attributes.id }
              : {},
        },
      };
    },

    addNodeView() {
      return ReactNodeViewRenderer(MermaidCodeBlockView);
    },
  });
