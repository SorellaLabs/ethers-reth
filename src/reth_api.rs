

// trait for reth EthApi

pub type RethClient = BlockchainProvider<
    Arc<Env<NoWriteMap>>,
    ShareableBlockchainTree<Arc<Env<NoWriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

#[async_trait]
pub trait RethEthApi {


}


impl<Client, Pool, Network> RethEthApi for EthApi<Client, Pool, Network> {

    fn default(self);

}
