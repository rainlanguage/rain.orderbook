# Subgraph

All the commands assume that you are on `subgraph` folder.

## Tests

1 - Install forge deps in root:

```bash
forge install --root ../ --shallow
```

2 - Install the subgraph submodules:

```bash
git submodule update --init --recursive --depth 1 rain.subgraph.docker/
```

3 - Start the docker container

```bash
nix run .#docker-up
```

4 - Obtain the meta files

```bash
(cd .. && nix run .#build-meta)
```

5 - Init the setup:

```bash
nix run .#init-setup 
```

6  - Get the json schemas

```bash
nix run .#generate-sg-schema
```

7 - Run the tests

```bash
nix run .#ci-test
```
