<script lang="ts">
  import { IconTelegram } from '@rainlanguage/ui-components';
  import { Button } from 'flowbite-svelte';
</script>

<div class="mx-auto flex max-w-prose flex-col items-center gap-y-8 pt-12">
  <img alt="Raindex logo" class="w-[400px]" src="/logo-light.svg" />
  <div class="mt-4 text-center text-2xl">
    Raindex allows anyone to write, deploy and manage token trading strategies, written in Rainlang,
    on any EVM network.
  </div>
  <div class="flex items-center gap-x-2">
    <Button target="_blank" href="https://t.me/+W0aQ36ptN_E2MjZk">
      <IconTelegram dark />
      <span class="ml-2">Join the community</span>
    </Button>
    <Button target="_blank" href="https://docs.rainlang.xyz/raindex/getting-started"
      >Get started</Button
    >
  </div>
  <div style="position: relative; padding-bottom: 64.63195691202873%; height: 0;" class="w-full">
    <!--    <iframe-->
    <!--      title="Raindex Demo"-->
    <!--      src="https://www.loom.com/embed/fca750f31f0a43258891cea0ddacb588?sid=21583276-742b-49d5-a7db-b6e9e01ca418"-->
    <!--      frameborder="0"-->
    <!--      allowfullscreen-->
    <!--      style="position: absolute; top: 0; left: 0; width: 100%; height: 100%;"-->
    <!--    ></iframe>-->
  </div>
</div>
