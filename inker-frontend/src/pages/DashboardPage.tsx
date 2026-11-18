import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  Plus,
  FileText,
  Pin,
  Archive,
  LogOut,
  Loader2,
  Search,
  Sparkles,
  BookOpen,
  Target,
  Layers,
  Clock3,
  ArrowUpRight,
  CalendarDays,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Logo } from "@/components/ui/logo";
import { PageLoader } from "@/components/ui/spinner";
import { ModeToggle } from "@/components/mode-toggle";

import { getNotes, createNote } from "../lib/api";
import { isAuthenticated, clearTokens } from "../lib/api-config";
import type { Note } from "../lib/api/types.gen";

type Filter = "all" | "pinned" | "archived";

const formatDate = (date: string) =>
  new Date(date).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

const getNoteSearchText = (note: Note) =>
  [
    note.title,
    note.description,
    note.content,
    note.subject,
    ...(note.tags ?? []),
    ...(note.learning_objectives ?? []),
    ...(note.coverage_notes ?? []),
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();

export default function DashboardPage() {
  const navigate = useNavigate();

  const [notes, setNotes] = useState<Note[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [newNoteTitle, setNewNoteTitle] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeFilter, setActiveFilter] = useState<Filter>("all");

  useEffect(() => {
    if (!isAuthenticated()) {
      navigate("/login");
      return;
    }

    loadNotes();
  }, [navigate]);

  const loadNotes = async () => {
    try {
      const response = await getNotes();

      if (response.data?.data) {
        setNotes(response.data.data);
      }
    } catch (error) {
      console.error("Failed to load notes:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateNote = async (e: FormEvent) => {
    e.preventDefault();

    if (!newNoteTitle.trim()) return;

    setIsCreating(true);

    try {
      const response = await createNote({
        body: { title: newNoteTitle.trim(), content: "" },
      });

      if (response.data?.data) {
        setNotes((prev) => [response.data.data!, ...prev]);
        setNewNoteTitle("");
        navigate(`/notes/${response.data.data.id}`);
      }
    } catch (error) {
      console.error("Failed to create note:", error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleLogout = () => {
    clearTokens();
    navigate("/login");
  };

  const activeNotes = useMemo(
    () => notes.filter((note) => !note.deleted_at),
    [notes]
  );

  const pinnedCount = activeNotes.filter(
    (note) => note.pinned && !note.archived
  ).length;

  const archivedCount = activeNotes.filter(
    (note) => note.archived
  ).length;

  const filteredNotes = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();

    return activeNotes
      .filter((note) => {
        if (activeFilter === "pinned") {
          return note.pinned && !note.archived;
        }

        if (activeFilter === "archived") {
          return note.archived;
        }

        return !note.archived;
      })
      .filter((note) =>
        query ? getNoteSearchText(note).includes(query) : true
      )
      .sort(
        (a, b) =>
          new Date(b.updated_at).getTime() -
          new Date(a.updated_at).getTime()
      );
  }, [activeNotes, activeFilter, searchQuery]);

  if (isLoading) {
    return <PageLoader text="Loading your workspace..." />;
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-50 border-b bg-background/80 backdrop-blur-xl">
        <div className="container mx-auto flex items-center justify-between px-4 py-3">
          <Logo href="/dashboard" size="sm" />

          <div className="flex items-center gap-2">
            <ModeToggle />

            <Button
              variant="ghost"
              size="sm"
              onClick={handleLogout}
              className="text-muted-foreground"
            >
              <LogOut className="mr-2 h-4 w-4" />
              Sign Out
            </Button>
          </div>
        </div>
      </header>

      <main className="container relative mx-auto max-w-7xl px-4 py-8 md:py-12">
        {/* Welcome */}
        <section className="mb-9 flex flex-col justify-between gap-5 md:flex-row md:items-end">
          <div>
            <div className="mb-3 flex items-center gap-2 text-sm text-muted-foreground">
              <Sparkles className="h-4 w-4 text-primary" />
              Your personal knowledge workspace
            </div>

            <h1 className="text-3xl font-bold tracking-tight md:text-4xl">
              Your library
            </h1>

            <p className="mt-2 text-muted-foreground">
              Organize your thoughts, track what you learn, and revisit your
              knowledge.
            </p>
          </div>

          <Button asChild variant="outline">
            <Link to="/interactive/yt">
              <BookOpen className="mr-2 h-4 w-4" />
              Focus Lecture
              <ArrowUpRight className="ml-2 h-4 w-4" />
            </Link>
          </Button>
        </section>

        {/* Statistics */}
        <section className="mb-8 grid grid-cols-1 gap-3 sm:grid-cols-3">
          <StatCard
            icon={<FileText />}
            label="Total notes"
            value={activeNotes.length}
            description="Across your library"
          />

          <StatCard
            icon={<Pin />}
            label="Pinned"
            value={pinnedCount}
            description="Your important notes"
          />

          <StatCard
            icon={<Archive />}
            label="Archived"
            value={archivedCount}
            description="Stored for later"
          />
        </section>

        {/* Create note */}
        <Card className="mb-8 border-border/70 shadow-sm">
          <CardContent className="flex flex-col gap-3 p-4 sm:flex-row">
            <form
              onSubmit={handleCreateNote}
              className="flex flex-1 gap-2"
            >
              <div className="relative flex-1">
                <Plus className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />

                <Input
                  value={newNoteTitle}
                  onChange={(e) => setNewNoteTitle(e.target.value)}
                  placeholder="What would you like to learn about?"
                  className="h-11 pl-10"
                  disabled={isCreating}
                />
              </div>

              <Button
                type="submit"
                disabled={isCreating || !newNoteTitle.trim()}
                className="h-11"
              >
                {isCreating ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <>
                    <Plus className="mr-2 h-4 w-4" />
                    New note
                  </>
                )}
              </Button>
            </form>
          </CardContent>
        </Card>

        {/* Library controls */}
        <section>
          <div className="mb-5 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex flex-wrap items-center gap-1 rounded-lg border bg-card p-1">
              {(
                [
                  ["all", "All notes"],
                  ["pinned", "Pinned"],
                  ["archived", "Archived"],
                ] as const
              ).map(([filter, label]) => (
                <Button
                  key={filter}
                  size="sm"
                  variant={activeFilter === filter ? "secondary" : "ghost"}
                  onClick={() => setActiveFilter(filter)}
                >
                  {label}
                  <span className="ml-2 text-xs text-muted-foreground">
                    {filter === "all"
                      ? activeNotes.length - archivedCount
                      : filter === "pinned"
                        ? pinnedCount
                        : archivedCount}
                  </span>
                </Button>
              ))}
            </div>

            <div className="relative w-full sm:max-w-xs">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />

              <Input
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search your knowledge..."
                className="pl-10"
              />
            </div>
          </div>

          {/* Notes */}
          {filteredNotes.length > 0 ? (
            <>
              <div className="mb-4 flex items-center justify-between">
                <h2 className="font-semibold">
                  {activeFilter === "all"
                    ? "All notes"
                    : activeFilter === "pinned"
                      ? "Pinned notes"
                      : "Archived notes"}
                </h2>

                <span className="text-sm text-muted-foreground">
                  {filteredNotes.length} results
                </span>
              </div>

              <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
                {filteredNotes.map((note) => (
                  <NoteCard key={note.id} note={note} />
                ))}
              </div>
            </>
          ) : (
            <Card className="border-dashed shadow-none">
              <CardContent className="flex flex-col items-center py-16 text-center">
                <div className="mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-muted">
                  {searchQuery ? (
                    <Search className="h-6 w-6 text-muted-foreground" />
                  ) : (
                    <BookOpen className="h-6 w-6 text-muted-foreground" />
                  )}
                </div>

                <h3 className="text-lg font-semibold">
                  {searchQuery
                    ? "No matching notes"
                    : activeFilter === "pinned"
                      ? "Nothing pinned yet"
                      : activeFilter === "archived"
                        ? "No archived notes"
                        : "Your library is empty"}
                </h3>

                <p className="mt-2 max-w-sm text-sm text-muted-foreground">
                  {searchQuery
                    ? "Try searching with a different title, subject, tag, or topic."
                    : "Create a note and start building your personal knowledge library."}
                </p>

                {searchQuery && (
                  <Button
                    variant="outline"
                    className="mt-4"
                    onClick={() => setSearchQuery("")}
                  >
                    Clear search
                  </Button>
                )}
              </CardContent>
            </Card>
          )}
        </section>
      </main>
    </div>
  );
}

function StatCard({
  icon,
  label,
  value,
  description,
}: {
  icon: React.ReactNode;
  label: string;
  value: number;
  description: string;
}) {
  return (
    <Card className="border-border/70 shadow-sm">
      <CardContent className="flex items-center gap-4 p-5">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
          {icon}
        </div>

        <div className="min-w-0">
          <p className="text-sm text-muted-foreground">{label}</p>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold tabular-nums">{value}</span>
            <span className="truncate text-xs text-muted-foreground">
              {description}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function NoteCard({ note }: { note: Note }) {
  const objectives = note.learning_objectives ?? [];
  const coverage = note.coverage_notes ?? [];
  const tags = note.tags ?? [];

  return (
    <Link
      to={`/notes/${note.id}`}
      className="group block h-full outline-none"
    >
      <Card className="h-full border-border/70 transition-all duration-200 hover:-translate-y-1 hover:border-primary/40 hover:shadow-lg group-focus-visible:ring-2 group-focus-visible:ring-ring">
        <CardHeader className="space-y-3 pb-3">
          {/* Subject and status */}
          <div className="flex min-w-0 items-center justify-between gap-2">
            <div className="flex min-w-0 flex-wrap items-center gap-2">
              {note.subject && (
                <span className="max-w-full truncate rounded-md bg-primary/10 px-2 py-1 text-xs font-medium text-primary">
                  {note.subject}
                </span>
              )}

              {note.archived && (
                <span className="rounded-md bg-muted px-2 py-1 text-xs text-muted-foreground">
                  Archived
                </span>
              )}
            </div>

            {note.pinned && (
              <Pin className="h-4 w-4 shrink-0 fill-primary text-primary" />
            )}
          </div>

          {/* Title */}
          <CardTitle className="line-clamp-2 text-lg leading-snug transition-colors group-hover:text-primary">
            {note.title || "Untitled"}
          </CardTitle>

          {/* Description */}
          {note.description && (
            <p className="line-clamp-2 text-sm leading-relaxed text-muted-foreground">
              {note.description}
            </p>
          )}
        </CardHeader>

        <CardContent className="space-y-4 pt-0">
          {/* Content preview */}
          {note.content?.trim() && (
            <div className="rounded-lg bg-muted/40 p-3">
              <p className="line-clamp-3 whitespace-pre-wrap text-xs leading-relaxed text-muted-foreground">
                {note.content.replace(/[#*_`]/g, "").trim()}
              </p>
            </div>
          )}

          {/* Tags */}
          {tags.length > 0 && (
            <div className="flex flex-wrap gap-1.5">
              {tags.slice(0, 4).map((tag, index) => (
                <span
                  key={`${tag}-${index}`}
                  className="rounded-md border bg-background px-2 py-1 text-xs text-muted-foreground"
                >
                  #{tag}
                </span>
              ))}

              {tags.length > 4 && (
                <span className="px-1 py-1 text-xs text-muted-foreground">
                  +{tags.length - 4}
                </span>
              )}
            </div>
          )}

          {/* Learning objectives */}
          {objectives.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-xs font-semibold">
                <Target className="h-3.5 w-3.5 text-primary" />
                Learning objectives
                <span className="text-muted-foreground">
                  ({objectives.length})
                </span>
              </div>

              <ul className="space-y-1.5">
                {objectives.slice(0, 2).map((objective, index) => (
                  <li
                    key={index}
                    className="flex items-start gap-2 text-xs leading-relaxed text-muted-foreground"
                  >
                    <span className="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-primary/60" />
                    <span className="line-clamp-2">{objective}</span>
                  </li>
                ))}
              </ul>

              {objectives.length > 2 && (
                <p className="pl-3.5 text-xs text-muted-foreground">
                  +{objectives.length - 2} more objectives
                </p>
              )}
            </div>
          )}

          {/* Coverage notes */}
          {coverage.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-xs font-semibold">
                <Layers className="h-3.5 w-3.5 text-primary" />
                Topics covered
                <span className="text-muted-foreground">
                  ({coverage.length})
                </span>
              </div>

              <div className="flex flex-wrap gap-1.5">
                {coverage.slice(0, 3).map((item, index) => (
                  <span
                    key={index}
                    className="max-w-full truncate rounded-md bg-muted px-2 py-1 text-xs text-muted-foreground"
                  >
                    {item}
                  </span>
                ))}

                {coverage.length > 3 && (
                  <span className="px-1 py-1 text-xs text-muted-foreground">
                    +{coverage.length - 3}
                  </span>
                )}
              </div>
            </div>
          )}

          {/* Footer */}
          <div className="flex items-center justify-between gap-2 border-t pt-3">
            <div className="min-w-0 space-y-1">
              <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <Clock3 className="h-3 w-3" />
                Updated {formatDate(note.updated_at)}
              </div>

              <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <CalendarDays className="h-3 w-3" />
                Created {formatDate(note.created_at)}
              </div>

              <p className="text-[10px] text-muted-foreground">
                Version {note.version}
              </p>
            </div>

            <ArrowUpRight className="h-4 w-4 shrink-0 text-muted-foreground transition-all group-hover:-translate-y-1 group-hover:translate-x-1 group-hover:text-primary" />
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
