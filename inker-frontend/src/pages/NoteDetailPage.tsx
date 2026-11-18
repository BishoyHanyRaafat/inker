
import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import {
  ArrowLeft,
  Check,
  Clock,
  FileText,
  Pin,
  Save,
  Tag,
  Target,
  BookOpen,
  AlignLeft,
} from "lucide-react";

import { isAuthenticated } from "../lib/api-config";
import { getNoteById, patchNote } from "../lib/api";
import type { Note } from "../lib/api/types.gen";

import { DocumentEditor } from "@/components/editor/document-editor";
import { Logo } from "@/components/ui/logo";
import { PageLoader, Spinner } from "@/components/ui/spinner";
import { Button } from "@/components/ui/button";
import { ModeToggle } from "@/components/mode-toggle";

import { cn } from "../lib/utils";

type NoteDraft = {
  title: string;
  content: string;
  description: string;
  subject: string;
  tags: string[];
  learning_objectives: string[];
  coverage_notes: string[];
  pinned: boolean;
};

const toDraft = (note: Note): NoteDraft => ({
  title: note.title ?? "",
  content: note.content ?? "",
  description: note.description ?? "",
  subject: note.subject ?? "",
  tags: note.tags ?? [],
  learning_objectives: note.learning_objectives ?? [],
  coverage_notes: note.coverage_notes ?? [],
  pinned: note.pinned,
});

function MetadataSection({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: React.ElementType;
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="space-y-3">
      <div className="flex items-center gap-2">
        <Icon className="h-4 w-4 text-muted-foreground" />
        <h3 className="text-sm font-semibold">{title}</h3>
      </div>

      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}

      {children}
    </section>
  );
}

function TextareaField({
  value,
  onChange,
  placeholder,
  rows = 3,
}: {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  rows?: number;
}) {
  return (
    <textarea
      value={value}
      onChange={(event) => onChange(event.target.value)}
      placeholder={placeholder}
      rows={rows}
      className="w-full resize-y rounded-lg border border-border bg-background px-3 py-2 text-sm outline-none transition-colors placeholder:text-muted-foreground/50 focus:border-primary"
    />
  );
}

export default function NoteEditorPage() {
  const { id: noteId } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [note, setNote] = useState<Note | null>(null);
  const [draft, setDraft] = useState<NoteDraft | null>(null);

  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Refs keep asynchronous saves synchronized with the latest draft.
  const noteRef = useRef<Note | null>(null);
  const draftRef = useRef<NoteDraft | null>(null);
  const dirtyRef = useRef(false);
  const savingRef = useRef(false);
  const revisionRef = useRef(0);

  // Authentication
  useEffect(() => {
    if (!isAuthenticated()) {
      navigate("/login");
    }
  }, [navigate]);

  // Load note
  useEffect(() => {
    let cancelled = false;

    async function loadNote() {
      if (!isAuthenticated()) return;

      if (!noteId) {
        setError("Invalid note ID");
        setIsLoading(false);
        return;
      }

      try {
        setIsLoading(true);
        setError(null);

        const response = await getNoteById({
          path: { note_id: noteId },
        });

        const noteData = response.data?.data;

        if (!noteData) {
          if (!cancelled) setError("Note not found");
          return;
        }

        if (cancelled) return;

        const initialDraft = toDraft(noteData);

        noteRef.current = noteData;
        draftRef.current = initialDraft;

        dirtyRef.current = false;
        revisionRef.current = 0;

        setNote(noteData);
        setDraft(initialDraft);
        setHasChanges(false);
        setLastSaved(null);
        setSaveError(null);
      } catch (err) {
        console.error("Failed to load note:", err);

        if (!cancelled) {
          setError("Failed to load note. Please try again.");
        }
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    }

    loadNote();

    return () => {
      cancelled = true;
    };
  }, [noteId]);

  // Update a field and mark the draft as dirty.
  function updateField<K extends keyof NoteDraft>(
    key: K,
    value: NoteDraft[K],
  ) {
    const current = draftRef.current;
    if (!current) return;

    const next = {
      ...current,
      [key]: value,
    };

    draftRef.current = next;
    revisionRef.current += 1;
    dirtyRef.current = true;

    setDraft(next);
    setHasChanges(true);
    setSaveError(null);
  }

  // Save the latest draft.
  const handleSave = useCallback(async () => {
    const currentNote = noteRef.current;
    const currentDraft = draftRef.current;

    if (
      !currentNote ||
      !currentDraft ||
      !dirtyRef.current ||
      savingRef.current
    ) {
      return;
    }

    const snapshot = { ...currentDraft };
    const revision = revisionRef.current;

    savingRef.current = true;
    setIsSaving(true);
    setSaveError(null);

    try {
      await patchNote({
        path: { note_id: currentNote.id },
        body: {
          title: snapshot.title,
          content: snapshot.content,
          description: snapshot.description || null,
          subject: snapshot.subject || null,
          tags: snapshot.tags,
          learning_objectives: snapshot.learning_objectives,
          coverage_notes: snapshot.coverage_notes,
          pinned: snapshot.pinned,
          version: currentNote.version,
        },
      });

      setLastSaved(new Date());

      // Only mark clean if nothing changed during the request.
      if (revisionRef.current === revision) {
        dirtyRef.current = false;
        setHasChanges(false);
      }
    } catch (err) {
      console.error("Failed to save note:", err);
      setSaveError("Couldn't save your changes. Try again.");
    } finally {
      savingRef.current = false;
      setIsSaving(false);
    }
  }, []);

  // Debounced autosave.
  useEffect(() => {
    if (!hasChanges) return;

    const timer = window.setTimeout(() => {
      handleSave();
    }, 3000);

    return () => window.clearTimeout(timer);
  }, [draft, hasChanges, isSaving, handleSave]);

  // Ctrl/Cmd + S
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();
        handleSave();
      }
    }

    window.addEventListener("keydown", handleKeyDown);

    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleSave]);

  if (!isAuthenticated()) {
    return <PageLoader text="Redirecting to login..." />;
  }

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <Spinner size="lg" text="Loading note..." />
      </div>
    );
  }

  if (error || !note || !draft) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center gap-4 bg-background">
        <p className="text-muted-foreground">
          {error || "Note not found"}
        </p>

        <Button onClick={() => navigate("/dashboard")} variant="outline">
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to Dashboard
        </Button>
      </div>
    );
  }

  const createdDate = new Date(note.created_at).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

  return (
    <div className="min-h-screen bg-background">
      {/* Top navigation */}
      <header className="sticky top-0 z-50 border-b border-border/60 bg-background/85 backdrop-blur-xl">
        <div className="mx-auto flex h-16 max-w-[1600px] items-center justify-between px-4 sm:px-6">
          <div className="flex items-center gap-4">
            <Link
              to="/dashboard"
              className="flex items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
            >
              <ArrowLeft className="h-4 w-4" />
              <span className="hidden sm:inline">Dashboard</span>
            </Link>

            <div className="h-5 w-px bg-border" />

            <Logo href="/dashboard" size="sm" showText={false} />
          </div>

          <div className="flex items-center gap-3">
            {/* Save status */}
            <div className="hidden items-center gap-2 text-xs text-muted-foreground sm:flex">
              {isSaving ? (
                <>
                  <Spinner size="sm" />
                  <span>Saving...</span>
                </>
              ) : saveError ? (
                <span className="text-destructive">{saveError}</span>
              ) : hasChanges ? (
                <>
                  <Clock className="h-3.5 w-3.5" />
                  <span>Unsaved changes</span>
                </>
              ) : lastSaved ? (
                <>
                  <Check className="h-3.5 w-3.5 text-green-500" />
                  <span>Saved</span>
                </>
              ) : (
                <span>All changes saved</span>
              )}
            </div>

            <ModeToggle />

            <Button
              variant="ghost"
              size="icon-sm"
              title={draft.pinned ? "Unpin note" : "Pin note"}
              onClick={() => updateField("pinned", !draft.pinned)}
              className={cn(
                "text-muted-foreground",
                draft.pinned && "text-primary",
              )}
            >
              <Pin
                className={cn(
                  "h-4 w-4",
                  draft.pinned && "fill-current",
                )}
              />
            </Button>

            <Button
              size="sm"
              onClick={handleSave}
              disabled={isSaving || !hasChanges}
            >
              {isSaving ? (
                <Spinner size="sm" />
              ) : (
                <Save className="mr-2 h-4 w-4" />
              )}
              <span className="hidden sm:inline">Save</span>
            </Button>
          </div>
        </div>
      </header>

      {/* Workspace */}
      <main className="mx-auto max-w-[1600px] px-4 py-6 sm:px-6 lg:py-10">
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-[minmax(0,1fr)_280px] xl:grid-cols-[minmax(0,1fr)_320px]">
          {/* Main editor */}
          <div className="min-w-0">
            <div className="mx-auto max-w-4xl">
              {/* Note heading */}
              <div className="mb-8 space-y-4">
                <input
                  type="text"
                  value={draft.title}
                  onChange={(event) =>
                    updateField("title", event.target.value)
                  }
                  placeholder="Untitled note"
                  className="w-full border-none bg-transparent text-3xl font-bold tracking-tight outline-none placeholder:text-muted-foreground/30 sm:text-4xl"
                />

                <textarea
                  value={draft.description}
                  onChange={(event) =>
                    updateField("description", event.target.value)
                  }
                  placeholder="Add a short description..."
                  rows={2}
                  className="w-full resize-none border-none bg-transparent text-base text-muted-foreground outline-none placeholder:text-muted-foreground/40"
                />

                <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                  <span>Created {createdDate}</span>
                  <span>·</span>
                  <span>{draft.content.length.toLocaleString()} characters</span>
                  {draft.pinned && (
                    <>
                      <span>·</span>
                      <span className="flex items-center gap-1 text-primary">
                        <Pin className="h-3 w-3 fill-current" />
                        Pinned
                      </span>
                    </>
                  )}
                </div>
              </div>

              {/* Tiptap editor */}
              <DocumentEditor
                content={draft.content}
                onChange={(content) => updateField("content", content)}
                onSave={handleSave}
                placeholder="Start writing your note..."
                autoFocus
              />
            </div>
          </div>

          {/* Metadata sidebar */}
          <aside className="h-fit space-y-7 rounded-xl border border-border/70 bg-card/40 p-5 lg:sticky lg:top-24">

            <MetadataSection
              icon={BookOpen}
              title="Subject"
              description="Which subject does this note belong to?"
            >
              <input
                value={draft.subject}
                onChange={(event) =>
                  updateField("subject", event.target.value)
                }
                placeholder="e.g. Operating Systems"
                className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              />
            </MetadataSection>

            <div className="h-px bg-border" />

            <MetadataSection
              icon={Tag}
              title="Tags"
              description="Separate tags with commas."
            >
              <input
                value={draft.tags.join(", ")}
                onChange={(event) =>
                  updateField(
                    "tags",
                    event.target.value
                      .split(",")
                      .map((tag) => tag.trim())
                      .filter(Boolean),
                  )
                }
                placeholder="rust, backend, databases"
                className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm outline-none focus:border-primary"
              />

              {draft.tags.length > 0 && (
                <div className="flex flex-wrap gap-1.5">
                  {draft.tags.map((tag, index) => (
                    <span
                      key={`${tag}-${index}`}
                      className="rounded-md bg-secondary px-2 py-1 text-xs text-secondary-foreground"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              )}
            </MetadataSection>

            <div className="h-px bg-border" />

            <MetadataSection
              icon={Target}
              title="Learning objectives"
              description="What should you understand after studying this note? One objective per line."
            >
              <TextareaField
                value={draft.learning_objectives.join("\n")}
                onChange={(value) =>
                  updateField(
                    "learning_objectives",
                    value.split("\n").map((item) => item.trim()).filter(Boolean),
                  )
                }
                placeholder={"Understand ownership\nExplain borrowing rules"}
                rows={5}
              />
            </MetadataSection>

            <div className="h-px bg-border" />

            <MetadataSection
              icon={AlignLeft}
              title="Coverage notes"
              description="Track what this note covers. One item per line."
            >
              <TextareaField
                value={draft.coverage_notes.join("\n")}
                onChange={(value) =>
                  updateField(
                    "coverage_notes",
                    value.split("\n").map((item) => item.trim()).filter(Boolean),
                  )
                }
                placeholder={"Ownership\nBorrowing\nLifetimes"}
                rows={5}
              />
            </MetadataSection>

            <div className="h-px bg-border" />

            <MetadataSection icon={FileText} title="Information">
              <div className="space-y-2 text-xs">
                <div className="flex justify-between gap-3">
                  <span className="text-muted-foreground">Created</span>
                  <span>{createdDate}</span>
                </div>

                <div className="flex justify-between gap-3">
                  <span className="text-muted-foreground">Characters</span>
                  <span>{draft.content.length.toLocaleString()}</span>
                </div>
              </div>
            </MetadataSection>
          </aside>
        </div>
      </main>
    </div>
  );
}
