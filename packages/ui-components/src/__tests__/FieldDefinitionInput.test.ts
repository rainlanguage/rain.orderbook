import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import FieldDefinitionInput from '../lib/components/deployment/FieldDefinitionInput.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: vi.fn().mockImplementation(() => ({
		saveFieldValue: vi.fn(),
		getFieldValue: vi.fn(),
		isFieldPreset: vi.fn(),
		getAllFieldValues: vi.fn(),
		getCurrentDeployment: vi.fn()
	}))
}));

describe('FieldDefinitionInput', () => {
	let mockGui: DotrainOrderGui;
	const mockFieldDefinition = {
		binding: 'test-binding',
		name: 'Test Field',
		description: 'Test Description',
		presets: [
			{ id: 'preset1', name: 'Preset 1', value: 'value1' },
			{ id: 'preset2', name: 'Preset 2', value: 'value2' }
		]
	};

	beforeEach(() => {
		mockGui = new DotrainOrderGui();
	});

	it('renders field name and description', () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui,
				open: true
			}
		});

		expect(getByText('Test Field')).toBeTruthy();
		expect(getByText('Test Description')).toBeTruthy();
	});

	it('renders preset buttons', () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui,
				open: true
			}
		});

		expect(getByText('Preset 1')).toBeTruthy();
		expect(getByText('Preset 2')).toBeTruthy();
	});

	it('handles preset button clicks', async () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui,
				open: true
			}
		});

		await fireEvent.click(getByText('Preset 1'));

		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: true,
			value: 'preset1'
		});
	});

	it('handles custom input changes', async () => {
		const { getByPlaceholderText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui,
				open: true
			}
		});

		const input = getByPlaceholderText('Enter custom value');
		await fireEvent.input(input, { target: { value: 'custom value' } });

		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: false,
			value: 'custom value'

		});
	});

	it('does not show Custom button for is-fast-exit binding', () => {
		const fastExitFieldDef = {
			...mockFieldDefinition,
			binding: 'is-fast-exit'
		};

		const { queryByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: fastExitFieldDef,
				gui: mockGui,
				open: true
			}
		});

		expect(queryByText('Custom')).toBeNull();
	});
});
