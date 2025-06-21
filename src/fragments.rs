use bevy::prelude::*;
use bevy_sequence::prelude::*;

#[derive(Component, Clone)]
pub struct Dialogue(pub String);

impl IntoFragment<Dialogue> for &'static str {
    fn into_fragment(self, context: &Context<()>, commands: &mut Commands) -> FragmentId {
        <_ as IntoFragment<Dialogue>>::into_fragment(
            bevy_sequence::fragment::DataLeaf::new(Dialogue(self.into())),
            context,
            commands,
        )
    }
}
