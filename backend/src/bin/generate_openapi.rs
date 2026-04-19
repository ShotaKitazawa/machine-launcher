use machine_launcher::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let doc = ApiDoc::openapi();
    print!(
        "{}",
        serde_yaml::to_string(&doc).expect("failed to serialize OpenAPI spec")
    );
}
