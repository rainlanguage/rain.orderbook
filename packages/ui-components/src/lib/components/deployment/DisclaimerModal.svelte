<script lang="ts">
	import { Alert, Modal, Button } from 'flowbite-svelte';
	import { ExclamationCircleSolid } from 'flowbite-svelte-icons';
	export let open: boolean = false;
	export let onAccept: () => void = () => {};

	async function handleAcceptDisclaimer() {
		open = false;
		onAccept();
	}
</script>

<Modal
	bind:open
	class="max-h-full dark:border dark:border-gray-700 dark:bg-gray-900"
	dialogClass="fixed top-0 start-0 end-1 h-modal md:inset-0 md:h-full z-50 w-full p-4 flex justify-center items-center h-full"
	data-testid="deployment-disclaimer-modal"
>
	<div class="flex flex-col items-start gap-y-8 p-4">
		<div class="space-y-4">
			<Alert color="red" class="text-base">
				<div class="flex items-center justify-center">
					<ExclamationCircleSolid class="h-6 w-6 text-red-500" />
					<span class="ml-2">
						Before you deploy your strategy, make sure you understand the following...
					</span>
				</div>
			</Alert>
			<ul class="list-outside list-disc space-y-2">
				<li class="ml-4">
					This front end is provided as a tool to interact with the Raindex smart contracts.
				</li>
				<li class="ml-4">
					You are deploying your own strategy and depositing funds to an immutable smart contract
					using your own wallet and private keys.
				</li>
				<li class="ml-4">
					Nobody is custodying your funds, there is no recourse for recovery of funds if lost.
				</li>
				<li class="ml-4">There is no endorsement or guarantee provided with these strategies.</li>
				<li class="ml-4">
					Do not proceed if you do not understand the strategy you are deploying.
				</li>
				<li class="ml-4">Do not invest unless you are prepared to lose all funds.</li>
			</ul>
		</div>
		<div class="flex justify-center gap-2">
			<Button
				size="lg"
				class="w-32 bg-gradient-to-br from-blue-600 to-violet-600"
				on:click={handleAcceptDisclaimer}>Deploy</Button
			>
			<Button size="lg" class="w-32" color="light" on:click={() => (open = false)}>Cancel</Button>
		</div>
	</div>
</Modal>
