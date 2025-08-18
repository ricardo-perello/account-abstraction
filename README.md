# aa-learning (skeleton)

Repo layout:

- contracts/: Foundry workspace
  - src/
    - SmartAccount.sol (abstract)
    - interfaces/
      - IEntryPoint.sol
      - UserOperation.sol
  - script/
    - Deploy.s.sol (placeholder)
  - test/
    - (add tests later)
  - foundry.toml
- client/: Rust (Alloy) tester
  - Cargo.toml
  - src/
    - main.rs
    - userop.rs
    - abi/SmartAccount.json (copy from contracts/out later)
- .env.example
- docs/
  - flow-notes.md
