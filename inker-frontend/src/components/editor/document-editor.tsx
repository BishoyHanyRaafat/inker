
import { useCallback, useEffect, useRef } from "react";
import { EditorContent, useEditor } from "@tiptap/react";
import { Markdown } from "@tiptap/markdown";
import StarterKit from "@tiptap/starter-kit";
import Link from "@tiptap/extension-link";
import Underline from "@tiptap/extension-underline";
import Placeholder from "@tiptap/extension-placeholder";
import { Mathematics } from "@tiptap/extension-mathematics";
import { TableKit } from "@tiptap/extension-table";
import { createLowlight } from "lowlight";

import typescript from "highlight.js/lib/languages/typescript";
import javascript from "highlight.js/lib/languages/javascript";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import json from "highlight.js/lib/languages/json";
import bash from "highlight.js/lib/languages/bash";

import {
  Bold,
  Italic,
  Underline as UnderlineIcon,
  Strikethrough,
  Heading1,
  Heading2,
  List,
  ListOrdered,
  Quote,
  Code,
  Code2,
  Link as LinkIcon,
  Minus,
  Undo2,
  Redo2,
  Workflow,
} from "lucide-react";

import { cn } from "../../lib/utils";
import { CodeBlockLowlightMermaid } from "./mermaid-editor-plugin";
import { Admonition } from "./admonition";

interface DocumentEditorProps {
  content: string;
  onChange: (content: string) => void;
  onSave?: () => void;
  placeholder?: string;
  className?: string;
  autoFocus?: boolean;
}

interface ToolbarButtonProps {
  label: string;
  active?: boolean;
  disabled?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}

/**
 * Create Lowlight once instead of recreating it on every render.
 */
const lowlight = createLowlight();

lowlight.register("typescript", typescript);
lowlight.register("javascript", javascript);
lowlight.register("python", python);
lowlight.register("rust", rust);
lowlight.register("json", json);
lowlight.register("bash", bash);

function ToolbarButton({
  label,
  active = false,
  disabled = false,
  onClick,
  children,
}: ToolbarButtonProps) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md",
        "text-muted-foreground transition-colors",
        "hover:bg-accent hover:text-accent-foreground",
        "disabled:pointer-events-none disabled:opacity-40",
        active && "bg-accent text-accent-foreground",
      )}
    >
      {children}
    </button>
  );
}

export function DocumentEditor({
  content,
  onChange,
  onSave,
  placeholder = "Start writing...",
  className,
  autoFocus = false,
}: DocumentEditorProps) {
  const onSaveRef = useRef(onSave);

  useEffect(() => {
    onSaveRef.current = onSave;
  }, [onSave]);

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        codeBlock: false,
      }),

      Markdown,

      Underline,

      Admonition,

      CodeBlockLowlightMermaid.configure({
        lowlight,
        languageClassPrefix: "language-",
        defaultLanguage: null,
        enableTabIndentation: false,
        tabSize: 4,
        exitOnTripleEnter: true,
        exitOnArrowDown: true,
      }),

      Mathematics,

      TableKit.configure({
        table: {
          renderWrapper: true,
          resizable: true,
          HTMLAttributes: {
            class: "editor-table",
          },
        },
      }),

      Link.configure({
        openOnClick: false,
        autolink: true,
        defaultProtocol: "https",
      }),

      Placeholder.configure({
        placeholder,
      }),
    ],

    content,
    contentType: "markdown",
    autofocus: autoFocus,
    immediatelyRender: false,

    editorProps: {
      attributes: {
        class: cn(
          "min-h-[60vh] w-full min-w-0 outline-none",
          "text-base leading-8 text-foreground",
          "prose prose-neutral dark:prose-invert max-w-none",
          "prose-headings:font-semibold prose-headings:tracking-tight",
          "prose-p:my-3 prose-headings:mb-4 prose-headings:mt-8",
          "prose-a:text-primary prose-a:underline",
          "prose-blockquote:border-l-primary",
          "prose-code:rounded prose-code:bg-muted prose-code:px-1",
          "prose-pre:max-w-full prose-pre:overflow-x-auto",
          "prose-pre:bg-muted prose-pre:text-foreground",
          "prose-li:my-1",
          "prose-table:w-full",
          "prose-th:border prose-th:border-border prose-th:bg-muted",
          "prose-td:border prose-td:border-border",
          "[&_img]:max-w-full",
        ),
      },

      handleKeyDown: (_view, event) => {
        if (
          (event.metaKey || event.ctrlKey) &&
          event.key.toLowerCase() === "s"
        ) {
          event.preventDefault();
          onSaveRef.current?.();
          return true;
        }

        return false;
      },
    },

    onUpdate: ({ editor }) => {
      onChange(editor.getMarkdown());
    },
  });

  /**
   * Synchronize externally loaded content without creating another
   * editor update or triggering autosave.
   */
  useEffect(() => {
    if (!editor) return;

    const currentMarkdown = editor.getMarkdown();

    if (currentMarkdown !== content) {
      editor.commands.setContent(content, {
        contentType: "markdown",
        emitUpdate: false,
      });
    }
  }, [editor, content]);

  const setLink = useCallback(() => {
    if (!editor) return;

    const previousUrl = editor.getAttributes("link").href as
      | string
      | undefined;

    const url = window.prompt("Enter URL", previousUrl ?? "https://");

    if (url === null) return;

    if (!url.trim()) {
      editor.chain().focus().extendMarkRange("link").unsetLink().run();
      return;
    }

    editor
      .chain()
      .focus()
      .extendMarkRange("link")
      .setLink({ href: url.trim() })
      .run();
  }, [editor]);

  if (!editor) {
    return <div className="min-h-[60vh] w-full" />;
  }

  return (
    <div
      className={cn(
        "w-full min-w-0 overflow-hidden rounded-xl",
        "border border-border bg-card shadow-sm",
        className,
      )}
    >
      {/* Formatting toolbar */}
      <div
        className={cn(
          "sticky top-0 z-10 flex flex-wrap items-center gap-1",
          "border-b border-border bg-card/95 p-2 backdrop-blur",
        )}
      >
        <ToolbarButton
          label="Bold"
          active={editor.isActive("bold")}
          onClick={() => editor.chain().focus().toggleBold().run()}
        >
          <Bold className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Italic"
          active={editor.isActive("italic")}
          onClick={() => editor.chain().focus().toggleItalic().run()}
        >
          <Italic className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Underline"
          active={editor.isActive("underline")}
          onClick={() => editor.chain().focus().toggleUnderline().run()}
        >
          <UnderlineIcon className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Strikethrough"
          active={editor.isActive("strike")}
          onClick={() => editor.chain().focus().toggleStrike().run()}
        >
          <Strikethrough className="h-4 w-4" />
        </ToolbarButton>

        <div className="mx-1 h-5 w-px shrink-0 bg-border" />

        <ToolbarButton
          label="Heading 1"
          active={editor.isActive("heading", { level: 1 })}
          onClick={() =>
            editor.chain().focus().toggleHeading({ level: 1 }).run()
          }
        >
          <Heading1 className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Heading 2"
          active={editor.isActive("heading", { level: 2 })}
          onClick={() =>
            editor.chain().focus().toggleHeading({ level: 2 }).run()
          }
        >
          <Heading2 className="h-4 w-4" />
        </ToolbarButton>

        <div className="mx-1 h-5 w-px shrink-0 bg-border" />

        <ToolbarButton
          label="Bullet list"
          active={editor.isActive("bulletList")}
          onClick={() => editor.chain().focus().toggleBulletList().run()}
        >
          <List className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Numbered list"
          active={editor.isActive("orderedList")}
          onClick={() => editor.chain().focus().toggleOrderedList().run()}
        >
          <ListOrdered className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Quote"
          active={editor.isActive("blockquote")}
          onClick={() => editor.chain().focus().toggleBlockquote().run()}
        >
          <Quote className="h-4 w-4" />
        </ToolbarButton>

        <div className="mx-1 h-5 w-px shrink-0 bg-border" />

        <ToolbarButton
          label="Inline code"
          active={editor.isActive("code")}
          onClick={() => editor.chain().focus().toggleCode().run()}
        >
          <Code className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Code block"
          active={
            editor.isActive("codeBlock") &&
            editor.getAttributes("codeBlock").language !== "mermaid"
          }
          onClick={() => editor.chain().focus().toggleCodeBlock().run()}
        >
          <Code2 className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Mermaid diagram"
          active={editor.isActive("codeBlock", { language: "mermaid" })}
          onClick={() =>
            editor
              .chain()
              .focus()
              .toggleCodeBlock({ language: "mermaid" })
              .run()
          }
        >
          <Workflow className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Insert link"
          active={editor.isActive("link")}
          onClick={setLink}
        >
          <LinkIcon className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Horizontal divider"
          onClick={() => editor.chain().focus().setHorizontalRule().run()}
        >
          <Minus className="h-4 w-4" />
        </ToolbarButton>

        <div className="mx-1 h-5 w-px shrink-0 bg-border" />

        <ToolbarButton
          label="Undo"
          disabled={!editor.can().undo()}
          onClick={() => editor.chain().focus().undo().run()}
        >
          <Undo2 className="h-4 w-4" />
        </ToolbarButton>

        <ToolbarButton
          label="Redo"
          disabled={!editor.can().redo()}
          onClick={() => editor.chain().focus().redo().run()}
        >
          <Redo2 className="h-4 w-4" />
        </ToolbarButton>
      </div>

      {/* Editing surface */}
      <div className="min-w-0 px-5 py-8 sm:px-10">
        <EditorContent editor={editor} />
      </div>
    </div>
  );
}
