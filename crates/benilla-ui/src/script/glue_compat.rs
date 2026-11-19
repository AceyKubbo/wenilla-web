//! wenilla carry: the glue VM answers `_G`.
//!
//! 1.12.1 has no `_G` global: the reference's base array (`0x811e28`) omits it and an addon reaches
//! the globals by `getfenv(0)` — see the `globals` row of `benilla-ui/tests/vm_parity.rs`. The
//! in-game VM keeps that surface, and so do the parity gates. The glue screens, though, are where
//! a realm service supplies its own `RealmList.xml`, and a UI written for a later client spells the
//! globals `_G[this:GetName().."Status"]` in the realm button's `OnLoad`. There the missing binding
//! faults on line 2 with `attempt to index global '_G' (a nil value)`, the handler stops partway,
//! and the row's labels never get their anchors.
//!
//! Deviation: `_G` is a global the reference glue state does not have. It is published in the glue
//! VM only, so the in-game VM's 1.12 surface is untouched, and addons that feature-detect `_G` to
//! branch on the client version can still do so in the world. Kept in the fork because the
//! alternative is a broken realm list for any UI written past 1.12.

use mlua::Lua;

/// Publish `_G` as the glue VM's own globals table, once, at the VM's birth. A self-reference is a
/// cycle in the table, which Lua allows; `pairs(getfenv(0))` then answers it like any other global.
pub(super) fn install(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    globals.clone().set("_G", globals)
}

#[cfg(test)]
mod tests {
    use crate::script::UiScript;

    #[test]
    fn the_glue_vm_answers_g_and_the_in_game_vm_does_not() {
        let glue = UiScript::new_glue().unwrap();
        assert_eq!(
            glue.eval::<bool>("return _G == getfenv(0)").unwrap(),
            true,
            "the glue VM's `_G` is its globals table"
        );
        assert_eq!(
            glue.eval::<String>("return type(_G)").unwrap(),
            "table"
        );

        let world = UiScript::new().unwrap();
        assert_eq!(
            world.eval::<String>("return type(_G)").unwrap(),
            "nil",
            "the in-game VM keeps 1.12's surface"
        );
    }

    /// The reported fault: a GlueXML `<OnLoad>` indexing `_G` on its second line. With no alias
    /// the handler stops there and the load log records `attempt to index global '_G' (a nil value)`.
    #[test]
    fn a_glue_onload_indexing_g_runs_clean() {
        let glue = UiScript::new_glue().unwrap();
        let doc = crate::framexml::parse(
            r#"<Ui>
                <Frame name="RealmListRealmButton1">
                    <Scripts>
                        <OnLoad>
                            local name = this:GetName()
                            _G[name.."Loaded"] = 1
                        </OnLoad>
                    </Scripts>
                </Frame>
            </Ui>"#,
        )
        .expect("valid FrameXML");
        let report = crate::loader::load(&glue, &doc, &|_| None);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(
            glue.eval::<i64>("return RealmListRealmButton1Loaded").unwrap(),
            1,
            "the handler's second line ran"
        );
    }
}
