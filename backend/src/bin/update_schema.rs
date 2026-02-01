use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::Schema;

use backend::api::Query;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let mut file = File::create("frontend/src/lib/api.graphql").await.unwrap();
    file.write_all(schema.sdl().as_bytes()).await.unwrap();
}
