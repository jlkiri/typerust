<script lang="ts">
  import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
  import { rust } from "@codemirror/lang-rust";
  import { indentUnit } from "@codemirror/language";
  import { onMount } from "svelte";
  import { code } from "./code";
  import { oneDark } from "@codemirror/theme-one-dark";
  import type { Extension } from "@codemirror/state";
  import lz from "lz-string";
  import ActionButton from "./ActionButton.svelte";
  import * as fmt from "formatter";

  let parent = null;
  let editor: EditorView = null;

  let isDarkTheme = true;

  const codeUpdateListener = EditorView.updateListener.of((update) => {
    const codeText = update.state.toJSON().doc;
    code.set(codeText);
    debouncedHashUpdater(codeText);
  });

  const fixedHeightEditor = EditorView.theme({
    "&": {
      height: "100%",
      "max-height": "100vh",
    },
    ".cm-content": {
      "font-family": "Source Code Pro, monospace",
    },
    ".cm-scroller": { overflow: "auto" },
  });

  function debounce(cb: (e: string) => void) {
    let timeout = null;
    return function (e: string) {
      clearTimeout(timeout);
      timeout = setTimeout(() => cb(e), 500);
    };
  }

  function updateHash(code: string) {
    if (location.hash != "") {
      const compressedCode = lz.compressToEncodedURIComponent(code);
      location.hash = compressedCode;
    }
  }

  const debouncedHashUpdater = debounce(updateHash);

  const basicExtensions = [
    basicSetup,
    rust(),
    EditorView.lineWrapping,
    EditorState.tabSize.of(4),
    indentUnit.of("    "),
  ];

  const defaultCode = `fn main() {\n    println!("Hello, world!");\n}\n`;
  const initialCode =
    location.hash != ""
      ? lz.decompressFromEncodedURIComponent(location.hash.slice(1))
      : defaultCode;

  function toggleEditorTheme() {
    isDarkTheme = !isDarkTheme;

    const previousCode = $code;
    editor.destroy();

    let extensions = [codeUpdateListener, fixedHeightEditor];
    if (isDarkTheme) {
      extensions.push(oneDark);
    }

    mountEditor(extensions);

    editor.dispatch({
      changes: {
        from: 0,
        insert: previousCode,
      },
    });
  }

  function format() {
    const prettyCode = fmt.pretty($code);
    editor.dispatch({
      changes: {
        from: 0,
        to: editor.state.doc.length,
        insert: prettyCode,
      },
    });
  }

  function mountEditor(...additionalExtensions: Array<Extension>) {
    editor = new EditorView({
      state: EditorState.create({
        extensions: [...basicExtensions, ...additionalExtensions],
      }),
      parent: parent,
    });
  }

  async function initializeFormatter() {
    await fmt.default();
  }

  onMount(initializeFormatter);

  onMount(() => {
    mountEditor(codeUpdateListener, fixedHeightEditor, oneDark);
    editor.dispatch({
      changes: {
        from: 0,
        insert: initialCode,
      },
    });
  });
</script>

<div class="editor-wrapper">
  <div class="editor" bind:this={parent} />
  <div class="toggle">
    <ActionButton dark={isDarkTheme} on:click={toggleEditorTheme}
      >{isDarkTheme ? "Light" : "Dark"}</ActionButton
    >
    <ActionButton on:click={format}>Format 📝</ActionButton>
  </div>
</div>

<style>
  .editor-wrapper {
    position: relative;
    flex: 1;
  }

  .editor {
    flex: 1;
    height: 100%;
  }

  .editor-wrapper .toggle {
    position: absolute;
    top: 1em;
    right: 1em;
    color: white;
  }

  :global(.cm-editor) {
    font-size: 1rem;
  }
</style>
