"use client";

import { useRef, useEffect, useCallback } from "react";
import type { TextBlock as TextBlockType, TextSegment, TextMark } from "@/lib/editor-types";
import { cn } from "@/lib/utils";

interface TextBlockProps {
  data: TextBlockType;
  onChange: (data: TextBlockType) => void;
  onKeyDown?: (e: React.KeyboardEvent) => void;
  onFocus?: () => void;
  placeholder?: string;
  autoFocus?: boolean;
  className?: string;
}

// Render a text segment with its marks
function renderSegment(segment: TextSegment, index: number) {
  let content: React.ReactNode = segment.text;

  // Apply marks in order
  segment.marks.forEach((mark) => {
    switch (mark) {
      case "Bold":
        content = <strong key={`${index}-bold`}>{content}</strong>;
        break;
      case "Italic":
        content = <em key={`${index}-italic`}>{content}</em>;
        break;
      case "Underline":
        content = (
          <span key={`${index}-underline`} className="underline">
            {content}
          </span>
        );
        break;
      case "StrikeThrough":
        content = <s key={`${index}-strike`}>{content}</s>;
        break;
      case "Code":
        content = (
          <code
            key={`${index}-code`}
            className="px-1.5 py-0.5 rounded bg-muted font-mono text-sm text-primary"
          >
            {content}
          </code>
        );
        break;
      case "Highlight":
        content = (
          <mark
            key={`${index}-highlight`}
            className="bg-yellow-500/30 px-0.5 rounded"
          >
            {content}
          </mark>
        );
        break;
      case "Link":
        content = (
          <a
            key={`${index}-link`}
            href={segment.text}
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary underline hover:text-primary/80"
          >
            {content}
          </a>
        );
        break;
    }
  });

  return <span key={index}>{content}</span>;
}

export function TextBlock({
  data,
  onChange,
  onKeyDown,
  onFocus,
  placeholder = "Type '/' for commands...",
  autoFocus = false,
  className,
}: TextBlockProps) {
  const editorRef = useRef<HTMLDivElement>(null);

  // Get plain text from segments
  const getPlainText = useCallback(() => {
    return data.segments.map((s) => s.text).join("");
  }, [data.segments]);

  // Update content from contentEditable
  const handleInput = useCallback(() => {
    if (!editorRef.current) return;

    const text = editorRef.current.innerText || "";

    // For now, we treat all content as a single segment
    // A more advanced implementation would preserve marks
    const newSegments: TextSegment[] = [{ text, marks: [] }];

    onChange({ segments: newSegments });
  }, [onChange]);

  // Handle paste to strip formatting
  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    e.preventDefault();
    const text = e.clipboardData.getData("text/plain");
    document.execCommand("insertText", false, text);
  }, []);

  // Focus the editor on mount if autoFocus
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

  const isEmpty = getPlainText() === "";

  return (
    <div className="relative group">
      <div
        ref={editorRef}
        contentEditable
        suppressContentEditableWarning
        className={cn(
          "outline-none min-h-[1.5em] py-1 px-1 -mx-1 rounded",
          "focus:bg-accent/30 transition-colors duration-150",
          "text-foreground leading-relaxed",
          className
        )}
        onInput={handleInput}
        onKeyDown={onKeyDown}
        onFocus={onFocus}
        onPaste={handlePaste}
        data-placeholder={placeholder}
      >
        {data.segments.map((segment, i) => renderSegment(segment, i))}
      </div>
      {isEmpty && (
        <div className="absolute top-1 left-1 pointer-events-none text-muted-foreground/50">
          {placeholder}
        </div>
      )}
    </div>
  );
}
