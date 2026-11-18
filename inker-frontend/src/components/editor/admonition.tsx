
import { Node } from "@tiptap/core";

const ADMONITION_TYPES = [
  "note",
  "tip",
  "info",
  "warning",
  "danger",
] as const;

type AdmonitionType = (typeof ADMONITION_TYPES)[number];

function isAdmonitionType(value: string): value is AdmonitionType {
  return ADMONITION_TYPES.includes(value as AdmonitionType);
}

export const Admonition = Node.create({
  name: "admonition",

  group: "block",
  content: "block+",
  defining: true,

  addAttributes() {
    return {
      type: {
        default: "note",

        parseHTML: (element) =>
          element.getAttribute("data-type") || "note",

        renderHTML: (attributes) => ({
          "data-type": attributes.type,
        }),
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: "div[data-admonition]",
      },
    ];
  },

  renderHTML({ node, HTMLAttributes }) {
    const type = isAdmonitionType(node.attrs.type)
      ? node.attrs.type
      : "note";

    return [
      "div",
      {
        ...HTMLAttributes,
        "data-admonition": "",
        "data-type": type,
        class: `admonition admonition-${type}`,
      },
      0,
    ];
  },

  markdownTokenizer: {
    name: "admonition",
    level: "block",

    start: (source) => source.indexOf(":::"),

    tokenize: (source, _tokens, lexer) => {
      const match =
        /^:::(note|tip|info|warning|danger)[ \t]*\r?\n([\s\S]*?)\r?\n:::[ \t]*(?:\r?\n|$)/.exec(
          source,
        );

      if (!match) return undefined;

      return {
        type: "admonition",
        raw: match[0],
        admonitionType: match[1],
        tokens: lexer.blockTokens(match[2]),
      };
    },
  },

  parseMarkdown: (token, helpers) => {
    const type = isAdmonitionType(token.admonitionType)
      ? token.admonitionType
      : "note";

    return {
      type: "admonition",
      attrs: { type },
      content: helpers.parseChildren(token.tokens || []),
    };
  },

  renderMarkdown: (node, helpers) => {
    const type = isAdmonitionType(node.attrs?.type)
      ? node.attrs.type
      : "note";

    const content = helpers.renderChildren(node.content || []);

    return `:::${type}\n\n${content}\n:::\n\n`;
  },
});
