<script lang="ts" generics="T">
	import { ButtonLoading, IconError } from '@rainlanguage/ui-components';
	import {
		IconInfo,
		IconLedger,
		IconSuccess,
		IconTelegram,
		IconWalletConnect,
		IconWarning,
		IconExternalLink,
		MetricChart
	} from '@rainlanguage/ui-components';
	let isLoading: boolean = true;
	// Mock data for `metric` prop
	const metric = {
		label: 'Test Metric',
		description: 'This is a test metric.',
		'unit-prefix': '$',
		'unit-suffix': ' USD',
		value: 'testValue',
		precision: 2
	};

	// Mock data for `data` prop
	const data = [
		{
			testValue: 1234.5678
		}
	];
</script>

<div>
	<ButtonLoading color="primary" class="w-full px-2 py-1" size="lg" pill loading={isLoading}
		>Loading button</ButtonLoading
	>
	<IconError />
	<IconExternalLink svgClass="!h-2 !w-2 mr-1" />
	<IconInfo />
	<IconLedger />
	<IconSuccess />
	<IconTelegram dark />
	<IconWalletConnect />
	<IconWarning />
	<MetricChart {metric} {data} />
</div>
