<script lang="ts" generics="T">
	import { LightweightChart } from '@rainlanguage/ui-components';
	import type { CreateQueryResult } from '@tanstack/svelte-query';
	import type { IChartApi, UTCTimestamp } from 'lightweight-charts';
	import { sortBy } from 'lodash';

	// eslint-disable-next-line no-undef
	export let query: CreateQueryResult<T[]>;
	// eslint-disable-next-line no-undef
	export let timeTransform: (data: T) => UTCTimestamp;
	// eslint-disable-next-line no-undef
	export let valueTransform: (data: T) => number;

	// eslint-disable-next-line no-undef
	const transformAndSortData = (data: T[]) => {
		const transformedData = data.map((d) => ({
			value: valueTransform(d),
			time: timeTransform(d)
		}));

		return sortBy(transformedData, (d) => d.time);
	};

	$: data = transformAndSortData($query.data ?? []);

	const createSeries = (chart: IChartApi) => chart.addLineSeries({ lineWidth: 1 });
</script>

<LightweightChart {createSeries} {data} loading={$query.isLoading} {...$$props} />
