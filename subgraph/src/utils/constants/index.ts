// Orderbook: TakeOrder(address sender, TakeOrderConfig config, uint256 input, uint256 output)
export let TAKE_ORDER_EVENT_TOPIC =
  "0x219a030b7ae56e7bea2baab709a4a45dc174a1f85e57730e5cb395bc32962542";

// Orderbook: Clear(address sender, Order alice, Order bob, ClearConfig clearConfig)
export let CLEAR_EVENT_TOPIC =
  "0xd153812deb929a6e4378f6f8cf61d010470840bf2e736f43fb2275803958bfa2";

// Orderbook: AfterClear(address sender, ClearStateChange clearStateChange);
export let AFTER_CLEAR_EVENT_TOPIC =
  "0x3f20e55919cca701abb2a40ab72542b25ea7eed63a50f979dd2cd3231e5f488d";

// ExpressionDeployer: NewExpression(address,bytes,uint256[],uint256[])
export let NEW_EXPRESSION_EVENT_TOPIC =
  "0x4a48f556905d90b4a58742999556994182322843167010b59bf8149724db51cf";
