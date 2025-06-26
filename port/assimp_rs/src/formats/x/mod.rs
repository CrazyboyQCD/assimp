pub mod errors;
pub mod exporter;
pub mod importer;
pub mod parser;
pub mod structs;

#[allow(unused)]
mod test {
    use crate::{
        structs::scene::AiScene, traits::importer::trait_define::InternalImporter,
        utils::get_model_path,
    };

    use super::importer::Importer;
    #[test]
    fn test_import_from_file() {
        let file_path = get_model_path("X", "test.x");
        println!("file_path: {:?}", file_path.display());
        let mut scene = AiScene::default();
        Importer::import_from_file(file_path.to_str().unwrap(), &mut scene).unwrap();
        println!("scene: {:?}", scene);
        // assert_eq!(scene.nodes.len(), 1);
    }
}
