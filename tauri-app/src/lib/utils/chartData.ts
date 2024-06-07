import type { FuzzResultFlat } from '$lib/typeshare/config';
import { hexToBigInt, type Hex, formatUnits } from 'viem';

export type TransformedHexAndFormattedData = { [key: string]: [number, Hex] };

// Transform the data from the backend to the format required by the plot library
export const transformData = (fuzzResult: FuzzResultFlat): TransformedHexAndFormattedData[] => {
  if (fuzzResult.data.some((row) => row.length !== fuzzResult.column_names.length)) {
    throw new Error('Number of column names does not match data length');
  }
  return fuzzResult.data.map((row) => {
    const rowObject: TransformedHexAndFormattedData = {};

    fuzzResult.column_names.forEach((columnName, index) => {
      if (columnName == 'block') {
        rowObject[columnName] = row[0];
      } else {
        rowObject[columnName] = [+formatUnits(hexToBigInt(row[index] as Hex), 18), row[index] as Hex];
      }
    });
    return rowObject;
  });
};

export type TransformedPlotData = { [key: string]: number };

export const transformDataForPlot = (fuzzResult: FuzzResultFlat): TransformedPlotData[] => {
  if (fuzzResult.data.some((row) => row.length !== fuzzResult.column_names.length)) {
    throw new Error('Number of column names does not match data length');
  }
  return fuzzResult.data.map((row) => {
    const rowObject: TransformedPlotData = {};

    fuzzResult.column_names.forEach((columnName, index) => {
      if (columnName != 'block') {
        rowObject[columnName] = +formatUnits(hexToBigInt(row[index] as Hex), 18);
      }
    });
    return rowObject;
  });
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('data transforms correctly and errors are caught', () => {
    const fuzzResultBlocks = {
      data: [
        ['14334', '0xDE0B6B3A7640000', '0x29A2241AF62C0000'],
        ['14334', '0x1BC16D674EC80000', '0x3782DACE9D900000'],
        ['14335', '0x29A2241AF62C0000', '0x5678'],
        ['14335', '0x1234', '0x5678'],
      ],
      column_names: ['block', 'col1', 'col2'],
      scenario: 'test',
    };

    const transformedBlockData = transformData(fuzzResult);

    expect(transformedBlockData.length).toEqual(4);
    expect(transformedBlockData[0].block).toEqual(14334);
    expect(transformedBlockData[1].block).toEqual(14334);
    expect(transformedBlockData[2].block).toEqual(14335);
    expect(transformedBlockData[3].block).toEqual(14335);

    expect(transformedBlockData[0].col1[0]).toEqual(1);
    expect(transformedBlockData[0].col2[0]).toEqual(3);
    expect(transformedBlockData[1].col1[0]).toEqual(2);
    expect(transformedBlockData[1].col2[0]).toEqual(4);

    expect(transformedBlockData[0].col1[1]).toEqual('0xDE0B6B3A7640000');
    expect(transformedBlockData[0].col2[1]).toEqual('0x29A2241AF62C0000');
    expect(transformedBlockData[1].col1[1]).toEqual('0x1BC16D674EC80000');
    expect(transformedBlockData[1].col2[1]).toEqual('0x3782DACE9D900000');

    const fuzzResult3 = {
      data: [
        ['0x1234', '0x5678'],
        ['0x1234', '0x5678'],
        ['0x1234', '0x5678'],
        ['0x1234', '0x5678'],
      ],
      column_names: ['col1'],
      scenario: 'test',
    };

    expect(() => transformData(fuzzResult3)).toThrowError(
      'Number of column names does not match data length',
    );
  });
}
