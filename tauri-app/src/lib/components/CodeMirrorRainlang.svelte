<script lang="ts">
  import CodeMirror from 'svelte-codemirror-editor';
  import { RainlangLR } from 'codemirror-rainlang';
  import { codeMirrorTheme } from '$lib/stores/darkMode';

  export let value: string;
  export let disabled = false;
  export let styles = {};
</script>

<CodeMirror
  bind:value
  extensions={[RainlangLR]}
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
/>
