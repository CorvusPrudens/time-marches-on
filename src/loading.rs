use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(debug_assertions))]
        let cont = GameState::Menu;

        #[cfg(debug_assertions)]
        let cont = GameState::Playing;

        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(cont)
                .load_collection::<TextureAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    //#[asset(path = "textures/bevy.png")]
    //pub bevy: Handle<Image>,
    //#[asset(path = "textures/github.png")]
    //pub github: Handle<Image>,
}
