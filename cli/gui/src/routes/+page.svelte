<script lang="ts">
	import { BinaryInterface } from '$lib/binary-interface';
	import type { DepositConfig } from '$lib/binary-interface/types';
	import Logs from '$lib/components/Logs.svelte';
	import { Button } from 'flowbite-svelte';

	let binInt = new BinaryInterface();

	const myDeposit: DepositConfig = {
		orderbook: 'someOrderbookAddress',
		tokenAddress: 'someTokenAddress',
		tokenDecimals: 18,
		amount: 'someAmount',
		rpcUrl: 'someRpcUrl'
		// ... other fields as necessary
	};
</script>

<Logs />

<Button
	on:click={() => {
		binInt.deposit(myDeposit);
	}}>Click me</Button
>
