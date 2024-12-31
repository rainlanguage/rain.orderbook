import { render, screen, waitFor } from '@testing-library/svelte';
import { test, expect } from 'vitest';
import EvalResultsTable from './EvalResultsTable.svelte';
import { formatEther, hexToBigInt, isHex } from 'viem';

test('renders table with the correct data', async () => {
	const table = {
		column_names: ['Column1', 'Column2', 'Column3'],
		rows: [
			['0x01', '0x02', '0x03'],
			['0x0a', '0x0b', '0x0c'],
			['0x1f', '0x2e', '0x3d'],
			['0xaa', '0xbb', '0xcc'],
			['0xff', '0xee', '0xdd']
		]
	};

	render(EvalResultsTable, { table });

	// Check if the table headers are rendered correctly
	expect(screen.getByText('Stack item')).toBeInTheDocument();
	expect(screen.getByText('Value')).toBeInTheDocument();
	expect(screen.getByText('Hex')).toBeInTheDocument();

	// Check if the table rows are rendered correctly
	await waitFor(() => {
		table.column_names.forEach((columnName, index) => {
			expect(screen.getByText(columnName)).toBeInTheDocument();
			expect(screen.getByText(table.rows[0][index])).toBeInTheDocument();

			// Verify if the formatted value is correct
			const value = table.rows[0][index];
			if (value) {
				const formattedValue = isHex(value) ? formatEther(hexToBigInt(value)) : '';
				expect(screen.getByText(formattedValue)).toBeInTheDocument();
			}
		});
	});
});
