// A click on button `button` of the mod's settings page.
fn vulkanmod_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::port_vulkanmod as m;
    if mod_id != m::ID {
        return;
    }
    // TODO: change the option behind `button` and keep it: game.native_set_setting(m::ID, key, value).
    let _ = (game, button);
}
