<script lang="ts">
	export let strategyUrl: string;

	$: console.log(strategyUrl);

	async function getStrategyInfo() {
		const response = await fetch(strategyUrl);
		const res = await response.text();
		console.log(res);
		return res;
	}

	$: dotrain = getStrategyInfo();

	$: console.log(dotrain);
</script>

<div class="aspect-square rounded-lg border border-gray-200 p-4 shadow-sm">
	<slot />
</div>
