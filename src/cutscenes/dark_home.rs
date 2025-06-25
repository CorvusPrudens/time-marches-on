use crate::cutscene::{
    chara::{Chara, Character},
    fragments::IntoBox,
};
use bevy_sequence::prelude::*;

pub fn sturgeon() -> impl IntoBox {
    (
        "how many times".chara(Chara::Sturgeon),
        "will you tell her".chara(Chara::Sturgeon),
        "how many times".chara(Chara::Sturgeon),
        "will you".chara(Chara::Sturgeon),
    )
        .always()
        .once()
}

pub fn shadow_1() -> impl IntoBox {
    "you built a cage".shadow().always().once()
}

pub fn shadow_2() -> impl IntoBox {
    "do you need more painkillers yet?".shadow().always().once()
}

pub fn shadow_3() -> impl IntoBox {
    "who is she, anyway".shadow().always().once()
}

pub fn shadow_4() -> impl IntoBox {
    "you'll be all alone".shadow().always().once()
}

pub fn shadow_5() -> impl IntoBox {
    "alone".shadow().always().once()
}

pub fn shadow_6() -> impl IntoBox {
    "it won't be long, now".shadow().always().once()
}

pub fn shadow_7() -> impl IntoBox {
    "die already".shadow().always().once()
}

pub fn shadow_8() -> impl IntoBox {
    "oh god, the smell".shadow().always().once()
}
