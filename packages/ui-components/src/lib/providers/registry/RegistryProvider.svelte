<script lang="ts">
  import { setRegistryContext } from './context';
  import { DotrainRegistry } from '@rainlanguage/orderbook';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { writable, derived } from 'svelte/store';

  // Optional default URL to use when no `registry` query param is set
  export let defaultUrl: string | undefined;

  const registry = writable<DotrainRegistry | null>(null);
  const loading = writable(false);
  const error = writable<string | null>(null);
  const registryUrl = writable<string>('');
  const isCustomRegistry = derived(registryUrl, (u) => Boolean(u));

  async function init(url: string) {
    loading.set(true);
    error.set(null);
    registry.set(null);
    try {
      const result = await DotrainRegistry.new(url);
      if (result.error) {
        error.set(result.error.readableMsg ?? result.error.msg ?? 'Failed to initialize registry');
        registry.set(null);
      } else {
        registry.set(result.value as DotrainRegistry);
      }
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
      registry.set(null);
    } finally {
      loading.set(false);
    }
  }

  function setRegistryUrl(url: string) {
    try {
      const current = new URL(window.location.href);
      if (url) current.searchParams.set('registry', url);
      else current.searchParams.delete('registry');
      window.history.pushState({}, '', current.toString());
      registryUrl.set(url);
      void init(url || (defaultUrl ?? ''));
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
    }
  }

  function appendRegistryToHref(href: string): string {
    try {
      const url = new URL(href, window.location.origin);
      const current = $registryUrl;
      if (current) url.searchParams.set('registry', current);
      return url.pathname + (url.search ? `?${url.searchParams.toString()}` : '');
    } catch {
      // Fallback: do not mutate href if URL parsing fails
      const current = $registryUrl;
      if (!current) return href;
      const hasQuery = href.includes('?');
      const sep = hasQuery ? '&' : '?';
      return `${href}${sep}registry=${encodeURIComponent(current)}`;
    }
  }

  // Set context once during component initialization
  setRegistryContext({ registry, loading, error, setRegistryUrl, registryUrl, isCustomRegistry, appendRegistryToHref });

  onMount(() => {
    const urlParam = $page.url.searchParams.get('registry');
    const url = urlParam || defaultUrl || '';
    registryUrl.set(urlParam ?? '');
    if (url) void init(url);
  });
</script>

<slot />
