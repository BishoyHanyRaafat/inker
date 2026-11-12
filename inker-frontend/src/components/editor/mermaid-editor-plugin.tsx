

import CodeBlockLowlight, {
  type CodeBlockLowlightOptions,
} from "@tiptap/extension-code-block-lowlight";

import type { MermaidConfig } from "mermaid";
import { mermaidPlugin } from "prosemirror-mermaid";
import { Plugin } from "@tiptap/pm/state";

interface CodeBlockLowlightMermaidOptions
  extends CodeBlockLowlightOptions {
  debounce?: number;
  mermaidConfig?: MermaidConfig;
  classList: string[] | string;
}

function mermaidIdPlugin(nodeName: string) {
  return new Plugin({
    appendTransaction: (_transactions, _oldState, newState) => {
      const tr = newState.tr;
      let modified = false;

      newState.doc.descendants((node, pos) => {
        if (node.type.name !== nodeName || node.attrs.id) {
          return;
        }

        tr.setNodeMarkup(pos, undefined, {
          ...node.attrs,
          id: `m${crypto.randomUUID().slice(0, 8)}`,
        });

        modified = true;
      });

      return modified ? tr : null;
    },
  });
}

export const CodeBlockLowlightMermaid =
  CodeBlockLowlight.extend<CodeBlockLowlightMermaidOptions>({
    addOptions() {
      return {
        ...this.parent?.(),
        lowlight: {},
        languageClassPrefix: "language-",
        exitOnTripleEnter: true,
        exitOnArrowDown: true,
        defaultLanguage: null,
        enableTabIndentation: false,
        tabSize: 4,
        HTMLAttributes: {},
        debounce: 300,
        mermaidConfig: {},
        classList: "mermaid-container",
      };
    },

    addAttributes() {
      const parentAttrs = this.parent?.() ?? {};

      return {
        ...parentAttrs,
        id: {
          default: null,

          parseHTML: (element) =>
            element.getAttribute("data-id"),

          renderHTML: (attributes) => {
            if (!attributes.id) return {};

            return {
              "data-id": attributes.id,
            };
          },
        },
      };
    },

    addProseMirrorPlugins() {
      return [
        ...(this.parent?.() || []),

        mermaidIdPlugin(this.name),

        mermaidPlugin({
          name: this.name,
          lowlight: this.options.lowlight,
          classList: this.options.classList,
          mermaidConfig: this.options.mermaidConfig,
          debounce: this.options.debounce,
        }),
      ];
    },
  });
