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
  <div class="absolute w-full h-[28px] top-0 z-100" data-tauri-drag-region></div>
{/if}