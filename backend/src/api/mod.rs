use async_graphql::*;
use serde::{Deserialize, Serialize};

mod job;
mod partition;

pub use job::*;
pub use partition::*;

pub trait HasId {
    type Id;
    fn id(&self) -> &Self::Id;
}

pub trait Selectable: HasId + Sized {
    type Selected: From<Self> + HasId<Id = Self::Id> + Sized;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Ref<T: Selectable> {
    Id(T::Id),
    Selected(T::Selected),
}

pub struct Query;

#[Object]
impl Query {
    async fn nodes(&self) -> u8 {
        0
    }
}

pub fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}
