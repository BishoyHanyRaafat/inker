// Editor types and utilities for the block-based editor

import type {
  Block,
  TextBlock,
  TextSegment,
  TextMark,
  NoteBlock,
  ImageBlock,
  TodoBlock,
  DividerBlock,
  TableBlock,
} from "./api/types.gen";

// Re-export API types for convenience
export type {
  Block,
  TextBlock,
  TextSegment,
  TextMark,
  NoteBlock,
  ImageBlock,
  TodoBlock,
  DividerBlock,
  TableBlock,
};

// Local state for editing blocks
export interface EditorBlock {
  id: string;
  block: Block;
  order: number;
  version: number;
  isNew?: boolean; // Flag for unsaved blocks
}

// Helper to create a default text block
export function createEmptyTextBlock(order: number): EditorBlock {
  return {
    id: `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    block: {
      type: "text",
      data: {
        segments: [{ text: "", marks: [] }],
      },
    },
    order,
    version: 1,
    isNew: true,
  };
}

// Helper to create a divider block
export function createDividerBlock(order: number): EditorBlock {
  return {
    id: `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    block: {
      type: "divider",
      data: {} as never,
    },
    order,
    version: 1,
    isNew: true,
  };
}

// Helper to create a todo block
export function createTodoBlock(order: number): EditorBlock {
  return {
    id: `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    block: {
      type: "todo",
      data: {
        items: [""],
      },
    },
    order,
    version: 1,
    isNew: true,
  };
}

// Helper to create an image block
export function createImageBlock(order: number, url: string = ""): EditorBlock {
  return {
    id: `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    block: {
      type: "image",
      data: {
        url,
      },
    },
    order,
    version: 1,
    isNew: true,
  };
}

// Get plain text from a text block
export function getPlainText(block: Block): string {
  if (block.type === "text") {
    return block.data.segments.map((s) => s.text).join("");
  }
  if (block.type === "todo") {
    return block.data.items.join("\n");
  }
  return "";
}

// Check if a text block is empty
export function isBlockEmpty(block: Block): boolean {
  if (block.type === "text") {
    return block.data.segments.every((s) => s.text === "");
  }
  if (block.type === "todo") {
    return block.data.items.every((item) => item === "");
  }
  if (block.type === "divider") {
    return false; // Dividers are never "empty"
  }
  if (block.type === "image") {
    return !block.data.url;
  }
  return true;
}

// Parse slash commands
export type SlashCommand = "text" | "todo" | "divider" | "image" | "table";

export const SLASH_COMMANDS: { command: SlashCommand; label: string; description: string; icon: string }[] = [
  { command: "text", label: "Text", description: "Just start writing with plain text", icon: "Type" },
  { command: "todo", label: "To-do list", description: "Track tasks with a to-do list", icon: "CheckSquare" },
  { command: "divider", label: "Divider", description: "Visually divide blocks", icon: "Minus" },
  { command: "image", label: "Image", description: "Embed an image from URL", icon: "Image" },
];

// Check if text starts with slash command
export function matchSlashCommand(text: string): SlashCommand | null {
  const lower = text.toLowerCase().trim();
  if (lower === "/text" || lower === "/t") return "text";
  if (lower === "/todo" || lower === "/td") return "todo";
  if (lower === "/divider" || lower === "/div" || lower === "---") return "divider";
  if (lower === "/image" || lower === "/img") return "image";
  return null;
}
