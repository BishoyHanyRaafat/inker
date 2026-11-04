// Editor types and utilities for the markdown editor

export type MarkdownContent = string;

export interface EditorState {
  content: MarkdownContent;
  version: number;
}
