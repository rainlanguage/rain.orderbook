<script lang="ts">
	import { orderbook } from '$lib';
	import type { get } from 'svelte/store';
	import { tokens } from './tokens';
	import { account } from 'svelte-wagmi-stores';
	import { hexToBigInt } from 'viem';

	let vaultId: string = '';

	type UnwrapStore<T> = T extends { subscribe: (sub: (val: infer R) => void) => any } ? R : never;
	type CountValueActualType = UnwrapStore<typeof orderbook>;

	$: orderConfig = vaultId
		? {
				validInputs: tokens.map((token) => ({
					vaultId: hexToBigInt(vaultId as `0x${string}`),
					decimals: token.decimals,
					token: token.address as `0x${string}`
				})),
				validOutputs: tokens.map((token) => ({
					vaultId: hexToBigInt(vaultId as `0x${string}`),
					decimals: token.decimals,
					token: token.address as `0x${string}`
				})),
				evaluableConfig: {
					deployer: '0xb20dfedc1b12aa6afa308064998a28531a18c714' as `0x${string}`,
					constants: [
						BigInt('50717328057819670919621266506729799030641233149'),
						BigInt('1'),
						BigInt('59180358874249724264754244400400157205534792422'),
						BigInt('0'),
						BigInt('84600'),
						BigInt('1000000000000000000000')
					],
					sources: [
						'0x000c00010004050000280000001700000004060000040601000406020004060300040604000406050004060600050006000c000e00470000002b000000170000000c000e000c000300480000000c00060004010000280000000c000a0004000100280000000c000c001a0000002a0000000c0000000c00020004010200280000000c00020004000000280000000c0005000c000400280000000c00050004040000280000000c000800040402002800000029000900170000000c0000000c0007' as `0x${string}`,
						'0x00040600000c0000000404040028000000170000001a0000000c0009001c0002000c000200040100000d0002000c000400470000000c000600040404001b0002000c0008000c000b002a0000002b000000170000000c0004000c000800480000' as `0x${string}`
					]
				},
				meta: '0x' as `0x${string}`
		  }
		: null;

	const addOrder = () => {
		if (!$orderbook || !orderConfig) return;
		$orderbook.write.addOrder([orderConfig]);
	};
</script>

{#if $account?.isConnected}
	<span>vault id</span>
	<input bind:value={vaultId} />
	<button on:click={addOrder}>Add Order</button>
	<hr />
	<pre>
        {JSON.stringify(orderConfig, null, 2)}
    </pre>
{/if}
