pub mod error;
pub mod importer;
pub mod parser;
pub mod writer;

const STRING_ENCODINGS: &[&encoding_rs::Encoding] = &[
    encoding_rs::UTF_16LE,
    encoding_rs::SHIFT_JIS,
    encoding_rs::GBK,
    encoding_rs::GB18030,
];

#[allow(unused)]
mod test {
    #[cfg(feature = "std")]
    use std::{fs, io::Write};

    #[cfg(feature = "std")]
    use crate::io::utils::get_model_path;
    use crate::{
        AiMat4,
        assets::{
            formats::x::{
                exporter::{self, Exporter},
                importer::XFormatImporter,
            },
            postprocess::{
                PostProcess,
                convert_to_left_hand_process::{
                    ConvertToLeftHandProcess, flip_uvs_process::FlipUVsProcess,
                    flip_winding_order_process::FlipWindingOrderProcess,
                },
            },
        },
        io::importer::traits::InternalImporter,
        structs::scene::AiScene,
    };

    #[test]
    #[cfg(feature = "std")]
    fn test_vmd_parser() {
        use crate::assets::formats::mmd::parser::vmd::{VMDParser, VMDRead, structs::VmdMotion};
        let data = fs::read("C:/Users/cnwxs/Downloads/physics_toggle_test_v2_yyb10th.vmd").unwrap();
        let t = std::time::Instant::now();
        for _ in 0..1000 {
            let mut parser = VMDParser::new(data.as_slice());
            let _ = VmdMotion::read(&mut parser).unwrap();
        }
        println!("parse time: {:?}", t.elapsed());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_pmx_parser() {
        use crate::assets::formats::mmd::parser::pmx::{PMXParser, PMXRead, structs::PmxModel};
        let data = fs::read("C:/Users/cnwxs/Downloads/constraint_test.pmx").unwrap();
        let t = std::time::Instant::now();
        for _ in 0..1000 {
            let mut parser = PMXParser::new(data.as_slice());
            let model = PmxModel::read(&mut parser).unwrap();
            // println!("{:#?}", model);
        }
        println!("parse time: {:?}", t.elapsed());
    }
}
