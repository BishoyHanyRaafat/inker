"use client";

import { useState, useCallback } from "react";
import type { ImageBlock as ImageBlockType } from "@/lib/api/types.gen";
import { cn } from "@/lib/utils";
import { Image as ImageIcon, Link, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface ImageBlockProps {
  data: ImageBlockType;
  onChange: (data: ImageBlockType) => void;
  onFocus?: () => void;
}

export function ImageBlock({ data, onChange, onFocus }: ImageBlockProps) {
  const [isEditing, setIsEditing] = useState(!data.url);
  const [urlInput, setUrlInput] = useState(data.url);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (!urlInput.trim()) {
        setError("Please enter an image URL");
        return;
      }

      // Basic URL validation
      try {
        new URL(urlInput);
      } catch {
        setError("Please enter a valid URL");
        return;
      }

      setError(null);
      setIsLoading(true);

      // Test if image loads
      const img = new window.Image();
      img.onload = () => {
        onChange({ url: urlInput.trim() });
        setIsEditing(false);
        setIsLoading(false);
      };
      img.onerror = () => {
        setError("Could not load image from this URL");
        setIsLoading(false);
      };
      img.src = urlInput;
    },
    [urlInput, onChange]
  );

  const handleRemove = useCallback(() => {
    onChange({ url: "" });
    setUrlInput("");
    setIsEditing(true);
  }, [onChange]);

  if (isEditing || !data.url) {
    return (
      <div
        className="border border-dashed border-border rounded-lg p-6 bg-card/30"
        onClick={onFocus}
      >
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="flex flex-col items-center gap-3 text-muted-foreground">
            <div className="w-12 h-12 rounded-lg bg-muted/50 flex items-center justify-center">
              <ImageIcon className="w-6 h-6" />
            </div>
            <p className="text-sm">Add an image from URL</p>
          </div>
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Link className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <Input
                type="url"
                value={urlInput}
                onChange={(e) => {
                  setUrlInput(e.target.value);
                  setError(null);
                }}
                placeholder="Paste image URL..."
                className="pl-10"
                autoFocus
              />
            </div>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? "Loading..." : "Embed"}
            </Button>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
        </form>
      </div>
    );
  }

  return (
    <div className="relative group rounded-lg overflow-hidden" onClick={onFocus}>
      <img
        src={data.url}
        alt=""
        className="max-w-full h-auto rounded-lg"
        loading="lazy"
      />
      <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity duration-150 flex gap-1">
        <Button
          variant="secondary"
          size="icon-sm"
          onClick={() => setIsEditing(true)}
          className="bg-background/80 backdrop-blur-sm"
        >
          <Link className="w-3.5 h-3.5" />
        </Button>
        <Button
          variant="secondary"
          size="icon-sm"
          onClick={handleRemove}
          className="bg-background/80 backdrop-blur-sm hover:bg-destructive hover:text-white"
        >
          <X className="w-3.5 h-3.5" />
        </Button>
      </div>
    </div>
  );
}
