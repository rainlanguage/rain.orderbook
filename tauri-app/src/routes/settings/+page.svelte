<script lang="ts">
  import { Heading, Button, Label, Input, Helper, Badge } from 'flowbite-svelte';
  import BadgeExternalLink from '../../lib/BadgeExternalLink.svelte';
</script>

<Heading tag="h1" class="mb-8 text-center text-4xl font-bold">Settings</Heading>

<div class="mb-4">
  <Label class="mb-2 block">Blockchain RPC URL</Label>
  <Input label="RPC URL" name="rpcUrl" required />
  <Helper class="mt-2 text-sm">
    The URL of the Blockchain node RPC endpoint you will use to submit Orderbook transactions. You
    can setup a hosted RPC account at <BadgeExternalLink href="https://infura.io" text="Infura" />
    for a reliable hosted RPC service provider. Or visit
    <BadgeExternalLink href="https://chainlist.org/" text="Chainlist" /> for find other publically available
    RPC providerss.
  </Helper>
</div>

<div class="mb-12">
  <Label class="mb-2 block">Subgraph URL</Label>
  <Input label="Subgraph URL" name="subgraphUrl" required />
  <Helper class="mt-2 text-sm">
    The URL of the Subgraph you will use to query Orderbook data. Contact us for help setting up a
    Subgraph for your Orderbook deployment.
  </Helper>
</div>

<Button color="green">Save Settings</Button>
