<script lang="ts">
	import { onMount } from 'svelte';

	let messages: string[] = [];
	let socket: WebSocket;

	onMount(() => {
		socket = new WebSocket('ws://localhost:8080/ws/');

		socket.onmessage = (event: MessageEvent) => {
			console.log('event received');
			messages = [...messages, event.data.toString()];
		};

		// return () => {
		// 	socket.close();
		// };
	});

	$: console.log(messages);
</script>

<div class="terminal">
	{#each messages as message}
		<div class="mb-2">{message}</div>
	{/each}
</div>

<style>
	.terminal {
		background-color: #222;
		color: #00ff00;
		font-family: 'Courier New', Courier, monospace;
		padding: 16px;
		border-radius: 8px;
		max-height: 400px;
		overflow-y: scroll;
	}
</style>
