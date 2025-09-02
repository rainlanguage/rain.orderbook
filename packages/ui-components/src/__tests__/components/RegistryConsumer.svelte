<script lang="ts">
  import { useRegistry } from '$lib/providers/registry/useRegistry';

  const {
    appendRegistryToHref,
    setRegistryUrl,
    isCustomRegistry,
    registryUrl
  } = useRegistry();

  // Expose for assertions if needed
  export let onSet: ((val: string) => void) | undefined;
  function handleSet(val: string) {
    setRegistryUrl(val);
    onSet && onSet(val);
  }
</script>

<div data-testid="href">{appendRegistryToHref('/deploy/test')}</div>
<div data-testid="is-custom">{$isCustomRegistry ? 'yes' : 'no'}</div>
<div data-testid="url">{$registryUrl}</div>
<button data-testid="set-btn" on:click={() => handleSet('custom')}>set</button>

