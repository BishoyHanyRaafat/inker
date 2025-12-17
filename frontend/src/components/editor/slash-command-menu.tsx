"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { cn } from "@/lib/utils";
import { SLASH_COMMANDS, type SlashCommand } from "@/lib/editor-types";
import {
  Type,
  CheckSquare,
  Minus,
  Image,
  Table,
} from "lucide-react";

const ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  Type,
  CheckSquare,
  Minus,
  Image,
  Table,
};

interface SlashCommandMenuProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (command: SlashCommand) => void;
  filter?: string;
  position?: { top: number; left: number };
}

export function SlashCommandMenu({
  isOpen,
  onClose,
  onSelect,
  filter = "",
  position,
}: SlashCommandMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);

  const filteredCommands = SLASH_COMMANDS.filter(
    (cmd) =>
      cmd.label.toLowerCase().includes(filter.toLowerCase()) ||
      cmd.command.toLowerCase().includes(filter.toLowerCase())
  );

  // Reset selection when filter changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [filter]);

  // Handle keyboard navigation
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) => (i + 1) % filteredCommands.length);
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) =>
            i === 0 ? filteredCommands.length - 1 : i - 1
          );
          break;
        case "Enter":
          e.preventDefault();
          if (filteredCommands[selectedIndex]) {
            onSelect(filteredCommands[selectedIndex].command);
          }
          break;
        case "Escape":
          e.preventDefault();
          onClose();
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, selectedIndex, filteredCommands, onSelect, onClose]);

  // Close on click outside
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen, onClose]);

  if (!isOpen || filteredCommands.length === 0) return null;

  return (
    <div
      ref={menuRef}
      className={cn(
        "absolute z-50 w-72 rounded-lg border border-border bg-popover shadow-lg",
        "animate-in fade-in-0 zoom-in-95 duration-150"
      )}
      style={
        position
          ? { top: position.top, left: position.left }
          : { top: "100%", left: 0 }
      }
    >
      <div className="p-1.5">
        <div className="px-2 py-1.5 text-xs font-medium text-muted-foreground">
          Basic blocks
        </div>
        {filteredCommands.map((cmd, index) => {
          const Icon = ICONS[cmd.icon] || Type;
          return (
            <button
              key={cmd.command}
              type="button"
              onClick={() => onSelect(cmd.command)}
              onMouseEnter={() => setSelectedIndex(index)}
              className={cn(
                "w-full flex items-center gap-3 px-2 py-2 rounded-md text-left",
                "transition-colors duration-100",
                index === selectedIndex
                  ? "bg-accent text-accent-foreground"
                  : "hover:bg-accent/50"
              )}
            >
              <div className="w-10 h-10 rounded-md bg-muted/50 flex items-center justify-center flex-shrink-0">
                <Icon className="w-5 h-5" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="font-medium text-sm">{cmd.label}</div>
                <div className="text-xs text-muted-foreground truncate">
                  {cmd.description}
                </div>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
