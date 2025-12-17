"use client";

import { useCallback } from "react";
import type { EditorBlock, Block } from "@/lib/editor-types";
import { TextBlock } from "./text-block";
import { TodoBlock } from "./todo-block";
import { DividerBlock } from "./divider-block";
import { ImageBlock } from "./image-block";
import { cn } from "@/lib/utils";
import { GripVertical, Plus, Trash2 } from "lucide-react";

interface BlockRendererProps {
  block: EditorBlock;
  onUpdate: (block: EditorBlock) => void;
  onDelete: () => void;
  onAddBelow: () => void;
  onKeyDown?: (e: React.KeyboardEvent) => void;
  onFocus?: () => void;
  isActive?: boolean;
  autoFocus?: boolean;
}

export function BlockRenderer({
  block,
  onUpdate,
  onDelete,
  onAddBelow,
  onKeyDown,
  onFocus,
  isActive = false,
  autoFocus = false,
}: BlockRendererProps) {
  const handleBlockChange = useCallback(
    (data: Block["data"]) => {
      onUpdate({
        ...block,
        block: { ...block.block, data } as Block,
      });
    },
    [block, onUpdate]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Handle Enter to create new block (for text blocks only)
      if (e.key === "Enter" && !e.shiftKey && block.block.type === "text") {
        e.preventDefault();
        onAddBelow();
      }
      // Handle backspace on empty block to delete
      if (e.key === "Backspace") {
        const target = e.target as HTMLElement;
        const text = target.innerText || "";
        if (text === "" && block.block.type === "text") {
          e.preventDefault();
          onDelete();
        }
      }
      onKeyDown?.(e);
    },
    [block.block.type, onAddBelow, onDelete, onKeyDown]
  );

  const renderBlock = () => {
    switch (block.block.type) {
      case "text":
        return (
          <TextBlock
            data={block.block.data}
            onChange={handleBlockChange}
            onKeyDown={handleKeyDown}
            onFocus={onFocus}
            autoFocus={autoFocus}
          />
        );
      case "todo":
        return (
          <TodoBlock
            data={block.block.data}
            onChange={handleBlockChange}
            onKeyDown={onKeyDown}
            onFocus={onFocus}
          />
        );
      case "divider":
        return <DividerBlock />;
      case "image":
        return (
          <ImageBlock
            data={block.block.data}
            onChange={handleBlockChange}
            onFocus={onFocus}
          />
        );
      default:
        return (
          <div className="text-muted-foreground text-sm italic">
            Unsupported block type
          </div>
        );
    }
  };

  return (
    <div
      className={cn(
        "group relative flex items-start gap-1 -ml-16 pl-16",
        "transition-colors duration-150",
        isActive && "bg-accent/20 rounded-lg"
      )}
    >
      {/* Block controls */}
      <div className="absolute left-2 top-1 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
        <button
          type="button"
          onClick={onAddBelow}
          className="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
          title="Add block below"
        >
          <Plus className="w-4 h-4" />
        </button>
        <button
          type="button"
          className="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-grab active:cursor-grabbing"
          title="Drag to reorder"
        >
          <GripVertical className="w-4 h-4" />
        </button>
        <button
          type="button"
          onClick={onDelete}
          className="p-1 rounded hover:bg-destructive/20 text-muted-foreground hover:text-destructive transition-colors"
          title="Delete block"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>

      {/* Block content */}
      <div className="flex-1 min-w-0">{renderBlock()}</div>
    </div>
  );
}
