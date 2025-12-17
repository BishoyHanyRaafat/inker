"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import type { EditorBlock, SlashCommand } from "@/lib/editor-types";
import {
  createEmptyTextBlock,
  createDividerBlock,
  createTodoBlock,
  createImageBlock,
  matchSlashCommand,
  getPlainText,
} from "@/lib/editor-types";
import { BlockRenderer } from "./block-renderer";
import { SlashCommandMenu } from "./slash-command-menu";
import { cn } from "@/lib/utils";

interface BlockEditorProps {
  blocks: EditorBlock[];
  onChange: (blocks: EditorBlock[]) => void;
  onSave?: () => void;
  className?: string;
}

export function BlockEditor({
  blocks,
  onChange,
  onSave,
  className,
}: BlockEditorProps) {
  const [activeBlockId, setActiveBlockId] = useState<string | null>(null);
  const [slashMenuOpen, setSlashMenuOpen] = useState(false);
  const [slashFilter, setSlashFilter] = useState("");
  const [focusBlockId, setFocusBlockId] = useState<string | null>(null);
  const editorRef = useRef<HTMLDivElement>(null);

  // Ensure there's always at least one block
  useEffect(() => {
    if (blocks.length === 0) {
      onChange([createEmptyTextBlock(0)]);
    }
  }, [blocks, onChange]);

  // Sort blocks by order
  const sortedBlocks = [...blocks].sort((a, b) => a.order - b.order);

  const updateBlock = useCallback(
    (updatedBlock: EditorBlock) => {
      const newBlocks = blocks.map((b) =>
        b.id === updatedBlock.id ? updatedBlock : b
      );
      onChange(newBlocks);

      // Check for slash commands in text blocks
      if (updatedBlock.block.type === "text") {
        const text = getPlainText(updatedBlock.block);
        if (text.startsWith("/")) {
          setSlashMenuOpen(true);
          setSlashFilter(text.slice(1));
        } else {
          setSlashMenuOpen(false);
          setSlashFilter("");
        }
      }
    },
    [blocks, onChange]
  );

  const deleteBlock = useCallback(
    (blockId: string) => {
      const blockIndex = sortedBlocks.findIndex((b) => b.id === blockId);
      const newBlocks = blocks.filter((b) => b.id !== blockId);

      // If we deleted all blocks, create a new empty one
      if (newBlocks.length === 0) {
        onChange([createEmptyTextBlock(0)]);
        return;
      }

      onChange(newBlocks);

      // Focus previous block
      if (blockIndex > 0) {
        setFocusBlockId(sortedBlocks[blockIndex - 1].id);
      } else if (sortedBlocks.length > 1) {
        setFocusBlockId(sortedBlocks[1].id);
      }
    },
    [blocks, sortedBlocks, onChange]
  );

  const addBlockBelow = useCallback(
    (afterBlockId: string) => {
      const afterBlock = blocks.find((b) => b.id === afterBlockId);
      if (!afterBlock) return;

      const newOrder = afterBlock.order + 0.5;
      const newBlock = createEmptyTextBlock(newOrder);

      // Rebalance orders
      const newBlocks = [...blocks, newBlock]
        .sort((a, b) => a.order - b.order)
        .map((b, i) => ({ ...b, order: i }));

      onChange(newBlocks);
      setFocusBlockId(newBlock.id);
    },
    [blocks, onChange]
  );

  const handleSlashCommand = useCallback(
    (command: SlashCommand) => {
      if (!activeBlockId) return;

      const activeBlock = blocks.find((b) => b.id === activeBlockId);
      if (!activeBlock) return;

      let newBlock: EditorBlock;
      switch (command) {
        case "text":
          newBlock = createEmptyTextBlock(activeBlock.order);
          break;
        case "todo":
          newBlock = createTodoBlock(activeBlock.order);
          break;
        case "divider":
          newBlock = createDividerBlock(activeBlock.order);
          break;
        case "image":
          newBlock = createImageBlock(activeBlock.order);
          break;
        default:
          newBlock = createEmptyTextBlock(activeBlock.order);
      }

      // Replace the active block with the new block type
      newBlock.id = activeBlock.id;
      newBlock.isNew = activeBlock.isNew;

      const newBlocks = blocks.map((b) =>
        b.id === activeBlockId ? newBlock : b
      );

      onChange(newBlocks);
      setSlashMenuOpen(false);
      setSlashFilter("");
      setFocusBlockId(newBlock.id);
    },
    [activeBlockId, blocks, onChange]
  );

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + S to save
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        onSave?.();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onSave]);

  // Click on empty area to add block at end
  const handleEditorClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === editorRef.current) {
        // Clicked on empty area, focus last block or create one
        if (sortedBlocks.length > 0) {
          const lastBlock = sortedBlocks[sortedBlocks.length - 1];
          setFocusBlockId(lastBlock.id);
        }
      }
    },
    [sortedBlocks]
  );

  return (
    <div
      ref={editorRef}
      className={cn("relative min-h-[300px] py-4", className)}
      onClick={handleEditorClick}
    >
      <div className="space-y-1">
        {sortedBlocks.map((block) => (
          <div key={block.id} className="relative">
            <BlockRenderer
              block={block}
              onUpdate={updateBlock}
              onDelete={() => deleteBlock(block.id)}
              onAddBelow={() => addBlockBelow(block.id)}
              onFocus={() => setActiveBlockId(block.id)}
              isActive={block.id === activeBlockId}
              autoFocus={block.id === focusBlockId}
            />
            {/* Slash command menu */}
            {block.id === activeBlockId && slashMenuOpen && (
              <SlashCommandMenu
                isOpen={slashMenuOpen}
                onClose={() => setSlashMenuOpen(false)}
                onSelect={handleSlashCommand}
                filter={slashFilter}
              />
            )}
          </div>
        ))}
      </div>

      {/* Add block button at the end */}
      {sortedBlocks.length > 0 && (
        <button
          type="button"
          onClick={() => addBlockBelow(sortedBlocks[sortedBlocks.length - 1].id)}
          className="mt-4 w-full py-3 text-muted-foreground hover:text-foreground hover:bg-accent/30 rounded-lg transition-colors duration-150 text-sm"
        >
          Click or press Enter to add a block
        </button>
      )}
    </div>
  );
}
