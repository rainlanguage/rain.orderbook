<script lang="ts">
  import CodeMirror from 'svelte-codemirror-editor';
  import { codeMirrorTheme } from '$lib/stores/darkMode';
  import { yaml } from '@codemirror/lang-yaml';
  import { parseConfigProblems } from '$lib/services/configCodemirrorProblems';
  import { RawRainlangExtension } from 'codemirror-rainlang';
  import { openLintPanel } from '@codemirror/lint';

  export let value: string;
  export let disabled = false;
  export let styles = {};

  const configStringExtension = new RawRainlangExtension({
    hover: async () => null,
    completion: async () => null,
    diagnostics: async (textDocument) => parseConfigProblems(textDocument.text),
  });
</script>

<CodeMirror
  bind:value
  extensions={[configStringExtension]}
  lang={yaml()}
  theme={$codeMirrorTheme}
  readonly={disabled}
  useTab={true}
  tabSize={2}
  styles={{
    '&': {
      width: '100%',
    },
    ...styles,
  }}
  on:ready={(e) => {
    openLintPanel(e.detail);
  }}
/>

<style global>
  :global(.ͼ1 .cm-panel.cm-panel-lint ul [aria-selected]) {
    background-color: inherit;
  }
</style>
