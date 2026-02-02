use async_graphql::SDLExportOptions;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let schema = backend::api::schema();
    let opts = SDLExportOptions::default();
    let mut file = File::create("frontend/schema.graphql").await.unwrap();
    file.write_all(schema.sdl_with_options(opts).as_bytes())
        .await
        .unwrap();
}
