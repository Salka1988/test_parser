use std::fmt::Formatter;

use fuel_core_chain_config::{ContractConfig, MessageConfig};
use fuel_core_types::fuel_tx::{Address, AssetId, Bytes32, ConsensusParameters};
use fuel_core_types::fuel_types::BlockHeight;
use fuel_core_types::fuel_vm::GasCosts;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use std::str::FromStr;

use serde;

use fuel_core_types::fuel_vm::GasCostsValues;
use serde_with::As;
use serde_with::FromInto;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{to_string, Value};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum ConsensusConfig {
    PoA { signing_key: String },
}

// impl ConsensusConfig {
//     pub fn default_poa() -> Self {
//         ConsensusConfig::PoA {
//             signing_key: String::new(),
//         }
//     }
// }
//

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CoinConfig {
    pub tx_id: Option<Bytes32>,
    pub output_index: Option<u8>,
    pub tx_pointer_block_height: Option<BlockHeight>,
    pub tx_pointer_tx_idx: Option<u16>,
    pub maturity: Option<BlockHeight>,
    pub owner: Address,
    pub amount: u64,
    pub asset_id: AssetId,
}

impl<'de> Deserialize<'de> for CoinConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(CoinConfigVisitor)
    }
}

#[derive(Debug)]
pub struct CoinConfigVisitor;

impl<'de> Visitor<'de> for CoinConfigVisitor {
    type Value = CoinConfig;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a CoinConfig struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut tx_id = None;
        let mut output_index = None;
        let mut tx_pointer_block_height = None;
        let mut tx_pointer_tx_idx = None;
        let mut maturity = None;
        let mut owner = None;
        let mut amount = None;
        let mut asset_id = None;

        while let Some(key) = MapAccess::next_key::<String>(&mut map)? {
            match key.as_str() {
                "tx_id" => {
                    tx_id =
                        Some(Bytes32::from_str(MapAccess::next_value::<String>(&mut map)?.as_str()).expect("Should not fail"));
                }
                "output_index" => {
                    output_index = Some(
                        u8::from_str_radix(&MapAccess::next_value::<String>(&mut map)?[2..], 16)
                            .expect("Should not fail"),
                    );
                }
                "tx_pointer_block_height" => {
                    tx_pointer_block_height = Some(
                        BlockHeight::from_str(map.next_value::<String>().unwrap().as_str())
                            .expect("TODO: Panicari stari"),
                    );
                }
                "tx_pointer_tx_idx" => {
                    tx_pointer_tx_idx = Some(
                        u16::from_str_radix(&MapAccess::next_value::<String>(&mut map)?[2..], 16)
                            .expect("Should not fail"),
                    );
                }
                "maturity" => {
                    maturity = Some(
                        BlockHeight::from_str(map.next_value::<String>().unwrap().as_str())
                            .expect("TODO: Panicari stari"),
                    );
                }
                "owner" => {
                    owner = Some(
                        Address::from_str(map.next_value::<String>().unwrap().as_str())
                            .expect("TODO: Panicari stari"),
                    );
                }
                "amount" => {
                    amount = Some(
                        u64::from_str_radix(&map.next_value::<String>().unwrap()[2..], 16)
                            .expect("Should not fail"),
                    );
                }
                "asset_id" => {
                    asset_id = Some(
                        AssetId::from_str(map.next_value::<String>().unwrap().as_str())
                            .expect("TODO: Panicari stari"),
                    );
                }
                _ => {
                    let _ = map.next_value::<Value>();
                }
            }
        }

        Ok(CoinConfig {
            tx_id,
            output_index,
            tx_pointer_block_height,
            tx_pointer_tx_idx,
            maturity,
            owner: owner.ok_or_else(|| de::Error::missing_field("owner"))?,
            amount: amount.ok_or_else(|| de::Error::missing_field("amount"))?,
            asset_id: asset_id.ok_or_else(|| de::Error::missing_field("asset_id"))?,
        })
    }
}

#[derive(Debug)]
pub struct StateConfig {
    pub coins: Option<Vec<CoinConfig>>,
    pub contracts: Option<Vec<ContractConfig>>,
    pub messages: Option<Vec<MessageConfig>>,
    pub height: Option<BlockHeight>,
}

impl<'de> Deserialize<'de> for StateConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(StateConfigVisitor)
    }
}

#[derive(Debug)]
pub struct StateConfigVisitor;

impl<'de> Visitor<'de> for StateConfigVisitor {
    type Value = StateConfig;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut coins = None;
        let mut contracts = None;
        let mut messages = None;
        let mut height = None;

        while let Some(key) = MapAccess::next_key::<String>(&mut map)? {
            match key.as_str() {
                "coins" => {
                    dbg!("Kupovina Steneta");
                    // coins = MapAccess::next_value::<Option<Vec<CoinConfig>>>(&mut map)?;
                    coins = map.next_value_seed(CoinDeserializer)?;
                }
                "contracts" => {
                    contracts = map.next_value()?;
                }
                "messages" => {
                    messages = map.next_value()?;
                }
                "height" => {
                    height = Some(
                        BlockHeight::from_str(map.next_value::<String>().unwrap().as_str())
                            .expect("TODO: Panicari stari"),
                    );
                }
                _ => {
                    let _ = map.next_value::<Value>();
                }
            }
        }


        // Todo Add checks for errors
        Ok(StateConfig {
            coins,
            contracts,
            messages,
            height,
        })
    }
}

pub struct CoinDeserializer;

impl<'de> DeserializeSeed<'de> for CoinDeserializer {
    type Value = Option<Vec<CoinConfig>>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        let visitor = CoinVisitor;
        let values = deserializer.deserialize_seq(visitor)?;

        // let vec = values.as_object().unwrap().values().map(|value| {
        //     let map = value.as_object().unwrap();
        //     let tx_id = map.get("tx_id").and_then(Value::as_str).unwrap_or_default();
        //     let output_index = map.get("output_index").and_then(Value::as_str).unwrap_or_default();
        //     let tx_pointer_block_height = map.get("tx_pointer_block_height").and_then(Value::as_str).unwrap_or_default();
        //     let tx_pointer_tx_idx = map.get("tx_pointer_tx_idx").and_then(Value::as_str).unwrap_or_default();
        //     let maturity = map.get("maturity").and_then(Value::as_str).unwrap_or_default();
        //     let owner = map.get("owner").and_then(Value::as_str).unwrap_or_default();
        //     let amount = map.get("amount").and_then(Value::as_str).unwrap_or_default();
        //     let asset_id = map.get("asset_id").and_then(Value::as_str).unwrap_or_default();
        //
        //     CoinConfig {
        //         tx_id: Some(Bytes32::from_str(tx_id).unwrap()),
        //         output_index:  Some(u8::from_str_radix(&output_index[2..], 16).unwrap()),
        //         tx_pointer_block_height: Some(BlockHeight::from_str(tx_pointer_block_height).unwrap()),
        //         tx_pointer_tx_idx: Some(u16::from_str_radix(&tx_pointer_tx_idx[2..], 16).unwrap()),
        //         maturity:  Some(BlockHeight::from_str(maturity).unwrap()),
        //         owner: Address::from_str(owner).unwrap(),
        //         amount: u64::from_str_radix(&amount[2..], 16).unwrap(),
        //         asset_id: AssetId::from_str(asset_id).unwrap()
        //     }
        //
        // }).collect::<Vec<CoinConfig>>();

        Ok(Some(vec![]))
    }
}


pub struct CoinVisitor;
//
impl<'de> Visitor<'de> for CoinVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
    {
        let mut agg_map = serde_json::Map::new();

        let mut index = 0;

        while let Some(item) = seq.next_element()? {
            if let Value::Object(map) = item {
                let key = index.to_string(); // Use the index as the key
                agg_map.insert(key.to_string(), Value::Object(map));
                index += 1;
            }
        }

        let values = Value::Object(agg_map);

        Ok(values)
    }
}


#[derive(Debug, Deserialize)]
pub struct Data {
    pub chain_name: String,
    pub block_gas_limit: u64,
    #[serde(default)]
    pub initial_state: Option<StateConfig>,
    pub transaction_parameters: ConsensusParameters,
    #[serde(default)]
    #[serde(with = "As::<FromInto<GasCostsValues>>")]
    pub gas_costs: GasCosts,
    pub consensus: ConsensusConfig,
}

#[derive(Debug)]
pub struct DataVisitor;

impl<'de> Visitor<'de> for DataVisitor {
    type Value = Data;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut chain_name = None;
        let mut block_gas_limit = None;
        let mut initial_state = None;
        let mut transaction_parameters = None;
        let mut gas_costs = None;
        let mut consensus = None;

        while let Some(key) = MapAccess::next_key::<String>(&mut map)? {
            match key.as_str() {
                "chain_name" => {
                    chain_name = Some(MapAccess::next_value::<String>(&mut map)?);
                }
                "block_gas_limit" => {
                    block_gas_limit = Some(MapAccess::next_value::<i64>(&mut map)?);
                }
                "initial_state" => {
                    initial_state = Some(MapAccess::next_value::<Option<StateConfig>>(&mut map)?);
                }
                "transaction_parameters" => {
                    transaction_parameters =
                        Some(MapAccess::next_value::<ConsensusParameters>(&mut map)?);
                }
                "gas_costs" => {
                    gas_costs = Some(MapAccess::next_value::<GasCosts>(&mut map)?);
                }
                "consensus" => {
                    consensus = Some(MapAccess::next_value::<ConsensusConfig>(&mut map)?);
                }
                _ => {
                    let _ = MapAccess::next_value::<Value>(&mut map);
                }
            };
        }

        Ok(Data {
            chain_name: chain_name.ok_or_else(|| serde::de::Error::missing_field("chain_name"))?,
            block_gas_limit: block_gas_limit
                .ok_or_else(|| de::Error::missing_field("block_gas_limit"))?
                as u64,
            initial_state: initial_state
                .ok_or_else(|| de::Error::missing_field("initial_state"))?,
            transaction_parameters: transaction_parameters
                .ok_or_else(|| de::Error::missing_field("transaction_parameters"))?,
            gas_costs: gas_costs.ok_or_else(|| de::Error::missing_field("gas_costs"))?,
            consensus: consensus.ok_or_else(|| de::Error::missing_field("consensus"))?,
        })
    }
}
