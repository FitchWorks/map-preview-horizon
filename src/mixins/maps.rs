// Each frame, before the HUD and the screens draw (`input_hud_begin`): the
// pictures of the maps the mod drew lately (`maps.data`, which the mod API
// doesn't show a client mod), the maps' palette, and, when the resource packs
// changed, the textures the slot's frame and glass are cut from.
fn map_preview_maps(game: &Game<'_>) {
    use mod_map_preview::preview as m;
    let atlas = &game.gui.atlas;
    if m::sprites_revision() != Some(atlas.revision) {
        let pixels = atlas.pixels();
        let sprite = |name: &str| -> Option<(Vec<u8>, u32, u32)> {
            let rect = atlas.sprites.get(name)?;
            let (tw, th) = rect.texels((atlas.width, atlas.height));
            if tw == 0 || th == 0 {
                return None;
            }
            let sx = (rect.u0 * atlas.width as f32).round() as usize;
            let sy = (rect.v0 * atlas.height as f32).round() as usize;
            let atlas_w = atlas.width as usize;
            let (w, h) = (tw as usize, th as usize);
            if sx.checked_add(w)? > atlas_w || sy.checked_add(h)? > atlas.height as usize {
                return None;
            }
            let mut out = Vec::with_capacity(w * h * 4);
            for row in 0..h {
                let start = ((sy + row) * atlas_w + sx) * 4;
                out.extend_from_slice(pixels.get(start..start + w * 4)?);
            }
            Some((out, tw, th))
        };
        let sprites = m::FrameSprites {
            map: sprite("item/map"),
            filled: sprite("item/filled_map"),
            glass: sprite("block/glass"),
        };
        let checker = atlas.sprites.get("map/map_background_checkerboard").map(|r| {
            let (w, h) = r.texels((atlas.width, atlas.height));
            [w as f32, h as f32]
        });
        m::set_sprites(atlas.revision, &sprites, checker);
    }
    let maps = m::take_wanted()
        .into_iter()
        .filter_map(|id| {
            let data = game.maps.data.get(&id)?;
            Some((id, m::MapPicture { colors: data.colors.clone(), locked: data.locked }))
        })
        .collect();
    m::set_maps(&horizon_client::blocks::MAP_COLORS, maps);
}
