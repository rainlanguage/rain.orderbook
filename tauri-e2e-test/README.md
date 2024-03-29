# Tauri End-to-End Tests

End-to-end tests currently only support linux.

## Run E2E Tests Headless
1. Build tauri app
2. `nix develop .#tauri-shell --command ob-tauri-e2e-test-headless`

## Run E2E Tests Visibly
1. Build tauri app
2. `nix develop .#tauri-shell --command ob-tauri-e2e-test`

## Writing Tests

Tests can be added to `tauri-e2e-test/tests`. There are a few conventions to follow:

### DOM Selectors

Ideally, DOM selectors should reflect the *role* the element plays within the page. 

Alternatively, if the element does not have a role that is clearly identifiable via selector, then the attribute `data-testid` should be added to the element to use in a selector. i.e.

```html
<button data-testid="button-close">Close</button>
```

If you are trying to select a 3rd party Svelte component, and thus are unable to set an attribute directly on the html tag it renders, then create an empty element either *within* the 3rd party Svelte component, or *wrapping* it. i.e.

```html
<DropdownComponent>
  My Dropdown
  <span data-testid="dropdown-label"></span>
</DropdownComponent
```

See [https://webdriver.io/docs/selectors/](WebdriverIO documentation) for the reasoning behind this.

### Waiting for UI state

A common challange of webdriver testing is *flaky failures.* This typically occurs because the UI will take some unpredictable amount of time to load & render a page, so your tests fail before they should even be making assertions.

Ideally, tests should never manually *sleep* for an arbitrary time, and instead always *wait* for some UI state: i.e. polling for that state until a longer timeout. This is implemented in the thirtyfour webdriver client via [ElementQuery](https://docs.rs/thirtyfour/latest/thirtyfour/extensions/query/struct.ElementQuery.html). Always use ElementQuery if possible.
