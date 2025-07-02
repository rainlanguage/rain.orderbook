1. We need to update the wasm macro to accept wasm export on function params for updating types and parameter names.
3. Remove any kind of old logic from the UI repo such as multi subgraph args, etc.
4. Introduce the new routing system with chain id, orderbook address and the order hash.
5. Remove activeSubgraph store dependency.
6. Look at all the store values and remove everything that is not needed.
7. Remove network and subgraph related variables from the pages and internal functions. Everything depends on the raindex client now.
8. Update all the inline documentations to use camel case for the function names.
10. Make sure the wasm macro PR is merged and published. Use the latest version.
11. Remove the old get add order calldata standalone wasm binding.
12. Update balance change query to use page size 1000.
13. Make sure to look at the commented TODOs in the codebase.
14. Look at the transaction manager tests and make sure they are correct.
