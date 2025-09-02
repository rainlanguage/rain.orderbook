<script lang="ts">
    import DeploymentTile from './DeploymentTile.svelte';
    import { useRegistry } from '$lib/providers/registry/useRegistry';
    import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook';
    export let orderName: string;

    const { registry } = useRegistry();

    let deployments: Array<[string, NameAndDescriptionCfg]> = [];
    let error: string | null = null;

    $: if ($registry) {
        const result = $registry.getDeploymentDetails(orderName);
        if (result.error) {
            error = result.error.readableMsg ?? result.error.msg ?? 'Failed to load deployments';
            deployments = [];
        } else {
            const map: Map<string, NameAndDescriptionCfg> = result.value ?? new Map();
            deployments = Array.from(map.entries());
            error = null;
        }
    }
</script>

{#if error}
    <p class="text-red-500">Error loading deployments:</p>
    <p class="text-gray-500">{error}</p>
{/if}

{#if deployments.length}
    <div
        class="mr-auto grid w-full grid-cols-1 justify-items-start gap-4 md:grid-cols-2 lg:w-auto lg:grid-cols-3"
    >
        {#each deployments as [key, { name, description }]}
            <DeploymentTile {name} {description} {key} {orderName} />
        {/each}
    </div>
{/if}
