<script lang="ts">
  import { platform } from '@tauri-apps/api/os';

  let currentPlatform: string | undefined = undefined;

  async function setup() {
    currentPlatform = await platform();
  }
  setup();
</script>

<!-- The titlebar is only hidden on macos -->
{#if currentPlatform === 'darwin'}
  <!-- On this div the tauri window will be draggable -->
  <div class="absolute w-full h-8 top-0 z-50" data-tauri-drag-region></div>
{/if}