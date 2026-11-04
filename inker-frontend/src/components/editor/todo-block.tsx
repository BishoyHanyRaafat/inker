import { useRef, useCallback } from "react";
import type { TodoBlock as TodoBlockType, TodoItem } from "../../lib/api/types.gen";
import { cn } from "../../lib/utils";
import { Square, CheckSquare, Plus, Trash2 } from "lucide-react";

interface TodoBlockProps {
  data: TodoBlockType;
  onChange: (data: TodoBlockType) => void;
  onKeyDown?: (e: React.KeyboardEvent) => void;
  onFocus?: () => void;
}

export function TodoBlock({ data, onChange, onKeyDown, onFocus }: TodoBlockProps) {
  const itemRefs = useRef<(HTMLInputElement | null)[]>([]);
  const items = data.items;

  const updateItem = useCallback(
    (index: number, updates: Partial<TodoItem>) => {
      const newItems = items.map((item, i) =>
        i === index ? { ...item, ...updates } : item
      );

      onChange({ items: newItems });
    },
    [items, onChange]
  );

  const addItem = useCallback(
    (afterIndex: number) => {
      const newItems = [...items];
      newItems.splice(afterIndex + 1, 0, {
        text: "",
        completed: false,
      });

      onChange({ items: newItems });

      requestAnimationFrame(() => {
        itemRefs.current[afterIndex + 1]?.focus();
      });
    },
    [items, onChange]
  );

  const removeItem = useCallback(
    (index: number) => {
      if (items.length <= 1) {
        updateItem(index, { text: "", completed: false });
        return;
      }

      const newItems = items.filter((_, i) => i !== index);
      onChange({ items: newItems });

      requestAnimationFrame(() => {
        itemRefs.current[Math.max(0, index - 1)]?.focus();
      });
    },
    [items, onChange, updateItem]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, index: number) => {
      if (e.key === "Enter") {
        e.preventDefault();
        addItem(index);
      } else if (e.key === "Backspace" && !items[index].text) {
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
            onClick={() => updateItem(index, { completed: !item.completed })}
            className={cn(
              "shrink-0 transition-colors duration-150",
              item.completed
                ? "text-primary"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            {item.completed ? (
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
              item.completed && "line-through text-muted-foreground"
            )}
          />

          <button
            type="button"
            onClick={() => removeItem(index)}
            className="shrink-0 opacity-0 group-hover/item:opacity-100 text-muted-foreground hover:text-destructive transition-all duration-150"
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
