import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import FieldDefinitionButtons from '../lib/components/deployment/wizard/FieldDefinitionButtons.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

// Mock the DotrainOrderGui class
vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: vi.fn().mockImplementation(() => ({
		saveFieldValue: vi.fn(),
		getFieldValue: vi.fn(),
		isFieldPreset: vi.fn()
	}))
}));

describe('FieldDefinitionButtons', () => {
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
		const { getByText } = render(FieldDefinitionButtons, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui
			}
		});

		expect(getByText('Test Field')).toBeTruthy();
		expect(getByText('Test Description')).toBeTruthy();
	});

	it('renders preset buttons', () => {
		const { getByText } = render(FieldDefinitionButtons, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui
			}
		});

		expect(getByText('Preset 1')).toBeTruthy();
		expect(getByText('Preset 2')).toBeTruthy();
		expect(getByText('Custom')).toBeTruthy();
	});

	it('handles preset button clicks', async () => {
		const { getByText } = render(FieldDefinitionButtons, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui
			}
		});

		await fireEvent.click(getByText('Preset 1'));

		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: true,
			value: 'preset1'
		});
	});

	it('shows custom input when Custom button is clicked', async () => {
		const { getByText, getByPlaceholderText } = render(FieldDefinitionButtons, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui
			}
		});

		await fireEvent.click(getByText('Custom'));
		expect(getByPlaceholderText('Enter custom value')).toBeTruthy();
	});

	it('handles custom input changes', async () => {
		const { getByText, getByPlaceholderText } = render(FieldDefinitionButtons, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui
			}
		});

		await fireEvent.click(getByText('Custom'));
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

		const { queryByText } = render(FieldDefinitionButtons, {
			props: {
				fieldDefinition: fastExitFieldDef,
				gui: mockGui
			}
		});

		expect(queryByText('Custom')).toBeNull();
	});
});
