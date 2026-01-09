// From bevy_panorbit_camera
use bevy::prelude::*;

/// A resource that tracks whether egui wants focus on the current and previous
/// frames.
///
/// The reason the previous frame's value is saved is because when you click
/// inside an egui window, Context::wants_pointer_input() still returns false
/// once before returning true. If the camera stops taking input only when it
/// returns false, there's one frame where both egui and the camera are using
/// the input events, which is not desirable.
///
/// This is re-exported in case it's useful. I recommend only using input
/// events if both `prev` and `curr` are false.
#[derive(Resource, PartialEq, Eq, Default, Debug)]
pub struct EguiWantsFocus {
    /// Whether egui wanted focus on the previous frame
    pub prev: bool,
    /// Whether egui wants focus on the current frame
    pub curr: bool,
}

pub(crate) fn check_egui_wants_focus(
    contexts: Option<bevy_egui::EguiContexts>,
    mut wants_focus: ResMut<EguiWantsFocus>,
    windows: Query<Entity, With<Window>>,
) {
    // If EguiPlugin is not added, early return with default state
    let Some(mut contexts) = contexts else {
        return;
    };
    // The window that the user is interacting with and the window that
    // contains the egui context that the user is interacting with are always
    // going to be the same. Therefore, we can assume that if any of the egui
    // contexts want focus, then it must be the one that the user is
    // interacting with.
    let new_wants_focus = windows.iter().any(|window| {
        if let Ok(ctx) = contexts.ctx_for_entity_mut(window) {
            ctx.wants_pointer_input()
                || ctx.wants_keyboard_input()
                || ctx.is_pointer_over_area()
        } else {
            false
        }
    });
    let new_res = EguiWantsFocus {
        prev: wants_focus.curr,
        curr: new_wants_focus,
    };
    trace!("Egui want focus: {new_res:?}");
    wants_focus.set_if_neq(new_res);
}
