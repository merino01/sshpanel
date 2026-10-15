/**
 * El área de edición del Editor, con CodeMirror 6 (ADR 0008): números de
 * línea, deshacer, buscar y reemplazar, y resaltado según el nombre del
 * archivo. Los colores salen del Tema: los de la interfaz como variables CSS y
 * los del resaltado de la paleta de la Terminal, pensada para leerse sobre su
 * fondo. Respeta los finales de línea del archivo (LF o CRLF).
 */
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching, HighlightStyle, indentOnInput, LanguageDescription, syntaxHighlighting } from "@codemirror/language";
import { languages } from "@codemirror/language-data";
import { highlightSelectionMatches, search, searchKeymap } from "@codemirror/search";
import { Compartment, EditorState } from "@codemirror/state";
import {
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { t } from "../../i18n";
import { languageHint, lineSeparator } from "../../lib/textFile";
import type { ThemePalette } from "../../lib/themes";

export interface EditorOptions {
  text: string;
  /** Nombre del archivo, para elegir el resaltado. */
  name: string;
  palette: ThemePalette;
  onChange: () => void;
  onCursor: (line: number, column: number) => void;
  /** Con el nombre del lenguaje resaltado, cuando se sabe. */
  onLanguage: (name: string) => void;
}

export interface EditorHandle {
  /** El texto actual, con los finales de línea del archivo. */
  text: () => string;
  setPalette: (palette: ThemePalette) => void;
  focus: () => void;
  destroy: () => void;
}

/** Los textos del panel de buscar de CodeMirror, en el idioma de la interfaz. */
function phrases(): Record<string, string> {
  return {
    Find: t("editor.search.find"),
    Replace: t("editor.search.replace"),
    next: t("editor.search.next"),
    previous: t("editor.search.previous"),
    all: t("editor.search.all"),
    "match case": t("editor.search.matchCase"),
    regexp: t("editor.search.regexp"),
    "by word": t("editor.search.byWord"),
    replace: t("editor.search.replaceOne"),
    "replace all": t("editor.search.replaceAll"),
    close: t("editor.search.close"),
    "Go to line": t("editor.search.goToLine"),
    go: t("editor.search.go"),
  };
}

const MONOSPACE = "ui-monospace, 'Cascadia Mono', 'DejaVu Sans Mono', monospace";

/** El aspecto del Editor con el Tema `palette`. */
function themeFor(palette: ThemePalette) {
  const code = palette.terminal;
  const view = EditorView.theme(
    {
      "&": { height: "100%", backgroundColor: code.background, color: code.foreground },
      ".cm-scroller": { fontFamily: MONOSPACE, fontSize: "0.875rem", lineHeight: "1.55" },
      ".cm-content": { padding: "0.75rem 0", caretColor: "var(--accent)" },
      ".cm-line": { padding: "0 1rem 0 0.75rem" },
      "&.cm-focused": { outline: "none" },
      ".cm-cursor, .cm-dropCursor": { borderLeft: "2px solid var(--accent)" },
      "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, ::selection": {
        backgroundColor: code.selection,
      },
      ".cm-activeLine": { backgroundColor: "color-mix(in srgb, var(--accent) 7%, transparent)" },
      ".cm-gutters": { backgroundColor: code.background, color: code.brightBlack, border: "none", borderRight: "1px solid var(--border)" },
      ".cm-lineNumbers .cm-gutterElement": { padding: "0 0.75rem 0 1rem", minWidth: "2.5ch" },
      ".cm-activeLineGutter": { backgroundColor: "transparent", color: code.foreground },
      ".cm-matchingBracket": { backgroundColor: "color-mix(in srgb, var(--accent) 25%, transparent)", outline: "none" },
      ".cm-selectionMatch": { backgroundColor: "color-mix(in srgb, var(--accent) 15%, transparent)" },
      ".cm-searchMatch": { backgroundColor: "color-mix(in srgb, var(--warning) 30%, transparent)" },
      ".cm-searchMatch-selected": { backgroundColor: "color-mix(in srgb, var(--warning) 55%, transparent)" },
      ".cm-specialChar": { color: code.red },
      ".cm-panels": { backgroundColor: "var(--surface)", color: "var(--fg)", fontFamily: "system-ui, sans-serif" },
      ".cm-panels.cm-panels-top": { borderBottom: "1px solid var(--border)" },
      ".cm-panel.cm-search": { padding: "0.5rem 2.5rem 0.5rem 0.75rem", fontSize: "0.85rem" },
      ".cm-panel.cm-search label": { fontSize: "0.85rem" },
      ".cm-textfield": {
        padding: "0.25rem 0.5rem",
        border: "1px solid var(--border-strong)",
        borderRadius: "0.35rem",
        background: "var(--bg)",
        color: "var(--fg)",
        font: "inherit",
      },
      ".cm-button": {
        padding: "0.25rem 0.6rem",
        border: "1px solid var(--border-strong)",
        borderRadius: "0.35rem",
        backgroundImage: "none",
        background: "var(--bg)",
        color: "var(--fg)",
        font: "inherit",
      },
      ".cm-panel button[name=close]": { color: "var(--fg)", fontSize: "1.1rem", right: "0.6rem", top: "0.35rem" },
    },
    { dark: palette.scheme === "dark" },
  );
  const highlight = HighlightStyle.define([
    { tag: [tags.comment, tags.meta], color: code.brightBlack, fontStyle: "italic" },
    { tag: [tags.keyword, tags.controlKeyword, tags.moduleKeyword, tags.operatorKeyword], color: code.magenta },
    { tag: [tags.string, tags.special(tags.string), tags.character], color: code.green },
    { tag: [tags.number, tags.bool, tags.null, tags.atom, tags.unit], color: code.yellow },
    { tag: [tags.regexp, tags.escape], color: code.cyan },
    { tag: [tags.function(tags.variableName), tags.function(tags.propertyName), tags.definition(tags.variableName)], color: code.blue },
    { tag: [tags.propertyName, tags.attributeName], color: code.cyan },
    { tag: [tags.typeName, tags.className, tags.namespace], color: code.brightYellow },
    { tag: [tags.tagName, tags.heading], color: code.red, fontWeight: "600" },
    { tag: [tags.labelName, tags.self, tags.special(tags.variableName)], color: code.brightMagenta },
    { tag: tags.link, color: code.blue, textDecoration: "underline" },
    { tag: tags.strong, fontWeight: "700" },
    { tag: tags.emphasis, fontStyle: "italic" },
    { tag: tags.invalid, color: code.brightRed },
  ]);
  return [view, syntaxHighlighting(highlight)];
}

/** El lenguaje para resaltar `name`, si se conoce. */
function languageOf(name: string): LanguageDescription | null {
  const hint = languageHint(name);
  return (hint && LanguageDescription.matchLanguageName(languages, hint)) || LanguageDescription.matchFilename(languages, name);
}

/** Monta el área de edición en `parent` con el texto del archivo. */
export function createEditor(parent: HTMLElement, options: EditorOptions): EditorHandle {
  const theme = new Compartment();
  const language = new Compartment();
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: options.text,
      extensions: [
        EditorState.lineSeparator.of(lineSeparator(options.text)),
        EditorState.phrases.of(phrases()),
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        drawSelection(),
        dropCursor(),
        indentOnInput(),
        bracketMatching(),
        highlightActiveLine(),
        highlightSelectionMatches(),
        search({ top: true }),
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, indentWithTab]),
        EditorView.contentAttributes.of({ "aria-label": t("editor.textLabel", { name: options.name }) }),
        theme.of(themeFor(options.palette)),
        language.of([]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) options.onChange();
          if (update.docChanged || update.selectionSet) {
            const head = update.state.selection.main.head;
            const line = update.state.doc.lineAt(head);
            options.onCursor(line.number, head - line.from + 1);
          }
        }),
      ],
    }),
  });

  // Cada lenguaje se descarga aparte, solo cuando se abre un archivo suyo.
  const description = languageOf(options.name);
  if (description) {
    description
      .load()
      .then((support) => {
        view.dispatch({ effects: language.reconfigure(support) });
        options.onLanguage(description.name);
      })
      .catch(() => {});
  }

  return {
    // `sliceDoc` une las líneas con el separador del archivo; `doc.toString()`, siempre con LF.
    text: () => view.state.sliceDoc(),
    setPalette: (palette) => view.dispatch({ effects: theme.reconfigure(themeFor(palette)) }),
    focus: () => view.focus(),
    destroy: () => view.destroy(),
  };
}
