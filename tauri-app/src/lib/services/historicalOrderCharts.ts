import type { TakeOrderEntity } from "$lib/typeshare/orderTakesList";
import type { UTCTimestamp } from "lightweight-charts";
import { timestampSecondsToUTCTimestamp } from '$lib/utils/time';
import { sortBy } from 'lodash';


export type HistoricalOrderChartData = { value: number; time: UTCTimestamp; color?: string }[];

export function prepareHistoricalOrderChartData(takeOrderEntities: TakeOrderEntity[], colorTheme: string) {
    const transformedData = takeOrderEntities.map((d) => ({
        value: parseFloat(d.ioratio),
        time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
        color: colorTheme == 'dark' ? '#5178FF' : '#4E4AF6',
        outputAmount: +d.output_display,
    }));

    // if we have multiple object in the array with the same timestamp, we need to merge them
    // we do this by taking the weighted average of the ioratio values for objects that share the same timestamp.
    const uniqueTimestamps = Array.from(new Set(transformedData.map((d) => d.time)));
    const finalData: HistoricalOrderChartData = [];
    uniqueTimestamps.forEach((timestamp) => {
        const objectsWithSameTimestamp = transformedData.filter((d) => d.time === timestamp);
        if (objectsWithSameTimestamp.length > 1) {
            // calculate a weighted average of the ioratio values using the amount of the output token as the weight
            const ioratioSum = objectsWithSameTimestamp.reduce(
                (acc, d) => acc + d.value * d.outputAmount,
                0,
            );
            const outputAmountSum = objectsWithSameTimestamp.reduce(
                (acc, d) => acc + d.outputAmount,
                0,
            );
            const ioratioAverage = ioratioSum / outputAmountSum;
            finalData.push({
                value: ioratioAverage,
                time: timestamp,
                color: objectsWithSameTimestamp[0].color,
            });
        } else {
            finalData.push(objectsWithSameTimestamp[0]);
        }
    });

    return sortBy(finalData, (d) => d.time);
}

if (import.meta.vitest) {
    const { it, expect } = import.meta.vitest

    it('transforms and sorts data as expected', () => {

        const takeOrderEntities: TakeOrderEntity[] = [
            {
                id: '1',
                transaction: { id: '1' },
                sender: { id: '1' },
                timestamp: '1632000000',
                order: { id: '1' },
                ioratio: '0.1',
                input: '1',
                input_display: '1',
                input_token: { id: '1', name: '1', symbol: '1', decimals: 1 },
                input_ioindex: '1',
                output: '1',
                output_display: '1',
                output_token: { id: '1', name: '1', symbol: '1', decimals: 1 },
                output_ioindex: '1',
            },
            {
                id: '2',
                transaction: { id: '2' },
                sender: { id: '2' },
                timestamp: '1630000000',
                order: { id: '2' },
                ioratio: '0.2',
                input: '2',
                input_display: '2',
                input_token: { id: '2', name: '2', symbol: '2', decimals: 2 },
                input_ioindex: '2',
                output: '2',
                output_display: '2',
                output_token: { id: '2', name: '2', symbol: '2', decimals: 2 },
                output_ioindex: '2',
            },
            {
                id: '3',
                transaction: { id: '3' },
                sender: { id: '3' },
                timestamp: '1631000000',
                order: { id: '3' },
                ioratio: '0.3',
                input: '3',
                input_display: '3',
                input_token: { id: '3', name: '3', symbol: '3', decimals: 3 },
                input_ioindex: '3',
                output: '3',
                output_display: '3',
                output_token: { id: '3', name: '3', symbol: '3', decimals: 3 },
                output_ioindex: '3',
            }];

        const result = prepareHistoricalOrderChartData(takeOrderEntities, 'dark');

        expect(result.length).toEqual(3);
        expect(result[0].value).toEqual(0.2);
        expect(result[0].time).toEqual(1630000000);
        expect(result[1].value).toEqual(0.3);
        expect(result[1].time).toEqual(1631000000);
        expect(result[2].value).toEqual(0.1);
        expect(result[2].time).toEqual(1632000000);

        // check the color
        expect(result[0].color).toEqual('#5178FF');
        expect(result[1].color).toEqual('#5178FF');
        expect(result[2].color).toEqual('#5178FF');
    });

    it("handles the case where multiple takeOrderEntities have the same timestamp", () => {

        const takeOrderEntities: TakeOrderEntity[] = [
            {
                id: '1',
                transaction: { id: '1' },
                sender: { id: '1' },
                timestamp: '1630000000',
                order: { id: '1' },
                ioratio: '0.1',
                input: '1',
                input_display: '1',
                input_token: { id: '1', name: '1', symbol: '1', decimals: 1 },
                input_ioindex: '1',
                output: '1',
                output_display: '1',
                output_token: { id: '1', name: '1', symbol: '1', decimals: 1 },
                output_ioindex: '1',
            },
            {
                id: '2',
                transaction: { id: '2' },
                sender: { id: '2' },
                timestamp: '1630000000',
                order: { id: '2' },
                ioratio: '0.2',
                input: '2',
                input_display: '2',
                input_token: { id: '2', name: '2', symbol: '2', decimals: 2 },
                input_ioindex: '2',
                output: '2',
                output_display: '2',
                output_token: { id: '2', name: '2', symbol: '2', decimals: 2 },
                output_ioindex: '2',
            },
            {
                id: '3',
                transaction: { id: '3' },
                sender: { id: '3' },
                timestamp: '1630000000',
                order: { id: '3' },
                ioratio: '0.3',
                input: '3',
                input_display: '3',
                input_token: { id: '3', name: '3', symbol: '3', decimals: 3 },
                input_ioindex: '3',
                output: '3',
                output_display: '3',
                output_token: { id: '3', name: '3', symbol: '3', decimals: 3 },
                output_ioindex: '3',
            }];

        const result = prepareHistoricalOrderChartData(takeOrderEntities, 'dark');

        // calculate the weighted average of the ioratio values
        const ioratioSum = 0.1 * 1 + 0.2 * 2 + 0.3 * 3;
        const outputAmountSum = 1 + 2 + 3;
        const ioratioAverage = ioratioSum / outputAmountSum;

        expect(result.length).toEqual(1);
        expect(result[0].value).toEqual(ioratioAverage);
    });
}