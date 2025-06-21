extern crate embed_resource;

use bevy_ldtk_scene::prelude::*;
use bevy_ldtk_scene::world::ExtractLdtkWorld;
use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        // on windows we will set our game icon as icon for the executable
        embed_resource::compile("build/windows/icon.rc");
    }

    println!("cargo::rerun-if-changed=assets/ldtk/time-marches-on.ldtk");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let world = ExtractLdtkWorld::new("assets/ldtk/time-marches-on.ldtk")
        .unwrap()
        .extract_with((
            ExtractComposites,
            ExtractEnums,
            ExtractEntityTypes,
            ExtractTileSets,
            ExtractCompEntities,
            ExtractEntityInstances,
            ExtractLevelUids,
        ))
        .unwrap()
        .write(PathBuf::new().join(out_dir).join("world.rs"))
        .unwrap()
        .write(PathBuf::new().join("ldtk_out.rs"))
        .unwrap()
        .world();
    world.save("assets/ldtk/time-marches-on.ron").unwrap();
}
