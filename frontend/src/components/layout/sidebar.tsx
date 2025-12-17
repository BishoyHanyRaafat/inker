"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { cn } from "@/lib/utils";
import { Logo } from "@/components/ui/logo";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { getNotes, createNote } from "@/lib/api";
import { clearTokens } from "@/lib/api-config";
import type { Model } from "@/lib/api/types.gen";
import {
  FileText,
  Plus,
  Search,
  Pin,
  Archive,
  LogOut,
  ChevronLeft,
  ChevronRight,
  Loader2,
  Settings,
  Home,
} from "lucide-react";

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className }: SidebarProps) {
  const pathname = usePathname();
  const router = useRouter();
  const [notes, setNotes] = useState<Model[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    loadNotes();
  }, []);

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

  const handleCreateNote = async () => {
    setIsCreating(true);
    try {
      const response = await createNote({
        body: { title: "Untitled" },
      });
      if (response.data?.data) {
        setNotes([response.data.data, ...notes]);
        router.push(`/notes/${response.data.data.id}`);
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

  const pinnedNotes = filteredNotes.filter((note) => note.pinned && !note.archived);
  const regularNotes = filteredNotes.filter((note) => !note.pinned && !note.archived);

  return (
    <aside
      className={cn(
        "flex flex-col h-screen bg-sidebar border-r border-sidebar-border transition-all duration-300",
        isCollapsed ? "w-16" : "w-64",
        className
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-sidebar-border">
        {!isCollapsed && <Logo size="sm" href="/dashboard" />}
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="text-sidebar-foreground/70 hover:text-sidebar-foreground"
        >
          {isCollapsed ? (
            <ChevronRight className="w-4 h-4" />
          ) : (
            <ChevronLeft className="w-4 h-4" />
          )}
        </Button>
      </div>

      {/* Search & New Note */}
      {!isCollapsed && (
        <div className="p-3 space-y-2 border-b border-sidebar-border">
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search notes..."
              className="pl-8 h-9 bg-sidebar-accent/50 border-sidebar-border text-sm"
            />
          </div>
          <Button
            onClick={handleCreateNote}
            disabled={isCreating}
            className="w-full h-9 justify-start gap-2"
            variant="outline"
          >
            {isCreating ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Plus className="w-4 h-4" />
            )}
            {!isCreating && "New Note"}
          </Button>
        </div>
      )}

      {/* Collapsed new note button */}
      {isCollapsed && (
        <div className="p-2 border-b border-sidebar-border">
          <Button
            onClick={handleCreateNote}
            disabled={isCreating}
            variant="ghost"
            size="icon"
            className="w-full"
          >
            {isCreating ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Plus className="w-4 h-4" />
            )}
          </Button>
        </div>
      )}

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto py-2">
        {/* Home link */}
        <Link
          href="/dashboard"
          className={cn(
            "flex items-center gap-3 px-3 py-2 mx-2 rounded-md text-sm transition-colors",
            pathname === "/dashboard"
              ? "bg-sidebar-accent text-sidebar-accent-foreground"
              : "text-sidebar-foreground/70 hover:text-sidebar-foreground hover:bg-sidebar-accent/50"
          )}
        >
          <Home className="w-4 h-4 flex-shrink-0" />
          {!isCollapsed && <span>Home</span>}
        </Link>

        {!isCollapsed && (
          <>
            {/* Pinned Notes */}
            {pinnedNotes.length > 0 && (
              <div className="mt-4">
                <div className="px-3 py-1.5 text-xs font-medium text-sidebar-foreground/50 flex items-center gap-2">
                  <Pin className="w-3 h-3" />
                  Pinned
                </div>
                {pinnedNotes.map((note) => (
                  <NoteItem
                    key={note.id}
                    note={note}
                    isActive={pathname === `/notes/${note.id}`}
                  />
                ))}
              </div>
            )}

            {/* Regular Notes */}
            {regularNotes.length > 0 && (
              <div className="mt-4">
                <div className="px-3 py-1.5 text-xs font-medium text-sidebar-foreground/50 flex items-center gap-2">
                  <FileText className="w-3 h-3" />
                  Notes
                </div>
                {regularNotes.map((note) => (
                  <NoteItem
                    key={note.id}
                    note={note}
                    isActive={pathname === `/notes/${note.id}`}
                  />
                ))}
              </div>
            )}

            {/* Empty state */}
            {filteredNotes.length === 0 && !isLoading && (
              <div className="px-3 py-8 text-center text-sm text-muted-foreground">
                {searchQuery ? "No notes found" : "No notes yet"}
              </div>
            )}

            {/* Loading state */}
            {isLoading && (
              <div className="px-3 py-8 flex justify-center">
                <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
              </div>
            )}
          </>
        )}
      </nav>

      {/* Footer */}
      <div className="p-2 border-t border-sidebar-border">
        <Button
          variant="ghost"
          onClick={handleLogout}
          className={cn(
            "w-full text-sidebar-foreground/70 hover:text-sidebar-foreground",
            isCollapsed ? "justify-center" : "justify-start gap-2"
          )}
          size={isCollapsed ? "icon" : "sm"}
        >
          <LogOut className="w-4 h-4" />
          {!isCollapsed && "Sign Out"}
        </Button>
      </div>
    </aside>
  );
}

function NoteItem({ note, isActive }: { note: Model; isActive: boolean }) {
  return (
    <Link
      href={`/notes/${note.id}`}
      className={cn(
        "flex items-center gap-2 px-3 py-2 mx-2 rounded-md text-sm transition-colors",
        isActive
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground/70 hover:text-sidebar-foreground hover:bg-sidebar-accent/50"
      )}
    >
      <FileText className="w-4 h-4 flex-shrink-0" />
      <span className="truncate flex-1">{note.title || "Untitled"}</span>
      {note.pinned && <Pin className="w-3 h-3 flex-shrink-0 fill-current" />}
    </Link>
  );
}
