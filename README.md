# ejit-evm

Ejit EVM is work in progress to implement the ethereum EVM and node as a JIT
compiler. Contracts will be transcoded on the fly to run using machine instructions.

This EVM is characterised by having almost no dependencies, making it possile
to run it in a variety of non-standard environments on on bare metal platforms.

As yet, we have no database and are instead just using a `BTreeMap` as a placeholder
for a KV store.

This implementation is a close copy of the Python etherereum specification which
makes it easy to maintain.

Some elements, like the state trie will need some improvement to remain competitive
with the JIT execution engine.

This project has just started. Drop me a line if you are interested in generating
an Ethereum node from scratch!
