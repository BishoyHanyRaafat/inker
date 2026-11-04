import { useState, useCallback } from "react";
import { cn } from "../../lib/utils";
import MDEditor from "@uiw/react-md-editor";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface DocumentEditorProps {
  content: string;
  onChange: (content: string) => void;
  onSave?: () => void;
  placeholder?: string;
  className?: string;
  autoFocus?: boolean;
}

export function DocumentEditor({
  content,
  onChange,
  placeholder = "Start writing...",
  className,
}: DocumentEditorProps) {
  const handleEditorChange = useCallback((value: string) => {
    onChange(value);
  }, [onChange]);

  return (
    <div className={cn("flex flex-col gap-4", className)}>
      <div className="border border-border rounded-lg overflow-hidden bg-card shadow-sm">
        <MDEditor
          value={content}
          onChange={handleEditorChange}
          placeholder={placeholder}
          preview="edit"
          height={70}
          className="prose-invert"
        />
      </div>

      {/* Preview area for better UX if needed,
          although MDEditor provides a side-by-side preview */}
      <div className="mt-6 p-6 border border-border rounded-lg bg-background prose prose-invert max-w-none">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>
          {content}
        </ReactMarkdown>
      </div>
    </div>
  );
}
