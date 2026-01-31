use serde::Deserialize;
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum NodeStateInfo {
    Idle,
    Alloc,
    Mix,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeInfo<'src> {
    #[serde(rename = "NodeName")]
    pub name: &'src str,
    #[serde(rename = "State")]
    pub state: NodeStateInfo,
    #[serde(rename = "CPUAlloc")]
    pub cpu_alloc: u32,
    #[serde(rename = "CPUTot")]
    pub cpus: u32,
    #[serde(rename = "RealMemory")]
    pub real_memory: u32,
    #[serde(rename = "AllocMem")]
    pub alloc_mem: u32,
    #[serde(rename = "FreeMem")]
    pub free_mem: u32,
    #[serde(rename = "Partitions")]
    pub partitions: Vec<&'src str>,
    #[serde(rename = "CfgTRES")]
    pub resources: BTreeMap<&'src str, ResourceQuantity>,
    #[serde(rename = "AllocTRES")]
    pub allocated: BTreeMap<&'src str, ResourceQuantity>,
}

// Will handle parsing memory M and G suffixes
#[derive(Debug, Clone)]
pub struct ResourceQuantity(u64);

impl Into<u64> for ResourceQuantity {
    fn into(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ResourceQuantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResVisitor;

        impl<'de> serde::de::Visitor<'de> for ResVisitor {
            type Value = ResourceQuantity;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string like '100M' or '1G' or a raw number")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let s = v.trim();
                let mut multiplier = 1;
                let mut value = s;
                if s.ends_with('M') {
                    multiplier = 1024 * 1024;
                    value = &s[..s.len() - 1];
                } else if s.ends_with('G') {
                    multiplier = 1024 * 1024 * 1024;
                    value = &s[..s.len() - 1];
                }
                Ok(ResourceQuantity(
                    value.parse::<u64>().map_err(serde::de::Error::custom)? * multiplier,
                ))
            }
        }
        deserializer.deserialize_str(ResVisitor)
    }
}
