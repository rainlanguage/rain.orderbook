<script lang="ts">
  import { Heading } from 'flowbite-svelte';
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';

  redirectIfSettingsNotDefined();
</script>

<Heading tag="h1" class="text-center text-4xl font-bold">Vaults</Heading>
