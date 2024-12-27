import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/svelte';
import FieldDefinitionDropdown from '../lib/components/deployment/FieldDefinitionDropdown.svelte';
import type { DotrainOrderGui, GuiFieldDefinition } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';

export type FieldDefinitionDropdownProps = ComponentProps<FieldDefinitionDropdown>;

describe('FieldDefinitionDropdown', () => {
	let mockGui: DotrainOrderGui;
	let fieldDefinition: GuiFieldDefinition;

	beforeEach(() => {
		mockGui = {
			saveFieldValue: vi.fn(),
			isFieldPreset: vi.fn()
		} as unknown as DotrainOrderGui;

		fieldDefinition = {
			name: 'Test Field',
			binding: 'test-binding',
			presets: [
				{ id: 'preset1', name: 'Preset 1', value: '' },
				{ id: 'preset2', name: 'Preset 2', value: '' }
			]
		} as unknown as GuiFieldDefinition;
	});

	it('renders field name correctly', () => {
		render(FieldDefinitionDropdown, { props: { fieldDefinition, gui: mockGui } });
		expect(screen.getByText('Test Field')).toBeInTheDocument();
	});

	it('shows presets in dropdown', async () => {
		render(FieldDefinitionDropdown, { props: { fieldDefinition, gui: mockGui } });

		// Open dropdown
		const dropdown = screen.getByText('Select a preset');
		await fireEvent.click(dropdown);

		// Check if presets are rendered
		expect(screen.getByText('Preset 1')).toBeInTheDocument();
		expect(screen.getByText('Preset 2')).toBeInTheDocument();
		expect(screen.getByText('Custom value')).toBeInTheDocument();
	});

	it('calls saveFieldValue when preset is selected', async () => {
		render(FieldDefinitionDropdown, { props: { fieldDefinition, gui: mockGui } });

		// Open dropdown and select preset
		const dropdown = screen.getByText('Select a preset');
		await fireEvent.click(dropdown);
		await fireEvent.click(screen.getByText('Preset 1'));

		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: true,
			value: 'preset1'
		});
	});

	it('shows input field when custom value is selected', async () => {
		(mockGui.isFieldPreset as Mock).mockReturnValue(false);

		render(FieldDefinitionDropdown, { props: { fieldDefinition, gui: mockGui } });

		// Open dropdown and select custom
		const dropdown = screen.getByText('Select a preset');
		await fireEvent.click(dropdown);
		await fireEvent.click(screen.getByText('Custom value'));

		// Check if input field appears
		const input = screen.getByPlaceholderText('Enter value');
		expect(input).toBeInTheDocument();

		// Test input change
		await fireEvent.change(input, { target: { value: 'custom input' } });
		expect(mockGui.saveFieldValue).toHaveBeenCalledWith('test-binding', {
			isPreset: false,
			value: 'custom input'
		});
	});

	it('handles case when gui is not provided', () => {
		render(FieldDefinitionDropdown, { props: { fieldDefinition } as FieldDefinitionDropdownProps });
		expect(screen.getByText('Test Field')).toBeInTheDocument();
		// Should not throw any errors
	});
});
