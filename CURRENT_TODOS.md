3. Remove any kind of old logic from the UI repo such as multi subgraph args, etc.
6. Look at all the store values and remove everything that is not needed.
7. Remove network and subgraph related variables from the pages and internal functions. Everything depends on the raindex client now.
10. Make sure the wasm macro PR is merged and published. Use the latest version.
13. Make sure to look at the commented TODOs in the codebase.
