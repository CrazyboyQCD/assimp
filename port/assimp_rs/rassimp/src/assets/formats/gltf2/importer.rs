use gltf::{Gltf, image, import_buffers, import_images};

use crate::{
    AiReal,
    assets::formats::gltf2::error::GltfImportError,
    io::importer::traits::{EmptyConfig, InternalImporter},
    structs::{
        importer_desc::{ImporterDesc, ImporterFlags},
        material::{
            AiMaterial,
            property::{AiBasicProperty, AiColorDiffuseProperty, AiProperty, AiUVTransform},
        },
        scene::AiScene,
        texture::AiTexture,
    },
};

static DESC: ImporterDesc = ImporterDesc {
    name: "glTF Importer",
    author: "",
    maintainer: "",
    comments: "",
    flags: ImporterFlags::from_bits_retain(
        ImporterFlags::SUPPORT_TEXT_FLAVOUR.bits()
            | ImporterFlags::SUPPORT_BINARY_FLAVOUR.bits()
            | ImporterFlags::SUPPORT_COMPRESSED_FLAVOUR.bits(),
    ),
    min_major: 0,
    min_minor: 0,
    max_major: 0,
    max_minor: 0,
    file_extensions: &["gltf", "GLTF"],
};

pub struct GltfFomatImporter;

impl GltfFomatImporter {
    pub fn desc() -> &'static ImporterDesc {
        &DESC
    }

    fn import_embedded_textures(image_data: Vec<image::Data>, gltf: &Gltf, scene: &mut AiScene) {
        scene.textures.reserve(image_data.len());
        for (img_data, img) in image_data.into_iter().zip(gltf.images()) {
            let name = img.name().unwrap_or("unknown_image_name");
            let width = img_data.width;
            let height = img_data.height;
            scene.textures.push(AiTexture {
                file_name: name.to_owned(),
                width,
                height,
                pc_data: img_data.pixels.clone(),
                ..Default::default()
            });
        }
    }

    fn import_materials(gltf: &Gltf, scene: &mut AiScene, embedded_tex_idxs: &[usize]) {
        let materials = gltf.materials();
        scene.materials.reserve(materials.len());
        for material in materials {
            let mut ai_material = AiMaterial::default();
            if let Some(name) = material.name()
                && !name.is_empty()
            {
                ai_material.add_property(AiProperty::MaterialName(name.to_owned()), 0);
            }
            let pbr_metallic_roughness = material.pbr_metallic_roughness();
            let base_color_property =
                AiColorDiffuseProperty::Color4D(pbr_metallic_roughness.base_color_factor().into());
            ai_material.add_property(
                AiProperty::MaterialColorDiffuse(base_color_property.clone()),
                0,
            );
            ai_material.add_property(AiProperty::MaterialBaseColor(base_color_property), 0);

            if let Some(metallic_roughtness_texture) =
                pbr_metallic_roughness.metallic_roughness_texture()
            {
                let tex = metallic_roughtness_texture.texture();

                if let image::Source::Uri { uri, .. } = tex.source().source() {
                    let texture_base = AiBasicProperty::String(
                        if let Some(idx) = embedded_tex_idxs.get(tex.index()) {
                            format!("*{idx}")
                        } else {
                            uri.to_owned()
                        },
                    );
                    ai_material.add_property(
                        AiProperty::Custom((
                            "AI_MATKEY_GLTF_PBRMETALLICROUGHNESS_METALLICROUGHNESS_TEXTURE".into(),
                            texture_base.clone(),
                        )),
                        0,
                    );
                    ai_material.add_property(
                        AiProperty::Custom(("AI_MATKEY_METALNESS".into(), texture_base.clone())),
                        0,
                    );
                    ai_material.add_property(
                        AiProperty::Custom(("AI_MATKEY_DIFFUSE_ROUGHNESS".into(), texture_base)),
                        0,
                    );
                }

                let uv_index = AiBasicProperty::Int(metallic_roughtness_texture.tex_coord() as _);
                ai_material.add_property(
                    AiProperty::Custom((
                        "AI_MATKEY_GLTF_PBRMETALLICROUGHNESS_METALLICROUGHNESS_TEXTURE_UWSRC"
                            .into(),
                        uv_index.clone(),
                    )),
                    0,
                );
                ai_material.add_property(
                    AiProperty::Custom(("AI_MATKEY_METALNESS_UWSRC".into(), uv_index.clone())),
                    0,
                );
                ai_material.add_property(
                    AiProperty::Custom(("AI_MATKEY_METALNESS_UWSRC".into(), uv_index)),
                    0,
                );
                if let Some(transform) = metallic_roughtness_texture.texture_transform() {
                    let rotation = AiReal::from(transform.rotation());
                    let mut res = AiUVTransform {
                        scaling: transform.scale().map(AiReal::from).into(),
                        rotation: -rotation,
                        ..Default::default()
                    };

                    // A change of coordinates is required to map glTF UV transformations into the
                    // space used by Assimp. In glTF all UV origins are at 0,1
                    // (top left of texture) in Assimp space. In Assimp rotation
                    // occurs around the image center (0.5,0.5) where as in glTF rotation is around
                    // the texture origin. All three can be corrected for solely
                    // by a change of the translation since the transformations
                    // available are shape preserving. Note the importer already flips the V
                    // coordinate of the actual meshes during import.
                    let rcos = (-rotation).cos();
                    let rsin = (-rotation).sin();
                    let offset = transform.offset().map(AiReal::from);
                    res.translation.x = (0.5 * res.scaling.x) * (-rcos + rsin + 1.0) + offset[0];
                    res.translation.y = ((0.5 * res.scaling.y) * (rsin + rcos - 1.0)) + 1.0
                        - res.scaling.y
                        - offset[1];
                    ai_material.add_property(AiProperty::UvTransform(res), 0);
                }
            }

            ai_material.add_property(
                AiProperty::MaterialMetallicFactor(pbr_metallic_roughness.metallic_factor()),
                1,
            );
            let roughness_factor = pbr_metallic_roughness.roughness_factor();
            ai_material.add_property(AiProperty::MaterialRoughnessFactor(roughness_factor), 1);
            let roughness_as_shininess = 1.0 - roughness_factor;
            let roughness_as_shininess = roughness_as_shininess * (roughness_as_shininess * 1000.0);
            ai_material.add_property(AiProperty::MaterialShininess(roughness_as_shininess), 1);

            if let Some(normal_texture) = material.normal_texture() {
                let tex = normal_texture.texture();
                normal_texture.tex_coord();
                if let image::Source::Uri { uri, .. } = tex.source().source() {
                    let texture_base = if let Some(idx) = embedded_tex_idxs.get(tex.index()) {
                        format!("*{idx}")
                    } else {
                        uri.to_owned()
                    };
                    let property = AiBasicProperty::String(texture_base);
                    ai_material.add_property(
                        AiProperty::Custom((
                            "AI_MATKEY_GLTF_PBRMETALLICROUGHNESS_METALLICROUGHNESS_TEXTURE".into(),
                            property.clone(),
                        )),
                        0,
                    );
                    ai_material.add_property(
                        AiProperty::Custom(("AI_MATKEY_METALNESS".into(), property.clone())),
                        0,
                    );
                    ai_material.add_property(
                        AiProperty::Custom((
                            "AI_MATKEY_DIFFUSE_ROUGHNESS".into(),
                            property.clone(),
                        )),
                        0,
                    );
                }
            }

            scene.materials.push(ai_material);
        }
    }
}

impl InternalImporter<GltfImportError> for GltfFomatImporter {
    type ExtraConfig = EmptyConfig;

    #[cfg(feature = "std")]
    fn import_from_file(
        file_path: &std::path::Path,
        scene: &mut crate::structs::scene::AiScene,
        _config: Self::ExtraConfig,
    ) -> Result<(), GltfImportError> {
        let mut gltf_file = gltf::Gltf::open(file_path)?;
        let document = &gltf_file.document;
        let buffer = import_buffers(document, None, gltf_file.blob.take())?;
        let image_data = import_images(document, None, &buffer)?;
        Self::import_embedded_textures(image_data, &gltf_file, scene);
        todo!()
    }

    fn import_from_buf(
        buf: &[u8],
        _scene: &mut crate::structs::scene::AiScene,
        _config: Self::ExtraConfig,
    ) -> Result<(), GltfImportError> {
        let (_doc, _buffer_data, _image_data) = gltf::import_slice(buf)?;

        todo!()
    }
}
