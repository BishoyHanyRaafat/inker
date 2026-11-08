import { useEffect, useState, useCallback, useRef } from "react";
import { useNavigate, useParams, Link } from "react-router-dom";
import { isAuthenticated } from "../lib/api-config";
import { createNoteContent, getNoteById, getNoteContentByNoteId, patchNoteContent } from "../lib/api";
import type { Note } from "../lib/api/types.gen";
import { DocumentEditor } from "@/components/editor/document-editor";
import { Logo } from "@/components/ui/logo";
import { PageLoader, Spinner } from "@/components/ui/spinner";
import { Button } from "@/components/ui/button";
import {
  Pin,
  Save,
  Check,
  Clock,
  ArrowLeft,
} from "lucide-react";
import { cn } from "../lib/utils";

export default function NoteEditorPage() {
  const { id: noteId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [note, setNote] = useState<Note | null>(null);
  const [content, setContent] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [title, setTitle] = useState("");
  const [hasChanges, setHasChanges] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const contentRef = useRef(content);

  // Check authentication
  useEffect(() => {
    if (!isAuthenticated()) {
      navigate("/login");
    }
  }, [navigate]);

  // Load note and content
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

        // Load markdown content
        const contentResponse = await getNoteContentByNoteId({
          path: { note_id: noteId },
        });

        if (contentResponse.data?.data) {
          const contentData = contentResponse.data.data as any;
          const loadedContent = contentData.content || "";

          contentRef.current = loadedContent;
          setContent(loadedContent);
        } else if (contentResponse.response.status == 404) {
          await createNoteContent({ "body": { content: "", note_id: noteData.id } })
          contentRef.current = "";
          setContent("");
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
    contentRef.current = newContent;
    setContent(newContent);
    setHasChanges(true);
  }, []);

  // Save content

  const handleSave = useCallback(async () => {
    if (!note || !hasChanges || isSaving) return;

    const contentToSave = contentRef.current;

    setIsSaving(true);
    console.log("HIII")

    try {
      await patchNoteContent({
        body: {
          content: contentToSave,
          id: note.id,
          version: 1,
        },
      });

      setLastSaved(new Date());

      if (contentRef.current === contentToSave) {
        setHasChanges(false);
      }
    } catch (err) {
      console.error("Failed to save:", err);
    } finally {
      setIsSaving(false);
    }
  }, [note, hasChanges, isSaving]);

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
        <Button onClick={() => navigate("/dashboard")} variant="outline">
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
              to="/dashboard"
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
