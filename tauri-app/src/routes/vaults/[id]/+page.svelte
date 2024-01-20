<script lang="ts">
  import { Heading, Button, Card } from 'flowbite-svelte';
  import { vaultDetail } from '$lib/stores/vaultDetail';
  export let data: { id: string };

  vaultDetail.refetch(data.id);
  $: vault = $vaultDetail[data.id];
</script>

<Heading tag="h1" class="mb-8 text-center text-4xl font-bold">Vault Detail</Heading>

<div class="mb-8 flex w-full justify-center">
  <Card class="w-full space-y-8" size="lg">
    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Vault ID
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {data.id}
      </p>
    </div>

    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Owner Address
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {vault.owner.id}
      </p>
    </div>

    <div>
      <div class="flex justify-center space-x-20">
        <Button color="green" size="xl">Deposit</Button>
        <Button color="blue" size="xl">Withdraw</Button>
      </div>
    </div>
  </Card>
</div>
