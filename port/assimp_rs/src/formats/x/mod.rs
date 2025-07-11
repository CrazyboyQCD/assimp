pub mod errors;
pub mod exporter;
pub mod importer;
pub mod parser;
pub mod structs;

#[allow(unused)]
mod test {
    use std::{fs, io::Write};

    use super::importer::Importer;
    use crate::{
        formats::x::exporter::{self, Exporter},
        structs::scene::AiScene,
        traits::importer::trait_define::InternalImporter,
        utils::{float_precision::Mat4, get_model_path},
    };
    // #[test]
    // fn test_import_from_file() {
    //     let file_path = get_model_path("X", "WP_spear.X");
    //     // println!("file_path: {:?}", file_path.display());
    //     let mut scene = AiScene::default();
    //     let source = fs::read(file_path).unwrap();
    //     // let t = std::time::Instant::now();
    //     // Importer::import_from_buf(source.as_slice(), &mut scene).unwrap();
    //     // println!("time: {:?}", t.elapsed());
    //     // println!("scene: {:#?}", scene);
    //     fs::write(
    //         "WP_spear_tokens.txt",
    //         format!(
    //             "{:#?}",
    //             Importer::get_tokens(&source)
    //                 .unwrap()
    //                 .iter()
    //                 .map(|v| {
    //                     match str::from_utf8(v) {
    //                         Ok(s) => s.to_owned(),
    //                         Err(e) => format!("bytes[{}]: {:02X?}", v.len(), v),
    //                     }
    //                 })
    //                 .collect::<Vec<String>>()
    //         ),
    //     )
    //     .unwrap();
    //     // assert_eq!(scene.nodes.len(), 1);
    // }

    #[test]
    fn test_export_to_file() {
        let file_path = get_model_path("X", "test.X");
        let source = fs::read(file_path).unwrap();
        let t = std::time::Instant::now();
        let mut scene = AiScene::default();
        Importer::import_from_buf(source.as_slice(), &mut scene).unwrap();
        println!("parse time: {:?}", t.elapsed());
        let mut b = Default::default();
        let mut exporter = Exporter::new(&scene, &b);
        let mut writer = fs::File::create("test.txt").unwrap();
        let mut s = String::new();
        let t = std::time::Instant::now();
        exporter.write_to_stream(&mut s).unwrap();
        println!("export time: {:?}", t.elapsed());
        let t = std::time::Instant::now();
        writer.write_all(s.as_bytes()).unwrap();
        writer.flush().unwrap();
        println!("flush time: {:?}", t.elapsed());
    }
}
