import type { FuzzResultFlat } from "$lib/typeshare/config";
import { hexToBigInt, type Hex, formatUnits } from "viem";

export type TransformedPlotData = { [key: string]: number };

// Transform the data from the backend to the format required by the plot library
export const transformData = (fuzzResult: FuzzResultFlat): TransformedPlotData[] => {
    if (fuzzResult.data.some((row) => row.length !== fuzzResult.column_names.length)) {
        throw new Error('Number of column names does not match data length');
    }
    return fuzzResult.data.map((row) => {
        const rowObject: TransformedPlotData = {};
        fuzzResult.column_names.forEach((columnName, index) => {
            rowObject[columnName] = +formatUnits(hexToBigInt(row[index] as Hex), 18);
        });
        return rowObject;
    });
};

if (import.meta.vitest) {
    const { it, expect } = import.meta.vitest

    it('data transforms correctly and errors are caught', () => {

        const fuzzResult = {
            data: [
                ['0xDE0B6B3A7640000', '0x29A2241AF62C0000'],
                ['0x1BC16D674EC80000', '0x3782DACE9D900000'],
                ['0x29A2241AF62C0000', '0x5678'],
                ['0x1234', '0x5678'],
            ],
            column_names: ['col1', 'col2'],
            scenario: 'test'
        }

        const transformedData = transformData(fuzzResult);

        expect(transformedData.length).toEqual(4);
        expect(transformedData[0].col1).toEqual(1);
        expect(transformedData[0].col2).toEqual(3);
        expect(transformedData[1].col1).toEqual(2);
        expect(transformedData[1].col2).toEqual(4);

        const fuzzResult3 = {
            data: [
                ['0x1234', '0x5678'],
                ['0x1234', '0x5678'],
                ['0x1234', '0x5678'],
                ['0x1234', '0x5678'],
            ],
            column_names: ['col1'],
            scenario: 'test'
        }

        expect(() => transformData(fuzzResult3)).toThrowError('Number of column names does not match data length');
    });
}