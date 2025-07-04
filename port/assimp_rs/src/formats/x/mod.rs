pub mod errors;
pub mod exporter;
pub mod importer;
pub mod parser;
pub mod structs;

#[allow(unused)]
mod test {
    use std::fs;

    use crate::{
        structs::scene::AiScene, traits::importer::trait_define::InternalImporter,
        utils::get_model_path,
    };

    use super::importer::Importer;
    #[test]
    fn test_import_from_file() {
        let file_path = get_model_path("X", "test_cube_compressed.x");
        // println!("file_path: {:?}", file_path.display());
        let mut scene = AiScene::default();
        let source = fs::read(file_path).unwrap();
        let t = std::time::Instant::now();
        Importer::import_from_buf(source.as_slice(), &mut scene).unwrap();
        println!("time: {:?}", t.elapsed());
        // println!("scene: {:#?}", scene);
        fs::write("test_cube_text.txt", format!("{:#?}", scene)).unwrap();
        // assert_eq!(scene.nodes.len(), 1);
    }
}
