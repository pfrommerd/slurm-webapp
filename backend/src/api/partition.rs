use crate::api::{HasId, Ref, Selectable};
use async_graphql::{NewType, Object};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, NewType, Serialize, Deserialize)]
pub struct ClusterId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, NewType, Serialize, Deserialize)]
pub struct PartitionId(Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cluster {
    pub id: ClusterId,
    pub name: String,
    pub partitions: Vec<Ref<Partition>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterPartial {
    pub id: ClusterId,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Partition {
    pub id: PartitionId,
    pub name: String,
    pub cluster: Ref<Cluster>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionPartial {
    pub id: PartitionId,
    pub name: Option<String>,
    pub cluster: Option<Ref<Cluster>>,
}

#[Object(name = "Cluster")]
impl Ref<Cluster> {
    async fn id(&self) -> &ClusterId {
        match self {
            Ref::Id(id) => id,
            Ref::Selected(selected) => &selected.id,
        }
    }
}

#[rustfmt::skip]
mod impls {
    use super::*;
    impl From<Cluster> for ClusterPartial {
        fn from(cluster: Cluster) -> Self {
            ClusterPartial {
                id: cluster.id,
                name: Some(cluster.name),
            }
        }
    }
    impl From<Partition> for PartitionPartial {
        fn from(partition: Partition) -> Self {
            PartitionPartial {
                id: partition.id,
                name: Some(partition.name),
                cluster: Some(partition.cluster),
            }
        }
    }
    impl HasId for Cluster {
        type Id = ClusterId;
        fn id(&self) -> &ClusterId { &self.id }
    }
    impl HasId for ClusterPartial {
        type Id = ClusterId;
        fn id(&self) -> &ClusterId { &self.id }
    }
    impl HasId for Partition {
        type Id = PartitionId;
        fn id(&self) -> &PartitionId { &self.id }
    }
    impl HasId for PartitionPartial {
        type Id = PartitionId;
        fn id(&self) -> &PartitionId { &self.id }
    }
    impl Selectable for Cluster {
        type Selected = ClusterPartial;
    }
    impl Selectable for Partition {
        type Selected = PartitionPartial;
    }
}
