<script lang="ts">
  import IconTelegram from '$lib/components/IconTelegram.svelte';
  import { Button, Heading } from 'flowbite-svelte';
</script>

<div class="mx-auto flex max-w-prose flex-col items-center gap-y-8 pt-12">
  <img class="w-[400px]" src="/logo-light.svg" />
  <div class="mt-4 text-2xl">
    Raindex is an onchain orderbook contract that enables users to deploy complex, onchain
    strategies for managing token trades.
  </div>
  <div class="flex items-center gap-x-2">
    <Button href="https://t.me/+W0aQ36ptN_E2MjZk">
      <IconTelegram dark />
      <span class="ml-2">Join the community</span>
    </Button>
    <Button href="https://docs.rainlang.xyz/raindex/getting-started">Get started</Button>
  </div>
  <div style="position: relative; padding-bottom: 64.63195691202873%; height: 0;" class="w-full">
    <iframe
      src="https://www.loom.com/embed/fca750f31f0a43258891cea0ddacb588?sid=21583276-742b-49d5-a7db-b6e9e01ca418"
      frameborder="0"
      allowfullscreen
      style="position: absolute; top: 0; left: 0; width: 100%; height: 100%;"
    ></iframe>
  </div>
</div>
