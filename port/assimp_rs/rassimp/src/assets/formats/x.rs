/*
---------------------------------------------------------------------------
Open Asset Import Library (assimp)
---------------------------------------------------------------------------

Copyright (c) 2006-2025, assimp team

All rights reserved.

Redistribution and use of this software in source and binary forms,
with or without modification, are permitted provided that the following
conditions are met:

* Redistributions of source code must retain the above
  copyright notice, this list of conditions and the
  following disclaimer.

* Redistributions in binary form must reproduce the above
  copyright notice, this list of conditions and the
  following disclaimer in the documentation and/or other
  materials provided with the distribution.

* Neither the name of the assimp team, nor the names of its
  contributors may be used to endorse or promote products
  derived from this software without specific prior
  written permission of the assimp team.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
---------------------------------------------------------------------------
*/

//! Implements X format importer and exporter for the library

pub mod error;
pub mod exporter;
pub mod importer;
pub mod parser;
pub mod structs;

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
    #[cfg(feature = "std")]
    fn test_export_to_file() {
        use core::fmt::Write as FmtWrite;
        use std::{io::Write, path::PathBuf};

        use crate::assets::formats::assxml::writer::AssxmlWriter;

        let file_path = PathBuf::from("C:\\Users\\cnwxs\\Downloads\\Satoko.X");
        // get_model_path("X", "test.x")

        // println!("file_path: {:?}", file_path.display());
        // let source = fs::read(file_path).unwrap();
        // let t = std::time::Instant::now();
        // for _ in 0..1 {
        let mut scene = AiScene::default();
        XFormatImporter::import_from_file(&file_path, &mut scene, Default::default()).unwrap();
        // }
        // println!("parse time: {:?}", t.elapsed());
        // let mut scene = scene.clone();
        // FlipWindingOrderProcess::execute(&mut scene);
        // ConvertToLeftHandProcess::execute(&mut scene);
        // FlipUVsProcess::execute(&mut scene);
        // let mut s = String::new();
        // let mut exporter = Exporter::new(&scene, &mut s);
        // let mut writer = fs::File::create("../test.txt").unwrap();
        // let t = std::time::Instant::now();
        // exporter.write_to_stream().unwrap();
        // println!("export time: {:?}", t.elapsed());
        // let t = std::time::Instant::now();
        // writer.write_all(s.as_bytes()).unwrap();
        // writer.flush().unwrap();
        // println!("flush time: {:?}", t.elapsed());
        // fs::write("../test_0.txt", format!("{:#?}", scene)).unwrap();
        // let mut f = fs::File::create("../test_assxml.txt").unwrap();
        // struct FmtWriter<W: std::io::Write>(W);

        // impl<W: std::io::Write> std::fmt::Write for FmtWriter<W> {
        //     fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        //         self.0.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
        //     }

        //     fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<(), std::fmt::Error>
        // {         self.0.write_fmt(args).map_err(|_| std::fmt::Error)
        //     }
        // }
        // // let mut writer = FmtWriter(std::io::BufWriter::new(&mut f));
        // let mut writer = String::new();

        // let t = std::time::Instant::now();
        // AssxmlWriter::new(&scene, &mut writer)
        //     .write_dump(
        //         std::path::Path::new("C:\\Users\\cnwxs\\Downloads\\Satoko.X"),
        //         "",
        //         false,
        //     )
        //     .unwrap();
        // println!("assxml export time: {:?}", t.elapsed());
    }
}
