#![allow(clippy::print_with_newline, clippy::type_complexity)]
use std::thread::Scope;

use rassimp::{
    AiMat4, AiVec3,
    structs::{
        material::{property::AiProperty, texture_property::AiTextureType},
        memory::AiMemoryInfo,
        mesh::primitive_type::AiPrimitiveType,
        node::AiNode,
        scene::AiScene,
    },
};

use crate::error::AssimpCmdError;

const TREE_BRANCH: &str = "├╴";
const TREE_STOP: &str = "└╴";
const TREE_CONTINUE: &str = "│ ";

pub fn count_nodes(root: &AiNode, nodes: &[AiNode]) -> usize {
    let mut i = 0;
    for a in &root.children {
        i += count_nodes(&nodes[a.value()], nodes);
    }
    1 + i
}

pub fn get_max_depth(root: &AiNode, nodes: &[AiNode]) -> usize {
    let mut cnt = 0;
    for i in &root.children {
        cnt = cnt.max(get_max_depth(&nodes[i.value()], nodes));
    }
    cnt + 1
}

pub fn count_vertices(scene: &AiScene) -> usize {
    let mut cnt = 0;
    for mesh in &scene.meshes {
        cnt += mesh.vertices.len();
    }
    cnt
}

pub fn count_faces(scene: &AiScene) -> usize {
    let mut cnt = 0;
    for mesh in &scene.meshes {
        cnt += mesh.faces.len();
    }
    cnt
}

pub fn count_bones(scene: &AiScene) -> usize {
    let mut cnt = 0;
    for mesh in &scene.meshes {
        cnt += mesh.bones.len();
    }
    cnt
}

pub fn count_anim_channels(scene: &AiScene) -> usize {
    let mut cnt = 0;
    for animation in &scene.animations {
        cnt += animation.channels.len();
    }
    cnt
}

pub fn get_avg_face_per_mesh(scene: &AiScene) -> usize {
    if !scene.meshes.is_empty() {
        count_faces(scene) / scene.meshes.len()
    } else {
        0
    }
}

pub fn get_avg_verts_per_mesh(scene: &AiScene) -> usize {
    if !scene.meshes.is_empty() {
        count_vertices(scene) / scene.meshes.len()
    } else {
        0
    }
}

pub fn find_special_points(scene: &AiScene, special_points: &mut [AiVec3; 3]) {
    fn find_special_points_inner(
        scene: &AiScene,
        root: &AiNode,
        special_points: &mut [AiVec3; 3],
        mat: &AiMat4,
    ) {
        // XXX that could be greatly simplified by using code from code/ProcessHelper.h
        // XXX I just don't want to include it here.
        let trafo = root.transformation * *mat;
        for i in 0..root.meshes.len() {
            let mesh = &scene.meshes[root.meshes[i] as usize];

            for a in 0..mesh.vertices.len() {
                let v = trafo.transform_point3(mesh.vertices[a]);
                special_points[0] = special_points[0].min(v);
                special_points[1] = special_points[1].max(v);
            }
        }

        for i in 0..root.children.len() {
            find_special_points_inner(
                scene,
                &scene.nodes[root.children[i].value()],
                special_points,
                &trafo,
            );
        }
    }
    special_points[0] = AiVec3::new(1e10, 1e10, 1e10);
    special_points[1] = AiVec3::new(-1e10, -1e10, -1e10);

    find_special_points_inner(scene, &scene.nodes[0], special_points, &AiMat4::IDENTITY);
    special_points[2] = (special_points[0] + special_points[1]) * 0.5;
}

pub fn find_ptypes(scene: &AiScene) -> String {
    let mut have_it = [false; 4];
    for mesh in &scene.meshes {
        let pt = mesh.primitive_types;
        have_it = [
            pt.contains(AiPrimitiveType::Point),
            pt.contains(AiPrimitiveType::Line),
            pt.contains(AiPrimitiveType::Triangle),
            pt.contains(AiPrimitiveType::Polygon),
        ];
    }
    [
        if have_it[0] { "points" } else { "" },
        if have_it[1] { "lines" } else { "" },
        if have_it[2] { "triangles" } else { "" },
        if have_it[3] { "n-polygons" } else { "" },
    ]
    .join("")
}

pub fn print_hierarchy(
    node: &AiNode,
    nodes: &[AiNode],
    indent: &str,
    verbose: bool,
    last: Option<bool>,
    first: Option<bool>,
) {
    let last = last.unwrap_or(false);
    let first = first.unwrap_or(true);
    // tree visualization
    let branchchar = if first {
        ""
    } else if last {
        TREE_STOP
    } else {
        TREE_BRANCH
    };

    // print the indent and the branch character and the name
    println!("{indent}{branchchar}{}", node.name);

    // if there are meshes attached, indicate this
    if !node.meshes.is_empty() {
        print!(" (mesh ");
        let mut sep = false;
        for i in 0..node.meshes.len() {
            let mesh_index = node.meshes[i];
            if sep {
                print!(", ");
            }
            print!("{mesh_index}");
            sep = true;
        }
        print!(")");
    }

    // finish the line
    println!();

    // in verbose mode, print the transform data as well
    if verbose {
        // indent to use
        let mut indentadd = String::new();
        if last {
            indentadd += "  ";
        } else {
            indentadd += TREE_CONTINUE;
        } // "| "..
        if node.children.is_empty() {
            indentadd += "  ";
        } else {
            indentadd += TREE_CONTINUE;
        } // .."| "
        let (s, r, t) = node.transformation.to_scale_rotation_translation();
        if s.x != 1.0 || s.y != 1.0 || s.z != 1.0 {
            print!("{indent}{indentadd}");
            print!("  S:[{} {} {}]\n", s.x, s.y, s.z);
        }
        if r.x != 0.0 || r.y != 0.0 || r.z != 0.0 {
            print!("{indent}{indentadd}");
            print!("  R:[{} {} {}]\n", r.x, r.y, r.z);
        }
        if t.x != 0.0 || t.y != 0.0 || t.z != 0.0 {
            print!("{indent}{indentadd}");
            print!("  T:[{} {} {}]\n", t.x, t.y, t.z);
        }
    }

    // and recurse
    let next_indent = if first {
        indent.to_owned()
    } else if last {
        indent.to_owned() + "  "
    } else {
        indent.to_owned() + TREE_CONTINUE
    };

    for (i, index) in node.children.iter().enumerate() {
        print_hierarchy(
            &nodes[index.value()],
            nodes,
            &next_indent,
            verbose,
            Some(i == node.children.len() - 1),
            Some(false),
        );
    }
}

const AICMD_MSG_INFO_HELP_E: &str = r#"
assimp info <file> [-r] [-v]\n
\tPrint basic structure of a 3D model\n
\t-r,--raw: No postprocessing, do a raw import\n
\t-v,--verbose: Print verbose info such as node transform data\n
\t-s, --silent: Print only minimal info"#;

pub fn assimp_info(params: &[&str], num: usize) -> Result<(), AssimpCmdError> {
    // asssimp info <file> [-r]
    if num < 1 {
        println!("assimp info: Invalid number of arguments.\nSee \'assimp info --help\'");
        return Err(AssimpCmdError::InvalidNumberOfArguments);
    }

    // --help
    if matches!(params[0], "-h" | "--help" | "-?") {
        println!("{AICMD_MSG_INFO_HELP_E}");
        return Ok(());
    }

    // const std::string in = std::string(params[0]);

    // // get -r and -v arguments
    let mut raw = false;
    let mut verbose = false;
    let mut silent = false;
    for &param in params.iter().skip(1) {
        if matches!(param, "--raw" | "-r") {
            raw = true;
        }
        if matches!(param, "--verbose" | "-v") {
            verbose = true;
        }
        if matches!(param, "--silent" | "-s") {
            silent = true;
        }
    }

    // Verbose and silent at the same time are not allowed
    if verbose && silent {
        println!(
            "assimp info: Invalid arguments, verbose and silent at the same time are forbidden. "
        );
        return Err(AssimpCmdError::InvalidCombinaisonOfArguments);
    }

    // // Parse post-processing flags unless -r was specified
    // ImportData import;
    // if (!raw) {
    //     // get import flags
    //     ProcessStandardArguments(import, params + 1, num - 1);

    //     //No custom post process flags defined, we set all the post process flags active
    //     if (import.ppFlags == 0)
    //         import.ppFlags |= aiProcessPreset_TargetRealtime_MaxQuality;
    // }

    // // import the main model
    // const aiScene *scene = ImportModel(import, in);
    // if (!scene) {
    //     printf("assimp info: Unable to load input file %s\n",
    //             in.c_str());
    //     return AssimpCmdError::FailedToLoadInputFile;
    // }
    let scene = AiScene::default();

    let mem = scene.get_memory_requirements();

    let mut special_points = [AiVec3::default(); 3];
    find_special_points(&scene, &mut special_points);
    #[rustfmt::skip]
    println!(
        concat!(
            "Memory consumption: {} B\n",
            "Nodes:              {}\n",
            "Maximum depth       {}\n",
            "Meshes:             {}\n",
            "Animations:         {}\n",
            "Textures (embed.):  {}\n",
            "Materials:          {}\n",
            "Cameras:            {}\n",
            "Lights:             {}\n",
            "Vertices:           {}\n",
            "Faces:              {}\n",
            "Bones:              {}\n",
            "Animation Channels: {}\n",
            "Primitive Types:    {}\n",
            "Average faces/mesh  {}\n",
            "Average verts/mesh  {}\n",
            "Minimum point      ({} {} {})\n",
            "Maximum point      ({} {} {})\n",
            "Center point       ({} {} {})"
        ),
        mem.total,
        count_nodes(&scene.nodes[0], &scene.nodes),
        get_max_depth(&scene.nodes[0], &scene.nodes),
        scene.meshes.len(),
        scene.animations.len(),
        scene.textures.len(),
        scene.materials.len(),
        scene.cameras.len(),
        scene.lights.len(),
        count_vertices(&scene),
        count_faces(&scene),
        count_bones(&scene),
        count_anim_channels(&scene),
        find_ptypes(&scene),
        get_avg_face_per_mesh(&scene),
        get_avg_verts_per_mesh(&scene),
        special_points[0].x, special_points[0].y, special_points[0].z,
        special_points[1].x, special_points[1].y, special_points[1].z,
        special_points[2].x, special_points[2].y, special_points[2].z,
    );

    if silent {
        println!();
        return Ok(());
    }

    // meshes
    if !scene.meshes.is_empty() {
        println!("\nMeshes:  (name) [vertices / bones / faces | primitive_types]");
    }
    for (i, mesh) in scene.meshes.iter().enumerate() {
        println!("    {} ({})", i, mesh.name);
        print!(
            ": [{} / {} / {} |",
            mesh.vertices.len(),
            mesh.bones.len(),
            mesh.faces.len()
        );
        let ptypes = mesh.primitive_types;
        if ptypes.contains(AiPrimitiveType::Point) {
            print!(" point");
        }
        if ptypes.contains(AiPrimitiveType::Line) {
            print!(" line");
        }
        if ptypes.contains(AiPrimitiveType::Triangle) {
            print!(" triangle");
        }
        if ptypes.contains(AiPrimitiveType::Polygon) {
            print!(" polygon");
        }
        println!("]");
    }

    // materials
    if !scene.materials.is_empty() {
        println!("\nNamed Materials:");
    }
    for mat in &scene.materials {
        let name = mat.get_name().unwrap_or_default();
        println!("\n    \'{}\'", name);
        if !mat.properties.is_empty() {
            println!(" (prop) [index / bytes | texture semantic]");
        }
        for (i, prop) in mat.properties.iter().enumerate() {
            let textype = prop.r#type;
            println!(
                "\n        {} ({:?}): [{} | {}]",
                i,
                prop.property,
                prop.index,
                textype.as_str()
            );
        }
    }
    if !scene.materials.is_empty() {
        println!();
    }

    // textures
    let mut total = 0;
    for mat in &scene.materials {
        let types: [(AiTextureType, fn(&AiProperty) -> Option<&str>); 12] = [
            (AiTextureType::None, AiProperty::is_texture_file_property),
            (
                AiTextureType::Diffuse,
                AiProperty::is_texture_diffuse_property,
            ),
            (
                AiTextureType::Specular,
                AiProperty::is_texture_specular_property,
            ),
            (
                AiTextureType::Ambient,
                AiProperty::is_texture_ambient_property,
            ),
            (
                AiTextureType::Emissive,
                AiProperty::is_texture_emissive_property,
            ),
            (
                AiTextureType::Height,
                AiProperty::is_texture_height_property,
            ),
            (
                AiTextureType::Normals,
                AiProperty::is_texture_normals_property,
            ),
            (
                AiTextureType::Shininess,
                AiProperty::is_texture_shininess_property,
            ),
            (
                AiTextureType::Opacity,
                AiProperty::is_texture_opacity_property,
            ),
            (
                AiTextureType::Displacement,
                AiProperty::is_texture_displacement_property,
            ),
            (
                AiTextureType::Lightmap,
                AiProperty::is_texture_lightmap_property,
            ),
            (
                AiTextureType::Reflection,
                AiProperty::is_texture_reflection_property,
            ),
            // AiProperty::is_texture_base_color_property,
            // AiProperty::is_texture_normal_camera_property,
            // AiProperty::is_texture_emission_color_property,
            // AiProperty::is_texture_metalness_property,
            // AiProperty::is_texture_diffuse_roughness_property,
            // AiProperty::is_texture_ambient_occlusion_property,
            // AiProperty::is_texture_unknown_property,
        ];
        for (r#type, type_match_fn) in types {
            let mut idx = 0;
            while let Some(name) = mat.get_property_by_texture_type(idx, r#type, type_match_fn) {
                print!(
                    "{}\n    \'{name}\'",
                    if total > 0 { "" } else { "\nTexture Refs:" },
                );
                total += 1;
                idx += 1;
            }
        }
    }
    if total > 0 {
        println!();
    }

    // // animations
    total = 0;
    for anim in &scene.animations {
        if !anim.name.is_empty() {
            print!(
                "{}\n     \'{}\'",
                if total > 0 { "" } else { "\nNamed Animations:" },
                anim.name
            );
            total += 1;
        }
    }
    if total > 0 {
        println!();
    }

    // node hierarchy
    println!("\nNode hierarchy:");
    print_hierarchy(&scene.nodes[0], &scene.nodes, "", verbose, None, None);

    println!();
    Ok(())
}
