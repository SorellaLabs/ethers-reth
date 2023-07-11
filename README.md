![](assets/Octane.png)

# Octane

A ether-rs middleware for reth that bypasses JSON-RPC allowing for faster db queries. From our preliminary benchmarks we have seen a 2x speedup relative to IPC, and 3x speedup relative to local reth Http. See [anvil-benchmarks](https://github.com/SorellaLabs/anvil-benchmarks) for more details.

## Work in Progress!

Please note that Octane is currently in its early stages of development.

## Todo:

- [ ] Full log functionality
- [x] Full parity & geth trace functionality, not currently functional hope to fix this week
- [x] Mock ethers-reth client for github CI testing
- [x] Integration with Anvil: We plan to integrate with Anvil to offer super-fast simulation in fork mode. See [fastfoundry](https://github.com/SorellaLabs/fastfoundry) & [anvil-benchmarks](https://github.com/SorellaLabs/anvil-benchmarks)
- [ ] Full test coverage


### Contact

For any questions or enhancements requests, please open an issue on this repository or dm me on [twitter](https://twitter.com/0xvanbeethoven).
