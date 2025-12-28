pub struct ContractConfig {
    pub exchange: String,
    pub collateral: String,
    pub conditional_tokens: String,
}

pub fn get_contract_config(chain_id: u64, neg_risk: bool) -> Option<ContractConfig> {
    match neg_risk {
        true => {
            if chain_id == 137 {
                return Some(ContractConfig {
                    exchange: "0xC5d563A36AE78145C45a50134d48A1215220f80a".to_owned(),
                    collateral: "0x2791bca1f2de4661ed88a30c99a7a9449aa84174".to_owned(),
                    conditional_tokens: "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045".to_owned(),
                });
            } else if chain_id == 80002 {
                return Some(ContractConfig {
                    exchange: "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296".to_owned(),
                    collateral: "0x9c4e1703476e875070ee25b56a58b008cfb8fa78".to_owned(),
                    conditional_tokens: "0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB".to_owned(),
                });
            }
            None
        }
        false => {
            if chain_id == 137 {
                return Some(ContractConfig {
                    exchange: "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E".to_owned(),
                    collateral: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_owned(),
                    conditional_tokens: "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045".to_owned(),
                });
            } else if chain_id == 80002 {
                return Some(ContractConfig {
                    exchange: "0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40".to_owned(),
                    collateral: "0x9c4e1703476e875070ee25b56a58b008cfb8fa78".to_owned(),
                    conditional_tokens: "0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB".to_owned(),
                });
            }
            None
        }
    }
}
