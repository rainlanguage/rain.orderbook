import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import FieldDefinitionInput from '../lib/components/deployment/FieldDefinitionInput.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import userEvent from '@testing-library/user-event';

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
	let mockStateUpdateCallback: Mock;

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
		mockStateUpdateCallback = vi.fn();
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		mockGui = new (DotrainOrderGui as any)();
		mockGui.saveFieldValue = vi.fn().mockImplementation(() => {
			mockStateUpdateCallback();
		});
	});

	it('renders field name and description', () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui
			}
		});

		expect(getByText('Test Field')).toBeTruthy();
		expect(getByText('Test Description')).toBeTruthy();
	});

	it('renders preset buttons', () => {
		const { getByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: mockFieldDefinition,
				gui: mockGui
			}
		});

		expect(getByText('Preset 1')).toBeTruthy();
		expect(getByText('Preset 2')).toBeTruthy();
	});

	it('handles preset button clicks and triggers state update', async () => {
		const { getByText } = render(FieldDefinitionInput, {
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
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});

	it('handles custom input changes and triggers state update', async () => {
		const { getByPlaceholderText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: { ...mockFieldDefinition, showCustomField: true },
				gui: mockGui
			}
		});

		const input = getByPlaceholderText('Enter custom value');
		await fireEvent.input(input, { target: { value: 'custom value' } });

		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: false,
			value: 'custom value'
		});
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});

	it('does not show Custom button for is-fast-exit binding', () => {
		const fastExitFieldDef = {
			...mockFieldDefinition,
			binding: 'is-fast-exit'
		};

		const { queryByText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: fastExitFieldDef,
				gui: mockGui
			}
		});

		expect(queryByText('Custom')).toBeNull();
	});

	it('renders default value if it exists', async () => {
		const { getByPlaceholderText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: {
					...mockFieldDefinition,
					default: 'default value',
					showCustomField: true
				},
				gui: mockGui
			}
		});

		const input = getByPlaceholderText('Enter custom value') as HTMLInputElement;
		expect(input.value).toBe('default value');

		await userEvent.type(input, '@');

		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: false,
			value: 'default value@'
		});
	});
	it('renders selected value instead of default value', async () => {
		(mockGui.getFieldValue as Mock).mockReturnValue({
			isPreset: false,
			value: 'preset1'
		});

		const { getByPlaceholderText } = render(FieldDefinitionInput, {
			props: {
				fieldDefinition: {
					...mockFieldDefinition,
					default: 'default value',
					showCustomField: true
				},
				gui: mockGui
			}
		});

		const input = getByPlaceholderText('Enter custom value') as HTMLInputElement;
		expect(input.value).toBe('preset1');

		await userEvent.type(input, '@');

		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: false,
			value: 'preset1@'
		});
	});
});
