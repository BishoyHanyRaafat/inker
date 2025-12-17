"use client";

import { useEffect, useState, useCallback, use } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { isAuthenticated } from "@/lib/api-config";
import { getNoteById, getBlocksByNoteId, createBlock } from "@/lib/api";
import type { Model, NoteBlock } from "@/lib/api/types.gen";
import { DocumentEditor } from "@/components/editor/document-editor";
import { Logo } from "@/components/ui/logo";
import { PageLoader, Spinner } from "@/components/ui/spinner";
import { Button } from "@/components/ui/button";
import {
  Pin,
  Archive,
  MoreHorizontal,
  Save,
  Check,
  Clock,
  ArrowLeft,
  Home,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface NoteEditorPageProps {
  params: Promise<{ id: string }>;
}

// Convert blocks to HTML content for the editor
function blocksToHtml(blocks: NoteBlock[]): string {
  if (!blocks || blocks.length === 0) return "";

  return blocks
    .sort((a, b) => a.order - b.order)
    .map((block) => {
      switch (block.block.type) {
        case "text":
          const segments = block.block.data.segments || [];
          const text = segments
            .map((seg) => {
              let html = seg.text;
              seg.marks?.forEach((mark) => {
                switch (mark) {
                  case "Bold":
                    html = `<strong>${html}</strong>`;
                    break;
                  case "Italic":
                    html = `<em>${html}</em>`;
                    break;
                  case "Underline":
                    html = `<u>${html}</u>`;
                    break;
                  case "StrikeThrough":
                    html = `<s>${html}</s>`;
                    break;
                  case "Code":
                    html = `<code>${html}</code>`;
                    break;
                  case "Highlight":
                    html = `<mark>${html}</mark>`;
                    break;
                }
              });
              return html;
            })
            .join("");
          return `<p>${text || "<br>"}</p>`;

        case "todo":
          const items = block.block.data.items || [];
          return `<ul>${items
            .map((item) => {
              const checked = item.startsWith("[x]") || item.startsWith("[X]");
              const text = item.replace(/^\[.\]\s*/, "");
              return `<li>${checked ? "✓ " : "☐ "}${text}</li>`;
            })
            .join("")}</ul>`;

        case "divider":
          return "<hr>";

        case "image":
          return block.block.data.url
            ? `<p><img src="${block.block.data.url}" alt="" style="max-width: 100%;" /></p>`
            : "";

        default:
          return "";
      }
    })
    .join("\n");
}

// Convert HTML content back to blocks for saving
function htmlToBlocks(html: string, noteId: string): Array<{
  content: NoteBlock["block"];
  note_id: string;
  order: number;
}> {
  // For now, treat the entire content as one text block
  // A more sophisticated implementation would parse the HTML properly
  const tempDiv = typeof document !== "undefined" ? document.createElement("div") : null;
  if (!tempDiv) return [];
  
  tempDiv.innerHTML = html;
  
  const blocks: Array<{
    content: NoteBlock["block"];
    note_id: string;
    order: number;
  }> = [];
  
  let order = 0;
  
  // Simple conversion - each paragraph becomes a text block
  const elements = tempDiv.children;
  for (let i = 0; i < elements.length; i++) {
    const el = elements[i];
    
    if (el.tagName === "HR") {
      blocks.push({
        content: { type: "divider", data: {} as never },
        note_id: noteId,
        order: order++,
      });
    } else {
      // Convert element to text segments
      const text = el.textContent || "";
      if (text.trim() || el.innerHTML.includes("<img")) {
        blocks.push({
          content: {
            type: "text",
            data: {
              segments: [{ text, marks: [] }],
            },
          },
          note_id: noteId,
          order: order++,
        });
      }
    }
  }
  
  return blocks;
}

export default function NoteEditorPage({ params }: NoteEditorPageProps) {
  const { id: noteId } = use(params);
  const router = useRouter();
  const [note, setNote] = useState<Model | null>(null);
  const [content, setContent] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [title, setTitle] = useState("");
  const [hasChanges, setHasChanges] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Check authentication
  useEffect(() => {
    if (!isAuthenticated()) {
      router.push("/login");
    }
  }, [router]);

  // Load note and blocks
  useEffect(() => {
    const loadNote = async () => {
      try {
        setIsLoading(true);
        setError(null);

        // Load note details
        const noteResponse = await getNoteById({
          path: { note_id: noteId },
        });

        if (!noteResponse.data?.data) {
          setError("Note not found");
          return;
        }

        const noteData = noteResponse.data.data;
        setNote(noteData);
        setTitle(noteData.title);

        // Load blocks and convert to HTML
        const blocksResponse = await getBlocksByNoteId({
          path: { note_id: noteId },
        });

        if (blocksResponse.data?.data) {
          const html = blocksToHtml(blocksResponse.data.data);
          setContent(html);
        }
      } catch (err) {
        console.error("Failed to load note:", err);
        setError("Failed to load note");
      } finally {
        setIsLoading(false);
      }
    };

    loadNote();
  }, [noteId]);

  // Handle content changes
  const handleContentChange = useCallback((newContent: string) => {
    setContent(newContent);
    setHasChanges(true);
  }, []);

  // Save content
  const handleSave = useCallback(async () => {
    if (!note || !hasChanges) return;

    setIsSaving(true);
    try {
      // Convert HTML to blocks and save
      const blocks = htmlToBlocks(content, note.id);
      
      // For now, just save the first block (simplified)
      // A full implementation would sync all blocks
      if (blocks.length > 0) {
        await createBlock({
          body: {
            note_id: note.id,
            content: blocks[0].content,
            order: 0,
          },
        });
      }

      setLastSaved(new Date());
      setHasChanges(false);
    } catch (err) {
      console.error("Failed to save:", err);
    } finally {
      setIsSaving(false);
    }
  }, [note, content, hasChanges]);

  // Auto-save after changes (debounced)
  useEffect(() => {
    if (!hasChanges) return;

    const timer = setTimeout(() => {
      handleSave();
    }, 3000);

    return () => clearTimeout(timer);
  }, [hasChanges, handleSave]);

  if (!isAuthenticated()) {
    return <PageLoader text="Redirecting to login..." />;
  }

  if (isLoading) {
    return (
      <div className="min-h-screen bg-background bg-gradient-hero flex items-center justify-center">
        <Spinner size="lg" text="Loading note..." />
      </div>
    );
  }

  if (error || !note) {
    return (
      <div className="min-h-screen bg-background bg-gradient-hero flex flex-col items-center justify-center gap-4">
        <p className="text-muted-foreground">{error || "Note not found"}</p>
        <Button onClick={() => router.push("/dashboard")} variant="outline">
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back to Dashboard
        </Button>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-50 border-b border-border/50 bg-background/80 backdrop-blur-md">
        <div className="container mx-auto px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link
              href="/dashboard"
              className="flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors"
            >
              <ArrowLeft className="w-4 h-4" />
              <span className="hidden sm:inline">Back</span>
            </Link>
            
            <div className="h-4 w-px bg-border" />
            
            <Logo href="/dashboard" size="sm" showText={false} />
          </div>

          <div className="flex items-center gap-3">
            {/* Save status */}
            <div className="hidden sm:flex items-center gap-2 text-sm text-muted-foreground">
              {isSaving ? (
                <>
                  <Spinner size="sm" />
                  <span>Saving...</span>
                </>
              ) : lastSaved ? (
                <>
                  <Check className="w-4 h-4 text-green-500" />
                  <span>Saved</span>
                </>
              ) : hasChanges ? (
                <>
                  <Clock className="w-4 h-4" />
                  <span>Unsaved</span>
                </>
              ) : null}
            </div>

            <Button
              variant="ghost"
              size="icon-sm"
              className={cn(
                "text-muted-foreground hover:text-foreground",
                note.pinned && "text-primary"
              )}
              title={note.pinned ? "Unpin note" : "Pin note"}
            >
              <Pin className={cn("w-4 h-4", note.pinned && "fill-current")} />
            </Button>
            
            <Button
              onClick={handleSave}
              disabled={isSaving || !hasChanges}
              size="sm"
            >
              <Save className="w-4 h-4 mr-2" />
              Save
            </Button>
          </div>
        </div>
      </header>

      {/* Editor */}
      <main className="container mx-auto px-4 py-8">
        <div className="max-w-4xl mx-auto">
          {/* Title */}
          <input
            type="text"
            value={title}
            onChange={(e) => {
              setTitle(e.target.value);
              setHasChanges(true);
            }}
            placeholder="Untitled"
            className="w-full text-4xl font-bold bg-transparent border-none outline-none placeholder:text-muted-foreground/30 mb-2"
          />

          {/* Metadata */}
          <div className="flex items-center gap-4 text-sm text-muted-foreground mb-8">
            <span>
              Created{" "}
              {new Date(note.created_at).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
            <span>•</span>
            <span>
              Updated{" "}
              {new Date(note.updated_at).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
              })}
            </span>
          </div>

          {/* Document Editor */}
          <DocumentEditor
            content={content}
            onChange={handleContentChange}
            onSave={handleSave}
            placeholder="Start writing your note..."
            autoFocus
          />
        </div>
      </main>
    </div>
  );
}
