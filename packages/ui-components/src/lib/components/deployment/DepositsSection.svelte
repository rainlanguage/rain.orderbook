<script lang="ts">
	import DepositInput from './DepositInput.svelte';
	import type { DotrainOrderGui, GuiDeposit } from '@rainlanguage/orderbook/js_api';

	export let allDepositFields: GuiDeposit[];
	export let gui: DotrainOrderGui;
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;
</script>

{#each allDepositFields as deposit}
	<DepositInput bind:deposit {gui} {handleUpdateGuiState} />
{/each}
