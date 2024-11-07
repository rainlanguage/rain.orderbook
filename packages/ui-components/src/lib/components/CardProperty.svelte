<div {...$$props}>
	<h5
		data-testid="card-property-key"
		class="text-md mb-1 w-full tracking-tight text-gray-400 dark:text-gray-400"
	>
		<slot name="key" />
	</h5>
	<p class="font-regular break-all leading-tight text-gray-800 dark:text-white">
		<slot name="value" />
	</p>
</div>
