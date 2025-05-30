# Valence Programs

There are two ways to execute Valence Programs.

1. **On-chain Execution**:
Valence currently supports CosmWasm and EVM. SVM support coming soon. The rest of this section provides a high-level breakdown of the components that comprise a Valence Program using on-chain coprocessors.
    - [Domains](./domains.md)
    - [Accounts](../accounts/_overview.md)
    - [Libraries and Functions](./libraries_and_functions.md)
    - [Programs and Authorizations](./programs_and_authorizations.md)
    - [Middleware](./middleware.md)

2. **Off-chain Execution via ZK Coprocessor**:
Early specifications for the [Valence ZK System](./../zk/_overview.md). We aim to move as much computation off-chain as possible since off-chain computation is a more scalable approach to building a cross-chain execution environment.