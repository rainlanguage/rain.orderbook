import { Float, type FuzzResultFlat } from '@rainlanguage/orderbook';
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

export type PlotDataValue = {
  float: Float;
  formatted: string;
  value: number;
};
export type TransformedPlotData = { [key: string]: PlotDataValue };

export const transformDataForPlot = (fuzzResult: FuzzResultFlat): TransformedPlotData[] => {
  if (fuzzResult.data.rows.some((row) => row.length !== fuzzResult.data.columnNames.length)) {
    throw new Error('Number of column names does not match data length');
  }
  return fuzzResult.data.rows.map((row) => {
    const rowObject: TransformedPlotData = {};
    fuzzResult.data.columnNames.forEach((columnName, index) => {
      rowObject[columnName] = decodeRainFloat(row[index] as Hex, columnName);
    });
    return rowObject;
  });
};

const decodeRainFloat = (value: Hex, columnName: string): PlotDataValue => {
  const floatResult = Float.tryFromBigint(hexToBigInt(value));
  if (floatResult?.error || !floatResult?.value) {
    const message = floatResult?.error?.readableMsg ?? floatResult?.error?.msg ?? 'Unknown error';
    throw new Error(`Failed to parse ${columnName} value: ${message}`);
  }

  const formattedResult = floatResult.value.format();
  if (formattedResult.error || !formattedResult.value) {
    const message =
      formattedResult.error?.readableMsg ?? formattedResult.error?.msg ?? 'Unknown error';
    throw new Error(`Failed to format ${columnName} value: ${message}`);
  }

  const numericValue = Number(formattedResult.value);
  if (!Number.isFinite(numericValue)) {
    throw new Error(`Value for ${columnName} is not a finite number: ${formattedResult.value}`);
  }

  return {
    float: floatResult.value,
    formatted: formattedResult.value as string,
    value: numericValue,
  };
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
