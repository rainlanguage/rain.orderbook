import type { FuzzResultFlat } from '@rainlanguage/orderbook';
import { hexToBigInt, type Hex, formatUnits } from 'viem';

export type TransformedHexAndFormattedData = { [key: string]: [number, Hex] };

// Transform the data from the backend to the format required by the plot library
export const transformData = (fuzzResult: FuzzResultFlat): TransformedHexAndFormattedData[] => {
  if (fuzzResult.data.rows.some((row) => row.length !== fuzzResult.data.columnNames.length)) {
    throw new Error('Number of column names does not match data length');
  }
  return fuzzResult.data.rows.map((row) => {
    const rowObject: TransformedHexAndFormattedData = {};
    fuzzResult.data.columnNames.forEach((columnName, index) => {
      rowObject[columnName] = [+formatUnits(hexToBigInt(row[index] as Hex), 18), row[index] as Hex];
    });
    return rowObject;
  });
};

export type TransformedPlotData = { [key: string]: number };

export const transformDataForPlot = (fuzzResult: FuzzResultFlat): TransformedPlotData[] => {
  if (fuzzResult.data.rows.some((row) => row.length !== fuzzResult.data.columnNames.length)) {
    throw new Error('Number of column names does not match data length');
  }
  return fuzzResult.data.rows.map((row) => {
    const rowObject: TransformedPlotData = {};
    fuzzResult.data.columnNames.forEach((columnName, index) => {
      rowObject[columnName] = +formatUnits(hexToBigInt(row[index] as Hex), 18);
    });
    return rowObject;
  });
};

if (import.meta.vitest) {
  const { it, expect } = import.meta.vitest;

  it('data transforms correctly and errors are caught', () => {
    const fuzzResult = {
      data: {
        block_number: '0x1234',
        rows: [
          ['0xDE0B6B3A7640000', '0x29A2241AF62C0000'],
          ['0x1BC16D674EC80000', '0x3782DACE9D900000'],
          ['0x29A2241AF62C0000', '0x5678'],
          ['0x1234', '0x5678'],
        ],
        columnNames: ['col1', 'col2'],
      },
      scenario: 'test',
    };

    const transformedData = transformData(fuzzResult);

    expect(transformedData.length).toEqual(4);
    expect(transformedData[0].col1[0]).toEqual(1);
    expect(transformedData[0].col2[0]).toEqual(3);
    expect(transformedData[1].col1[0]).toEqual(2);
    expect(transformedData[1].col2[0]).toEqual(4);

    expect(transformedData[0].col1[1]).toEqual('0xDE0B6B3A7640000');
    expect(transformedData[0].col2[1]).toEqual('0x29A2241AF62C0000');
    expect(transformedData[1].col1[1]).toEqual('0x1BC16D674EC80000');
    expect(transformedData[1].col2[1]).toEqual('0x3782DACE9D900000');

    const fuzzResult3 = {
      data: {
        block_number: '0x1234',
        rows: [
          ['0x1234', '0x5678'],
          ['0x1234', '0x5678'],
          ['0x1234', '0x5678'],
          ['0x1234', '0x5678'],
        ],
        columnNames: ['col1'],
      },
      scenario: 'test',
    };

    expect(() => transformData(fuzzResult3)).toThrowError(
      'Number of column names does not match data length',
    );
  });
}
