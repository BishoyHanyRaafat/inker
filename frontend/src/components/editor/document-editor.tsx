"use client";

import { useState, useRef, useCallback, useEffect } from "react";
import { cn } from "@/lib/utils";
import {
  Bold,
  Italic,
  Underline,
  Strikethrough,
  Code,
  Highlighter,
  List,
  ListOrdered,
  Link,
  Link2Off,
  Undo,
  Redo,
  AlignLeft,
  AlignCenter,
  AlignRight,
  Image,
  Table,
  CheckSquare,
  Minus,
  Plus,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Modal } from "@/components/ui/modal";

interface DocumentEditorProps {
  content: string;
  onChange: (content: string) => void;
  onSave?: () => void;
  placeholder?: string;
  className?: string;
  autoFocus?: boolean;
}

interface ToolbarButtonProps {
  icon: React.ReactNode;
  onClick: (e?: React.MouseEvent) => void;
  active?: boolean;
  title: string;
  disabled?: boolean;
}

function ToolbarButton({ icon, onClick, active, title, disabled }: ToolbarButtonProps) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-sm"
      onClick={onClick}
      title={title}
      disabled={disabled}
      className={cn(
        "h-8 w-8 text-muted-foreground hover:text-foreground",
        active && "bg-accent text-foreground"
      )}
    >
      {icon}
    </Button>
  );
}

function ToolbarSeparator() {
  return <div className="w-px h-6 bg-border mx-1" />;
}

export function DocumentEditor({
  content,
  onChange,
  onSave,
  placeholder = "Start writing...",
  className,
  autoFocus = false,
}: DocumentEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const [activeFormats, setActiveFormats] = useState<Set<string>>(new Set());
  const [showInsertMenu, setShowInsertMenu] = useState(false);
  
  // Modal states
  const [showLinkModal, setShowLinkModal] = useState(false);
  const [showImageModal, setShowImageModal] = useState(false);
  const [showTableModal, setShowTableModal] = useState(false);
  
  // Form states
  const [linkUrl, setLinkUrl] = useState("https://");
  const [linkText, setLinkText] = useState("");
  const [imageUrl, setImageUrl] = useState("");
  const [tableRows, setTableRows] = useState("3");
  const [tableCols, setTableCols] = useState("3");
  
  // Store selection for link insertion
  const [savedSelection, setSavedSelection] = useState<Range | null>(null);

  // Execute command helper
  const execCommand = useCallback((command: string, value?: string) => {
    document.execCommand(command, false, value);
    editorRef.current?.focus();
    updateActiveFormats();
  }, []);

  // Check active formats when selection changes
  const updateActiveFormats = useCallback(() => {
    const formats = new Set<string>();
    if (document.queryCommandState("bold")) formats.add("bold");
    if (document.queryCommandState("italic")) formats.add("italic");
    if (document.queryCommandState("underline")) formats.add("underline");
    if (document.queryCommandState("strikeThrough")) formats.add("strikeThrough");
    if (document.queryCommandState("insertUnorderedList")) formats.add("ul");
    if (document.queryCommandState("insertOrderedList")) formats.add("ol");
    setActiveFormats(formats);
  }, []);

  // Handle content changes
  const handleInput = useCallback(() => {
    if (editorRef.current) {
      onChange(editorRef.current.innerHTML);
    }
  }, [onChange]);

  // Save current selection
  const saveSelection = useCallback(() => {
    const selection = window.getSelection();
    if (selection && selection.rangeCount > 0) {
      setSavedSelection(selection.getRangeAt(0).cloneRange());
    }
  }, []);

  // Restore saved selection
  const restoreSelection = useCallback(() => {
    if (savedSelection) {
      const selection = window.getSelection();
      selection?.removeAllRanges();
      selection?.addRange(savedSelection);
    }
  }, [savedSelection]);

  // Handle keyboard shortcuts
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Save shortcut
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        onSave?.();
        return;
      }

      // Format shortcuts
      if (e.metaKey || e.ctrlKey) {
        switch (e.key) {
          case "b":
            e.preventDefault();
            execCommand("bold");
            break;
          case "i":
            e.preventDefault();
            execCommand("italic");
            break;
          case "u":
            e.preventDefault();
            execCommand("underline");
            break;
          case "k":
            e.preventDefault();
            openLinkModal();
            break;
        }
      }

      updateActiveFormats();
    },
    [onSave, execCommand, updateActiveFormats]
  );

  // Open link modal
  const openLinkModal = useCallback(() => {
    saveSelection();
    const selection = window.getSelection();
    const selectedText = selection?.toString() || "";
    setLinkText(selectedText);
    setLinkUrl("https://");
    setShowLinkModal(true);
  }, [saveSelection]);

  // Insert link
  const insertLink = useCallback(() => {
    if (!linkUrl || linkUrl === "https://") return;
    
    restoreSelection();
    editorRef.current?.focus();
    
    setTimeout(() => {
      if (linkText && !savedSelection?.toString()) {
        document.execCommand("insertHTML", false, 
          `<a href="${linkUrl}" target="_blank" rel="noopener noreferrer">${linkText}</a>`
        );
      } else {
        document.execCommand("createLink", false, linkUrl);
        // Make links open in new tab
        const links = editorRef.current?.querySelectorAll(`a[href="${linkUrl}"]`);
        links?.forEach(link => {
          link.setAttribute("target", "_blank");
          link.setAttribute("rel", "noopener noreferrer");
        });
      }
      handleInput();
    }, 10);
    
    setShowLinkModal(false);
    setLinkUrl("https://");
    setLinkText("");
  }, [linkUrl, linkText, restoreSelection, savedSelection, handleInput]);

  // Remove link
  const removeLink = useCallback(() => {
    execCommand("unlink");
  }, [execCommand]);

  // Open image modal
  const openImageModal = useCallback(() => {
    setImageUrl("");
    setShowImageModal(true);
  }, []);

  // Insert image
  const insertImage = useCallback(() => {
    if (!imageUrl) return;
    
    editorRef.current?.focus();
    execCommand("insertHTML", 
      `<img src="${imageUrl}" alt="" style="max-width: 100%; height: auto; border-radius: 8px; margin: 16px 0;" />`
    );
    
    setShowImageModal(false);
    setImageUrl("");
  }, [imageUrl, execCommand]);

  // Insert horizontal rule / divider
  const insertDivider = useCallback(() => {
    execCommand("insertHTML", '<hr style="border: none; border-top: 1px solid var(--border); margin: 24px 0;" />');
  }, [execCommand]);

  // Insert todo/checklist
  const insertTodo = useCallback(() => {
    execCommand("insertHTML", `
      <label class="todo-item" style="display: flex; align-items: flex-start; gap: 10px; margin: 8px 0; cursor: pointer; user-select: none;">
        <input type="checkbox" class="todo-checkbox" style="margin-top: 5px; cursor: pointer; width: 18px; height: 18px; accent-color: var(--primary);" />
        <span class="todo-text" style="flex: 1; padding: 2px 0;">New task</span>
      </label>
    `);
  }, [execCommand]);

  // Handle checkbox changes for todo items
  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) return;

    const handleCheckboxChange = (e: Event) => {
      const target = e.target as HTMLInputElement;
      if (target.type === "checkbox" && target.classList.contains("todo-checkbox")) {
        const label = target.closest(".todo-item");
        const textSpan = label?.querySelector(".todo-text") as HTMLElement;
        if (textSpan) {
          if (target.checked) {
            textSpan.style.textDecoration = "line-through";
            textSpan.style.color = "var(--muted-foreground)";
          } else {
            textSpan.style.textDecoration = "none";
            textSpan.style.color = "inherit";
          }
        }
        // Trigger content change
        handleInput();
      }
    };

    editor.addEventListener("change", handleCheckboxChange);
    return () => editor.removeEventListener("change", handleCheckboxChange);
  }, [handleInput]);

  // Open table modal
  const openTableModal = useCallback(() => {
    setTableRows("3");
    setTableCols("3");
    setShowTableModal(true);
  }, []);

  // Insert table
  const insertTable = useCallback(() => {
    const numRows = parseInt(tableRows) || 3;
    const numCols = parseInt(tableCols) || 3;
    
    let tableHtml = '<table style="width: 100%; border-collapse: collapse; margin: 16px 0;">';
    for (let i = 0; i < numRows; i++) {
      tableHtml += '<tr>';
      for (let j = 0; j < numCols; j++) {
        const cellStyle = 'border: 1px solid var(--border); padding: 8px; min-width: 80px;';
        if (i === 0) {
          tableHtml += `<th style="${cellStyle} background: var(--muted); font-weight: 600;">Header</th>`;
        } else {
          tableHtml += `<td style="${cellStyle}">Cell</td>`;
        }
      }
      tableHtml += '</tr>';
    }
    tableHtml += '</table>';
    
    editorRef.current?.focus();
    execCommand("insertHTML", tableHtml);
    setShowTableModal(false);
  }, [tableRows, tableCols, execCommand]);

  // Insert code block
  const insertCodeBlock = useCallback(() => {
    execCommand("insertHTML", `
      <pre style="background: var(--muted); padding: 16px; border-radius: 8px; overflow-x: auto; margin: 16px 0; font-family: monospace;"><code>// Your code here</code></pre>
    `);
  }, [execCommand]);

  // Focus editor on mount if autoFocus
  useEffect(() => {
    if (autoFocus && editorRef.current) {
      editorRef.current.focus();
      // Move cursor to end
      const range = document.createRange();
      const selection = window.getSelection();
      range.selectNodeContents(editorRef.current);
      range.collapse(false);
      selection?.removeAllRanges();
      selection?.addRange(range);
    }
  }, [autoFocus]);

  // Set initial content
  useEffect(() => {
    if (editorRef.current && content && editorRef.current.innerHTML !== content) {
      editorRef.current.innerHTML = content;
    }
  }, [content]);

  // Track selection changes
  useEffect(() => {
    const handleSelectionChange = () => {
      updateActiveFormats();
    };
    document.addEventListener("selectionchange", handleSelectionChange);
    return () => document.removeEventListener("selectionchange", handleSelectionChange);
  }, [updateActiveFormats]);

  // Close insert menu when clicking outside
  useEffect(() => {
    if (!showInsertMenu) return;
    const handleClick = () => setShowInsertMenu(false);
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, [showInsertMenu]);

  const isEmpty = !content || content === "<br>" || content === "";

  return (
    <div className={cn("flex flex-col", className)}>
      {/* Link Modal */}
      <Modal
        isOpen={showLinkModal}
        onClose={() => setShowLinkModal(false)}
        title="Insert Link"
      >
        <form onSubmit={(e) => { e.preventDefault(); insertLink(); }} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="link-url">URL</Label>
            <Input
              id="link-url"
              type="url"
              value={linkUrl}
              onChange={(e) => setLinkUrl(e.target.value)}
              placeholder="https://example.com"
              autoFocus
            />
          </div>
          {!savedSelection?.toString() && (
            <div className="space-y-2">
              <Label htmlFor="link-text">Link Text</Label>
              <Input
                id="link-text"
                type="text"
                value={linkText}
                onChange={(e) => setLinkText(e.target.value)}
                placeholder="Click here"
              />
            </div>
          )}
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="outline" onClick={() => setShowLinkModal(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={!linkUrl || linkUrl === "https://"}>
              Insert Link
            </Button>
          </div>
        </form>
      </Modal>

      {/* Image Modal */}
      <Modal
        isOpen={showImageModal}
        onClose={() => setShowImageModal(false)}
        title="Insert Image"
      >
        <form onSubmit={(e) => { e.preventDefault(); insertImage(); }} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="image-url">Image URL</Label>
            <Input
              id="image-url"
              type="url"
              value={imageUrl}
              onChange={(e) => setImageUrl(e.target.value)}
              placeholder="https://example.com/image.jpg"
              autoFocus
            />
          </div>
          {imageUrl && (
            <div className="border border-border rounded-lg p-2 bg-muted/50">
              <img 
                src={imageUrl} 
                alt="Preview" 
                className="max-h-40 mx-auto rounded"
                onError={(e) => (e.currentTarget.style.display = 'none')}
              />
            </div>
          )}
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="outline" onClick={() => setShowImageModal(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={!imageUrl}>
              Insert Image
            </Button>
          </div>
        </form>
      </Modal>

      {/* Table Modal */}
      <Modal
        isOpen={showTableModal}
        onClose={() => setShowTableModal(false)}
        title="Insert Table"
      >
        <form onSubmit={(e) => { e.preventDefault(); insertTable(); }} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="table-rows">Rows</Label>
              <Input
                id="table-rows"
                type="number"
                min="1"
                max="20"
                value={tableRows}
                onChange={(e) => setTableRows(e.target.value)}
                autoFocus
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="table-cols">Columns</Label>
              <Input
                id="table-cols"
                type="number"
                min="1"
                max="10"
                value={tableCols}
                onChange={(e) => setTableCols(e.target.value)}
              />
            </div>
          </div>
          <div className="text-sm text-muted-foreground">
            This will create a {tableRows} × {tableCols} table with headers in the first row.
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button type="button" variant="outline" onClick={() => setShowTableModal(false)}>
              Cancel
            </Button>
            <Button type="submit">
              Insert Table
            </Button>
          </div>
        </form>
      </Modal>

      {/* Floating Toolbar */}
      <div className="sticky top-20 z-20 flex items-center gap-0.5 p-2 bg-card/95 backdrop-blur-sm border border-border/50 rounded-lg mb-6 flex-wrap shadow-lg">
        {/* Undo/Redo */}
        <ToolbarButton
          icon={<Undo className="w-4 h-4" />}
          onClick={() => execCommand("undo")}
          title="Undo (Ctrl+Z)"
        />
        <ToolbarButton
          icon={<Redo className="w-4 h-4" />}
          onClick={() => execCommand("redo")}
          title="Redo (Ctrl+Shift+Z)"
        />

        <ToolbarSeparator />

        {/* Text Formatting */}
        <ToolbarButton
          icon={<Bold className="w-4 h-4" />}
          onClick={() => execCommand("bold")}
          active={activeFormats.has("bold")}
          title="Bold (Ctrl+B)"
        />
        <ToolbarButton
          icon={<Italic className="w-4 h-4" />}
          onClick={() => execCommand("italic")}
          active={activeFormats.has("italic")}
          title="Italic (Ctrl+I)"
        />
        <ToolbarButton
          icon={<Underline className="w-4 h-4" />}
          onClick={() => execCommand("underline")}
          active={activeFormats.has("underline")}
          title="Underline (Ctrl+U)"
        />
        <ToolbarButton
          icon={<Strikethrough className="w-4 h-4" />}
          onClick={() => execCommand("strikeThrough")}
          active={activeFormats.has("strikeThrough")}
          title="Strikethrough"
        />

        <ToolbarSeparator />

        {/* Highlight */}
        <ToolbarButton
          icon={<Highlighter className="w-4 h-4" />}
          onClick={() => execCommand("backColor", "#fef08a")}
          title="Highlight"
        />

        <ToolbarSeparator />

        {/* Lists */}
        <ToolbarButton
          icon={<List className="w-4 h-4" />}
          onClick={() => execCommand("insertUnorderedList")}
          active={activeFormats.has("ul")}
          title="Bullet list"
        />
        <ToolbarButton
          icon={<ListOrdered className="w-4 h-4" />}
          onClick={() => execCommand("insertOrderedList")}
          active={activeFormats.has("ol")}
          title="Numbered list"
        />

        <ToolbarSeparator />

        {/* Alignment */}
        <ToolbarButton
          icon={<AlignLeft className="w-4 h-4" />}
          onClick={() => execCommand("justifyLeft")}
          title="Align left"
        />
        <ToolbarButton
          icon={<AlignCenter className="w-4 h-4" />}
          onClick={() => execCommand("justifyCenter")}
          title="Align center"
        />
        <ToolbarButton
          icon={<AlignRight className="w-4 h-4" />}
          onClick={() => execCommand("justifyRight")}
          title="Align right"
        />

        <ToolbarSeparator />

        {/* Links */}
        <ToolbarButton
          icon={<Link className="w-4 h-4" />}
          onClick={openLinkModal}
          title="Insert link (Ctrl+K)"
        />
        <ToolbarButton
          icon={<Link2Off className="w-4 h-4" />}
          onClick={removeLink}
          title="Remove link"
        />

        <ToolbarSeparator />

        {/* Insert Menu */}
        <div className="relative">
          <ToolbarButton
            icon={<Plus className="w-4 h-4" />}
            onClick={(e) => {
              e?.stopPropagation();
              setShowInsertMenu(!showInsertMenu);
            }}
            title="Insert..."
          />
          
          {showInsertMenu && (
            <div 
              className="absolute top-full left-0 mt-2 w-48 bg-popover border border-border rounded-lg shadow-xl z-50 py-1"
              onClick={(e) => e.stopPropagation()}
            >
              <button
                className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2 transition-colors"
                onClick={() => { openImageModal(); setShowInsertMenu(false); }}
              >
                <Image className="w-4 h-4" /> Image
              </button>
              <button
                className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2 transition-colors"
                onClick={() => { openTableModal(); setShowInsertMenu(false); }}
              >
                <Table className="w-4 h-4" /> Table
              </button>
              <button
                className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2 transition-colors"
                onClick={() => { insertTodo(); setShowInsertMenu(false); }}
              >
                <CheckSquare className="w-4 h-4" /> Checkbox
              </button>
              <button
                className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2 transition-colors"
                onClick={() => { insertDivider(); setShowInsertMenu(false); }}
              >
                <Minus className="w-4 h-4" /> Divider
              </button>
              <button
                className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2 transition-colors"
                onClick={() => { insertCodeBlock(); setShowInsertMenu(false); }}
              >
                <Code className="w-4 h-4" /> Code block
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Editor Area - Truly endless page */}
      <div className="relative cursor-text">
        {/* The actual editable content */}
        <div
          ref={editorRef}
          contentEditable
          suppressContentEditableWarning
          className={cn(
            "outline-none",
            "prose prose-invert max-w-none",
            "prose-headings:font-semibold prose-headings:text-foreground prose-headings:mt-8 prose-headings:mb-4",
            "prose-p:text-foreground prose-p:leading-relaxed prose-p:my-4",
            "prose-strong:text-foreground prose-strong:font-semibold",
            "prose-em:text-foreground",
            "prose-code:text-primary prose-code:bg-muted prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:font-mono prose-code:text-sm prose-code:before:content-none prose-code:after:content-none",
            "prose-pre:bg-muted prose-pre:text-foreground prose-pre:rounded-lg prose-pre:my-4",
            "prose-ul:text-foreground prose-ol:text-foreground prose-ul:my-4 prose-ol:my-4",
            "prose-li:text-foreground prose-li:my-1",
            "prose-a:text-primary prose-a:underline prose-a:underline-offset-2 hover:prose-a:text-primary/80",
            "prose-img:rounded-lg prose-img:my-4",
            "prose-hr:border-border prose-hr:my-8",
            "prose-table:my-4",
            "prose-th:border prose-th:border-border prose-th:p-2 prose-th:bg-muted",
            "prose-td:border prose-td:border-border prose-td:p-2",
            "[&_mark]:bg-yellow-500/30 [&_mark]:px-1 [&_mark]:rounded",
            "[&_.todo-item]:flex [&_.todo-item]:items-start [&_.todo-item]:gap-2.5 [&_.todo-item]:my-2 [&_.todo-item]:p-2 [&_.todo-item]:rounded-lg [&_.todo-item]:transition-colors [&_.todo-item:hover]:bg-accent/30",
            "[&_.todo-checkbox]:w-5 [&_.todo-checkbox]:h-5 [&_.todo-checkbox]:mt-0.5 [&_.todo-checkbox]:accent-primary [&_.todo-checkbox]:cursor-pointer",
            "[&_.todo-text]:flex-1",
            "text-lg leading-relaxed"
          )}
          onInput={handleInput}
          onKeyDown={handleKeyDown}
          onMouseUp={updateActiveFormats}
          data-placeholder={placeholder}
        />
        
        {/* Placeholder text */}
        {isEmpty && (
          <div className="absolute top-0 left-0 pointer-events-none text-muted-foreground/40 text-lg">
            {placeholder}
          </div>
        )}
        
        {/* Endless scroll area - click to focus editor at end */}
        <div 
          className="h-[100vh] cursor-text"
          onClick={() => {
            if (editorRef.current) {
              editorRef.current.focus();
              // Move cursor to end
              const range = document.createRange();
              const selection = window.getSelection();
              range.selectNodeContents(editorRef.current);
              range.collapse(false);
              selection?.removeAllRanges();
              selection?.addRange(range);
            }
          }}
        />
      </div>

      {/* Footer with word count */}
      <div className="sticky bottom-0 flex items-center justify-between py-3 px-1 text-xs text-muted-foreground bg-background/80 backdrop-blur-sm border-t border-border/30 mt-8">
        <div>
          {content ? (
            <>
              {content.replace(/<[^>]*>/g, "").trim().split(/\s+/).filter(Boolean).length} words
            </>
          ) : (
            "0 words"
          )}
        </div>
        <div className="flex items-center gap-4">
          <span className="hidden sm:inline">Ctrl+S to save</span>
          <span className="hidden sm:inline">•</span>
          <span className="hidden sm:inline">Ctrl+K for link</span>
        </div>
      </div>
    </div>
  );
}
