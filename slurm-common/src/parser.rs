use regex::Regex;
use serde::{de, forward_to_deserialize_any};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error {
            message: msg.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn from_str<'de, T: de::Deserialize<'de>>(input: &'de str) -> Result<T> {
    let deserializer = SlurmDeserializer::from_str(input);
    T::deserialize(deserializer)
}

pub struct SlurmDeserializer<'de> {
    input: &'de str,
}

impl<'de> SlurmDeserializer<'de> {
    fn from_str(input: &'de str) -> Self {
        SlurmDeserializer { input }
    }
}

impl<'de> de::Deserializer<'de> for SlurmDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let records: Vec<&str> = self
            .input
            .split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        // If there is only one record, it's a map, otherwise it's a sequence
        if records.len() == 1 {
            self.deserialize_map(visitor)
        } else {
            self.deserialize_seq(visitor)
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Split by blank lines (double newlines)
        let records: Vec<&str> = self
            .input
            .split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        visitor.visit_seq(RecordSeq {
            records,
            current: 0,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Deserialize the first record as a map
        let record = self
            .input
            .split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .next()
            .ok_or_else(|| de::Error::custom("No record found"))?;
        let mut map = HashMap::new();
        let key_regex = Regex::new(r"(?:^|[\s])([a-zA-Z0-9_\/-:.]+)=")
            .map_err(|e| de::Error::custom(e.to_string()))?;

        let matches: Vec<_> = key_regex.find_iter(record).collect();
        println!("matches: {:?}", matches);

        for i in 0..matches.len() {
            let m = matches[i];
            let key_capture = key_regex.captures(m.as_str()).unwrap().get(1).unwrap();
            println!("key capture: {}", m.as_str());
            let key = key_capture.as_str();

            let val_start = m.end();
            let val_end = if i + 1 < matches.len() {
                matches[i + 1].start()
            } else {
                record.len()
            };

            let raw_value = &record[val_start..val_end];
            let value = raw_value.trim_matches(|c| c == ' ' || c == '\n' || c == ',' || c == '\r');
            println!(
                "key: {}, value: {}, val_start: {}, val_end: {}",
                key, value, val_start, val_end
            );

            // Skip "null," None, or empty values
            if value.is_empty() || value == "(null)" || value == "None" {
                continue;
            }
            match map.entry(key) {
                Entry::Occupied(mut entry) => {
                    let existing = entry.get_mut();
                    match existing {
                        SlurmValue::Single(s) => {
                            *existing = SlurmValue::Repeated(vec![s, value]);
                        }
                        SlurmValue::Repeated(v) => v.push(value),
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(SlurmValue::Single(value));
                }
            }
        }

        // Convert to vec for MapAccess
        // We sort keys? No, MapAccess doesn't require order unless struct requires it?
        // Actually standard HashMap iteration is random. Serde is fine with that for maps/structs usually.
        let items: Vec<(&str, SlurmValue<'de>)> = map.into_iter().collect();
        visitor.visit_map(SlurmRecord { items, current: 0 })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct enum identifier ignored_any struct option
    }
}

struct RecordSeq<'de> {
    records: Vec<&'de str>,
    current: usize,
}

impl<'de> de::SeqAccess<'de> for RecordSeq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.current >= self.records.len() {
            return Ok(None);
        }
        let record = self.records[self.current];
        self.current += 1;
        seed.deserialize(SlurmDeserializer::from_str(record))
            .map(Some)
    }
}

struct SlurmRecord<'de> {
    items: Vec<(&'de str, SlurmValue<'de>)>,
    current: usize,
}

impl<'de> de::MapAccess<'de> for SlurmRecord<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.current >= self.items.len() {
            return Ok(None);
        }
        // Deserialize the key (which is a string)
        seed.deserialize(de::value::StrDeserializer::new(&self.items[self.current].0))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self.items[self.current].1.clone();
        self.current += 1;
        // Use our ValueDeserializer that can parse strings into numbers
        seed.deserialize(value)
    }
}
// A value in a record
#[derive(Clone)]
enum SlurmValue<'de> {
    Single(&'de str),
    Repeated(Vec<&'de str>), // A key that appears multiple times
}

// Use paste! macro to implement the visitor for each number type
macro_rules! impl_num_visitor {
    {$($type:ident)*} => {
        paste::paste! {
            $(fn [<deserialize_ $type>]<V>(self, visitor: V) -> Result<V::Value>
            where
                V: de::Visitor<'de>,
            {
                match self {
                    SlurmValue::Single(s) => visitor.[<visit_ $type>](s.parse().map_err(de::Error::custom)?),
                    SlurmValue::Repeated(_) => self.deserialize_any(visitor),
                }
            })*
        }
    }
}

impl<'de> de::Deserializer<'de> for SlurmValue<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Default to string
        match self {
            SlurmValue::Single(s) => visitor.visit_str(s),
            SlurmValue::Repeated(v) => visitor.visit_seq(ValueSeq {
                values: v,
                current: 0,
            }),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Since we filtered NULLs/Nones at map level, if we are here, it's Some.
        visitor.visit_some(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            SlurmValue::Single(s) => {
                if s == "1" || s.eq_ignore_ascii_case("true") {
                    visitor.visit_bool(true)
                } else if s == "0" || s.eq_ignore_ascii_case("false") {
                    visitor.visit_bool(false)
                } else {
                    Err(de::Error::custom(format!("Expected bool, got {}", s)))
                }
            }
            SlurmValue::Repeated(_) => self.deserialize_any(visitor),
        }
    }

    // special handler for seq, map
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            SlurmValue::Single(s) => visitor.visit_seq(ValueSeq {
                values: s.split(",").collect(),
                current: 0,
            }),
            SlurmValue::Repeated(v) => visitor.visit_seq(ValueSeq {
                values: v,
                current: 0,
            }),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let items = match self {
            SlurmValue::Single(s) => s
                .split(",")
                .map(|s| {
                    s.split_once("=")
                        .ok_or(de::Error::custom("Invalid key-value pair"))
                })
                .collect::<Result<Vec<(_, _)>>>()?,
            SlurmValue::Repeated(v) => v
                .iter()
                .map(|s| {
                    s.split_once("=")
                        .ok_or(de::Error::custom("Invalid key-value pair"))
                })
                .collect::<Result<Vec<(_, _)>>>()?,
        };
        visitor.visit_map(ValueMap { items, current: 0 })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            SlurmValue::Single(s) => visitor.visit_str(s),
            SlurmValue::Repeated(v) => visitor.visit_str(v[0]),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            SlurmValue::Single(s) => visitor.visit_seq(ValueSeq {
                values: s.split(",").collect(),
                current: 0,
            }),
            SlurmValue::Repeated(v) => visitor.visit_seq(ValueSeq {
                values: v,
                current: 0,
            }),
        }
    }

    fn deserialize_tuple_struct<V>(self, _name: &str, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(_len, visitor)
    }
    fn deserialize_struct<V>(self, _name: &str, _fields: &[&str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    impl_num_visitor! {
        u8 u16 i8 i16 u128 i128
        u64 u32 i64 i32 f32 f64
    }

    forward_to_deserialize_any! {
        char str string
        bytes byte_buf unit unit_struct newtype_struct enum ignored_any
    }
}

// A sequence of values
struct ValueSeq<'de> {
    values: Vec<&'de str>,
    current: usize,
}

impl<'de> de::SeqAccess<'de> for ValueSeq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.current >= self.values.len() {
            return Ok(None);
        }
        let value = self.values[self.current];
        self.current += 1;
        seed.deserialize(SlurmValue::Single(value)).map(Some)
    }
}

struct ValueMap<'de> {
    items: Vec<(&'de str, &'de str)>,
    current: usize,
}

impl<'de> de::MapAccess<'de> for ValueMap<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.current >= self.items.len() {
            return Ok(None);
        }
        let key = self.items[self.current].0;
        seed.deserialize(de::value::StrDeserializer::new(key))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self.items[self.current].1;
        self.current += 1;
        seed.deserialize(SlurmValue::Single(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_parse_node() {
        let input = "NodeName=node4504 Arch=x86_64 CoresPerSocket=32
   CPUAlloc=0 CPUEfctv=64 CPUTot=64 CPULoad=0.04
   AvailableFeatures=node4504,rocky8
   ActiveFeatures=node4504,rocky8
   Gres=gpu:l40s:4(S:0-1)
   NodeAddr=node4504 NodeHostName=node4504 Version=25.05.4
   OS=Linux 4.18.0-372.9.1.el8.x86_64 #1 SMP Tue May 10 14:48:47 UTC 2022
   RealMemory=1031314 AllocMem=0 FreeMem=77430 Sockets=2 Boards=1
   RestrictedCoresPerGPU=16(0-63)
   State=IDLE ThreadsPerCore=2 TmpDisk=0 Weight=1 Owner=N/A MCS_label=N/A
   Partitions=mit_preemptable
   BootTime=2025-07-07T16:34:43 SlurmdStartTime=2026-01-29T15:08:10
   LastBusyTime=2026-01-29T15:08:03 ResumeAfterTime=None
   CfgTRES=cpu=64,mem=1031314M,billing=64,gres/gpu=4,gres/gpu:l40s=4
   AllocTRES=
   CurrentWatts=0 AveWatts=0";

        #[allow(non_snake_case)]
        #[derive(Deserialize, Debug, PartialEq)]
        struct Node {
            NodeName: String,
            Arch: String,
            OS: String,
            CoresPerSocket: u32,
            CPULoad: f32,
            Gres: String,
            AllocTRES: Option<String>,
            State: String,
        }

        let node: Node = Node::deserialize(SlurmDeserializer::from_str(input)).unwrap();
        println!("{:?}", node);
        assert_eq!(node.NodeName, "node4504");
        assert_eq!(node.Arch, "x86_64");
        assert_eq!(node.CoresPerSocket, 32);
        assert_eq!(node.CPULoad, 0.04);
        assert_eq!(node.Gres, "gpu:l40s:4(S:0-1)");
        assert_eq!(
            node.OS,
            "Linux 4.18.0-372.9.1.el8.x86_64 #1 SMP Tue May 10 14:48:47 UTC 2022"
        );
        assert_eq!(node.AllocTRES, None); // Should be None due to empty value
        assert_eq!(node.State, "IDLE");
    }

    #[test]
    fn test_parse_job() {
        let input = "JobId=8601779 JobName=8445fb49-9088-4fd5-b463-65b76bf6c4bb
   UserId=cysteine(135712) GroupId=cysteine(100135712) MCS_label=N/A
   Priority=21212 Nice=0 Account=mit_general QOS=normal
   JobState=RUNNING Reason=None Dependency=(null)
   Requeue=0 Restarts=0 BatchFlag=1 Reboot=0 ExitCode=0:0
   RunTime=00:15:41 TimeLimit=08:00:00 TimeMin=N/A
   SubmitTime=2026-01-31T12:44:31 EligibleTime=2026-01-31T12:44:31
   AccrueTime=2026-01-31T12:44:31
   StartTime=2026-01-31T12:45:05 EndTime=2026-01-31T20:45:05 Deadline=N/A
   SuspendTime=None SecsPreSuspend=0 LastSchedEval=2026-01-31T12:45:05 Scheduler=Main
   Partition=sched_mit_hill AllocNode:Sid=node2429:26654
   ReqNodeList=(null) ExcNodeList=(null)
   NodeList=node156
   BatchHost=node156
   NumNodes=1 NumCPUs=1 NumTasks=1 CPUs/Task=1 ReqB:S:C:T=0:0:*:*
   ReqTRES=cpu=1,mem=15000M,node=1,billing=1
   AllocTRES=cpu=1,mem=15000M,node=1,billing=1
   Socks/Node=* NtasksPerN:B:S:C=0:0:*:* CoreSpec=*
   MinCPUsNode=1 MinMemoryNode=15000M MinTmpDiskNode=0
   Features=(null) DelayBoot=00:00:00
   OverSubscribe=OK Contiguous=0 Licenses=(null) LicensesAlloc=(null) Network=(null)
   Command=(null)
   WorkDir=/orcd/data/tami/003/projects/cysteine/dnds
   Comment=rule_pairwise_batch_wildcards__orcd_data_tami_003_projects_cysteine_dnds_bfragilis_396";

        #[allow(non_snake_case)]
        #[derive(Deserialize, Debug, PartialEq)]
        struct Job {
            JobId: u64,
            JobName: String,
            JobState: String,
            Reason: Option<String>,
            Dependency: Option<String>,
            ReqTRES: String,
        }

        let job: Job = Job::deserialize(SlurmDeserializer::from_str(input)).unwrap();
        println!("{:?}", job);
        assert_eq!(job.JobId, 8601779);
        assert_eq!(job.JobName, "8445fb49-9088-4fd5-b463-65b76bf6c4bb");
        assert_eq!(job.JobState, "RUNNING");
        assert_eq!(job.Reason, None);
        assert_eq!(job.Dependency, None);
        assert_eq!(job.ReqTRES, "cpu=1,mem=15000M,node=1,billing=1");
    }

    #[test]
    fn test_parse_tres() {
        let input = "cpu=64,mem=1031314M,billing=64,gres/gpu=4,gres/gpu:l40s=4";

        // Dynamic map, expecting raw strings content
        let map = HashMap::<String, String>::deserialize(SlurmValue::Single(input)).unwrap();
        println!("{:?}", map);
        assert_eq!(map.get("cpu").unwrap(), "64");
        assert_eq!(map.get("mem").unwrap(), "1031314M");
        assert_eq!(map.get("gres/gpu").unwrap(), "4");
        assert_eq!(map.get("gres/gpu:l40s").unwrap(), "4");
    }

    #[test]
    fn test_parse_multi_records() {
        let input = "NodeName=node1 State=IDLE\n\n\nNodeName=node2 State=ALLOCATED";

        #[allow(non_snake_case)]
        #[derive(Deserialize, Debug, PartialEq)]
        struct Node {
            NodeName: String,
            State: String,
        }

        let nodes = Vec::<Node>::deserialize(SlurmDeserializer::from_str(input)).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].NodeName, "node1");
        assert_eq!(nodes[0].State, "IDLE");
        assert_eq!(nodes[1].NodeName, "node2");
        assert_eq!(nodes[1].State, "ALLOCATED");
    }
}
