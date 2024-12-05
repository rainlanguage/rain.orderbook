<script>
  export let depth;
</script>

{#if depth === 1}
  <div class="mb-4 text-4xl font-medium dark:text-white"><slot /></div>
{:else if depth === 2}
  <div class="my-4 text-2xl font-medium dark:text-white"><slot /></div>
{:else if depth === 3}
  <div class="my-4 text-xl font-medium dark:text-white"><slot /></div>
{:else if depth === 4}
  <div class="my-4 text-lg font-medium dark:text-white"><slot /></div>
{:else if depth === 5}
  <h5><slot /></h5>
{:else if depth === 6}
  <h6><slot /></h6>
{:else}
  <slot />
{/if}
