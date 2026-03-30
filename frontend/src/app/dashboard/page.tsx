"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Logo } from "@/components/ui/logo";
import { PageLoader } from "@/components/ui/spinner";
import { getNotes, createNote } from "@/lib/api";
import { isAuthenticated, clearTokens } from "@/lib/api-config";
import type { Model } from "@/lib/api/types.gen";
import {
  Plus,
  FileText,
  Pin,
  Archive,
  LogOut,
  Loader2,
  Search,
  Sparkles,
} from "lucide-react";

export default function DashboardPage() {
  const router = useRouter();
  const [notes, setNotes] = useState<Model[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [newNoteTitle, setNewNoteTitle] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    if (!isAuthenticated()) {
      router.push("/login");
      return;
    }

    loadNotes();
  }, [router]);

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

  const handleCreateNote = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newNoteTitle.trim()) return;

    setIsCreating(true);
    try {
      const response = await createNote({
        body: { title: newNoteTitle },
      });

      if (response.data?.data) {
        setNotes([response.data.data, ...notes]);
        setNewNoteTitle("");
      }
    } catch (error) {
      console.error("Failed to create note:", error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleLogout = () => {
    clearTokens();
    router.push("/login");
  };

  const filteredNotes = notes.filter((note) =>
    note.title.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const pinnedNotes = filteredNotes.filter((note) => note.pinned);
  const regularNotes = filteredNotes.filter((note) => !note.pinned && !note.archived);
  const archivedNotes = filteredNotes.filter((note) => note.archived);

  if (isLoading) {
    return <PageLoader text="Loading your notes..." />;
  }

  return (
    <div className="min-h-screen bg-background bg-gradient-hero">
      {/* Background decorations */}
      <div className="fixed inset-0 bg-grid-pattern opacity-10 pointer-events-none" />

      {/* Header */}
      <header className="sticky top-0 z-50 border-b border-border/50 bg-background/80 backdrop-blur-md">
        <div className="container mx-auto px-4 py-3 flex items-center justify-between">
          <Logo href="/dashboard" size="sm" />
          <div className="flex items-center gap-3">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleLogout}
              className="text-muted-foreground hover:text-foreground"
            >
              <LogOut className="w-4 h-4 mr-2" />
              Sign Out
            </Button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8 relative z-10">
        <div className="max-w-5xl mx-auto">
          {/* Welcome Section */}
          <div className="mb-8 animate-fade-in-up">
            <h1 className="text-3xl md:text-4xl font-bold mb-2">
              <span className="gradient-text">Your Notes</span>
            </h1>
            <p className="text-muted-foreground">
              {notes.length === 0
                ? "Create your first note to get started"
                : `${notes.length} note${notes.length !== 1 ? "s" : ""} total`}
            </p>
            <div className="mt-4">
              <Button asChild variant="outline" className="backdrop-blur-sm">
                <Link href="/interactive/yt">Focus Lecture (Live Questions)</Link>
              </Button>
            </div>
          </div>

          {/* Create Note & Search */}
          <div className="flex flex-col sm:flex-row gap-4 mb-8 animate-fade-in-up delay-100">
            <form onSubmit={handleCreateNote} className="flex gap-3 flex-1">
              <div className="relative flex-1">
                <Plus className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                <Input
                  value={newNoteTitle}
                  onChange={(e) => setNewNoteTitle(e.target.value)}
                  placeholder="Create a new note..."
                  className="pl-10 h-11 bg-card/50 border-border/50"
                  disabled={isCreating}
                />
              </div>
              <Button
                type="submit"
                disabled={isCreating || !newNoteTitle.trim()}
                className="h-11 px-6"
              >
                {isCreating ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <>
                    <Plus className="w-4 h-4 mr-2" />
                    Create
                  </>
                )}
              </Button>
            </form>

            {notes.length > 0 && (
              <div className="relative w-full sm:w-64">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                <Input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search notes..."
                  className="pl-10 h-11 bg-card/50 border-border/50"
                />
              </div>
            )}
          </div>

          {/* Notes Grid */}
          {notes.length === 0 ? (
            <Card className="border-border/50 bg-card/50 backdrop-blur-sm animate-fade-in-up delay-200">
              <CardContent className="py-16 text-center">
                <div className="w-20 h-20 rounded-2xl bg-primary/10 flex items-center justify-center mx-auto mb-6">
                  <Sparkles className="w-10 h-10 text-primary" />
                </div>
                <h3 className="text-xl font-semibold mb-3">No notes yet</h3>
                <p className="text-muted-foreground max-w-sm mx-auto mb-6">
                  Create your first note to start organizing your thoughts and ideas.
                </p>
                <Button
                  onClick={() => {
                    const input = document.querySelector<HTMLInputElement>(
                      'input[placeholder="Create a new note..."]'
                    );
                    input?.focus();
                  }}
                >
                  <Plus className="w-4 h-4 mr-2" />
                  Create Your First Note
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-8">
              {/* Pinned Notes */}
              {pinnedNotes.length > 0 && (
                <section className="animate-fade-in-up delay-200">
                  <h2 className="text-sm font-medium text-muted-foreground mb-4 flex items-center gap-2">
                    <Pin className="w-4 h-4" />
                    Pinned
                  </h2>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {pinnedNotes.map((note) => (
                      <NoteCard key={note.id} note={note} />
                    ))}
                  </div>
                </section>
              )}

              {/* Regular Notes */}
              {regularNotes.length > 0 && (
                <section className="animate-fade-in-up delay-300">
                  {pinnedNotes.length > 0 && (
                    <h2 className="text-sm font-medium text-muted-foreground mb-4 flex items-center gap-2">
                      <FileText className="w-4 h-4" />
                      Notes
                    </h2>
                  )}
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {regularNotes.map((note) => (
                      <NoteCard key={note.id} note={note} />
                    ))}
                  </div>
                </section>
              )}

              {/* Archived Notes */}
              {archivedNotes.length > 0 && (
                <section className="animate-fade-in-up delay-400">
                  <h2 className="text-sm font-medium text-muted-foreground mb-4 flex items-center gap-2">
                    <Archive className="w-4 h-4" />
                    Archived
                  </h2>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {archivedNotes.map((note) => (
                      <NoteCard key={note.id} note={note} />
                    ))}
                  </div>
                </section>
              )}

              {/* No search results */}
              {filteredNotes.length === 0 && searchQuery && (
                <Card className="border-border/50 bg-card/50">
                  <CardContent className="py-12 text-center">
                    <Search className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                    <h3 className="text-lg font-semibold mb-2">No notes found</h3>
                    <p className="text-muted-foreground">
                      No notes match &quot;{searchQuery}&quot;
                    </p>
                  </CardContent>
                </Card>
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

function NoteCard({ note }: { note: Model }) {
  return (
    <Link href={`/notes/${note.id}`}>
      <Card className="group card-hover border-border/50 bg-card/50 backdrop-blur-sm cursor-pointer h-full">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center justify-between">
            <span className="truncate text-base">{note.title || "Untitled"}</span>
            <div className="flex items-center gap-1 shrink-0">
              {note.pinned && (
                <Pin className="w-4 h-4 text-primary fill-primary" />
              )}
              {note.archived && (
                <Archive className="w-4 h-4 text-muted-foreground" />
              )}
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent className="pt-0">
          <p className="text-xs text-muted-foreground">
            Updated{" "}
            {new Date(note.updated_at).toLocaleDateString("en-US", {
              month: "short",
              day: "numeric",
              year: "numeric",
            })}
          </p>
        </CardContent>
      </Card>
    </Link>
  );
}
