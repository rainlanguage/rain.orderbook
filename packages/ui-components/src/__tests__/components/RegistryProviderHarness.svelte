<script lang="ts">
  import RegistryProvider from '$lib/providers/registry/RegistryProvider.svelte';

  export let defaultUrl: string | undefined = undefined;
</script>

<RegistryProvider {defaultUrl}>
  <slot />
</RegistryProvider>
