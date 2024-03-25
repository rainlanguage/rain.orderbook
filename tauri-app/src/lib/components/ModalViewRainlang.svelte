<script lang="ts">
  import { Modal } from 'flowbite-svelte';
  import CodeMirrorRainlang from './CodeMirrorRainlang.svelte';

  export let text = "";
  export let open = false;
</script>

<Modal title="Generated Rainlang" bind:open outsideclose size="sm">
  <CodeMirrorRainlang value={text} disabled={true}/>
</Modal>
