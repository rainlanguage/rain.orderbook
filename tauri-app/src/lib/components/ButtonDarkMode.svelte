<script lang="ts">
  import { DarkMode } from "flowbite-svelte";
  import { colorTheme } from "$lib/stores/darkMode";

  function toggle (e: MouseEvent)  {
    const isDark = (e.target as HTMLElement).ownerDocument.documentElement.classList.contains('dark');
    colorTheme.set(isDark ? 'dark' : 'light');
  }
</script>

<button type="button" on:click={toggle}>
  <DarkMode  />
</button>