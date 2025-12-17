"use client";

import { useRef, useCallback } from "react";
import type { TodoBlock as TodoBlockType } from "@/lib/api/types.gen";
import { cn } from "@/lib/utils";
import { Square, CheckSquare, Plus, Trash2 } from "lucide-react";

interface TodoBlockProps {
  data: TodoBlockType;
  onChange: (data: TodoBlockType) => void;
  onKeyDown?: (e: React.KeyboardEvent) => void;
  onFocus?: () => void;
}

interface TodoItemState {
  text: string;
  checked: boolean;
}

// Parse todo item - format: "[x] text" or "[ ] text" or just "text"
function parseTodoItem(item: string): TodoItemState {
  if (item.startsWith("[x] ") || item.startsWith("[X] ")) {
    return { text: item.slice(4), checked: true };
  }
  if (item.startsWith("[ ] ")) {
    return { text: item.slice(4), checked: false };
  }
  return { text: item, checked: false };
}

// Serialize todo item back to string
function serializeTodoItem(state: TodoItemState): string {
  return `[${state.checked ? "x" : " "}] ${state.text}`;
}

export function TodoBlock({ data, onChange, onKeyDown, onFocus }: TodoBlockProps) {
  const itemRefs = useRef<(HTMLInputElement | null)[]>([]);

  const items = data.items.map(parseTodoItem);

  const updateItem = useCallback(
    (index: number, updates: Partial<TodoItemState>) => {
      const newItems = [...items];
      newItems[index] = { ...newItems[index], ...updates };
      onChange({ items: newItems.map(serializeTodoItem) });
    },
    [items, onChange]
  );

  const addItem = useCallback(
    (afterIndex: number) => {
      const newItems = [...items];
      newItems.splice(afterIndex + 1, 0, { text: "", checked: false });
      onChange({ items: newItems.map(serializeTodoItem) });
      // Focus the new item after render
      setTimeout(() => {
        itemRefs.current[afterIndex + 1]?.focus();
      }, 0);
    },
    [items, onChange]
  );

  const removeItem = useCallback(
    (index: number) => {
      if (items.length <= 1) {
        // Don't remove the last item, just clear it
        updateItem(index, { text: "", checked: false });
        return;
      }
      const newItems = items.filter((_, i) => i !== index);
      onChange({ items: newItems.map(serializeTodoItem) });
      // Focus previous item
      setTimeout(() => {
        const focusIndex = Math.max(0, index - 1);
        itemRefs.current[focusIndex]?.focus();
      }, 0);
    },
    [items, onChange, updateItem]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, index: number) => {
      if (e.key === "Enter") {
        e.preventDefault();
        addItem(index);
      } else if (e.key === "Backspace" && items[index].text === "") {
        e.preventDefault();
        removeItem(index);
      } else if (e.key === "ArrowUp" && index > 0) {
        e.preventDefault();
        itemRefs.current[index - 1]?.focus();
      } else if (e.key === "ArrowDown" && index < items.length - 1) {
        e.preventDefault();
        itemRefs.current[index + 1]?.focus();
      }
      onKeyDown?.(e);
    },
    [items, addItem, removeItem, onKeyDown]
  );

  return (
    <div className="space-y-1 py-1">
      {items.map((item, index) => (
        <div
          key={index}
          className="flex items-center gap-2 group/item hover:bg-accent/30 rounded px-1 -mx-1"
        >
          <button
            type="button"
            onClick={() => updateItem(index, { checked: !item.checked })}
            className={cn(
              "flex-shrink-0 transition-colors duration-150",
              item.checked ? "text-primary" : "text-muted-foreground hover:text-foreground"
            )}
          >
            {item.checked ? (
              <CheckSquare className="w-4 h-4" />
            ) : (
              <Square className="w-4 h-4" />
            )}
          </button>
          <input
            ref={(el) => {
              itemRefs.current[index] = el;
            }}
            type="text"
            value={item.text}
            onChange={(e) => updateItem(index, { text: e.target.value })}
            onKeyDown={(e) => handleKeyDown(e, index)}
            onFocus={onFocus}
            placeholder="To-do"
            className={cn(
              "flex-1 bg-transparent outline-none py-1 text-foreground placeholder:text-muted-foreground/50",
              item.checked && "line-through text-muted-foreground"
            )}
          />
          <button
            type="button"
            onClick={() => removeItem(index)}
            className="flex-shrink-0 opacity-0 group-hover/item:opacity-100 text-muted-foreground hover:text-destructive transition-all duration-150"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      ))}
      <button
        type="button"
        onClick={() => addItem(items.length - 1)}
        className="flex items-center gap-2 text-muted-foreground hover:text-foreground text-sm mt-1 py-1 px-1 -mx-1 rounded hover:bg-accent/30 transition-colors duration-150"
      >
        <Plus className="w-3.5 h-3.5" />
        Add item
      </button>
    </div>
  );
}
