// A map's slot (hotbar, inventory, containers): its picture over the plain
// map icon, when the client has the map.
fn map_preview_slot(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    x: f32,
    y: f32,
    batch: &mut horizon_client::hud::HudBatch,
) {
    use crate::game::map_preview as m;
    if !game.native_on(m::ID) {
        return;
    }
    let Some(id) = crate::game::maps::map_id(stack) else {
        return;
    };
    let settings = m::settings(|key| game.native_setting(m::ID, key));
    if settings.get(m::o::SLOT) == 0 {
        return;
    }
    let Some(map) = game.maps.data.get(&id) else {
        return;
    };

    // The slot's item count text was painted immediately before this hook.
    // Pop trailing text vertices within the slot so the map preview draws behind them.
    let mut split_at = batch.vertices.len();
    while split_at > 0 {
        let v = &batch.vertices[split_at - 1];
        if v.text == 1
            && v.pos[0] >= x - 2.0
            && v.pos[0] <= x + 24.0
            && v.pos[1] >= y - 2.0
            && v.pos[1] <= y + 24.0
        {
            split_at -= 1;
        } else {
            break;
        }
    }
    let decorations = batch.vertices.split_off(split_at);

    let style = settings.get(m::o::SLOT_STYLE);
    let high_res = settings.get(m::o::SLOT_RES) == 1;
    let locked_glass = settings.get(m::o::LOCKED_GLASS) == 1;
    let is_locked = map.locked || stack.data().and_then(|d| d.map_post_processing) == Some(0);
    let want_locked = is_locked && locked_glass;
    let palette = &horizon_client::blocks::MAP_COLORS;

    let (item_rects, full_rects) = if style == 2 || want_locked {
        m::slot_overlays_for_revision(game.gui.atlas.revision, [x, y], want_locked, |emit| {
            let atlas = &game.gui.atlas;
            let pixels = atlas.pixels();
            let sprite_rgba = |name: &str| -> Option<(Vec<u8>, u32, u32)> {
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
            let map_sp = sprite_rgba("item/map");
            let filled_sp = sprite_rgba("item/filled_map");
            let glass_sp = sprite_rgba("block/glass");
            emit(m::FrameSprites {
                map: map_sp.as_ref().map(|(px, w, h)| (px.as_slice(), *w, *h)),
                filled: filled_sp.as_ref().map(|(px, w, h)| (px.as_slice(), *w, *h)),
                glass: glass_sp.as_ref().map(|(px, w, h)| (px.as_slice(), *w, *h)),
            });
        })
    } else {
        (Vec::new(), Vec::new())
    };

    if style == 2 && !item_rects.is_empty() {
        let (paper_quad, paper_color) = m::tilted_paper_quad([x, y]);
        game.native_quad(batch, paper_quad, paper_color);

        let cells = if high_res { m::MAP_SIZE } else { 13 };
        for (quad, argb) in m::tilted_picture(&map.colors, palette, [x, y], cells) {
            game.native_quad(batch, quad, argb);
        }

        for (rect, argb) in item_rects {
            game.native_fill(batch, rect, argb);
        }
    } else {
        let edged = style == 1 || style == 2;
        let paper = settings.get(m::o::PAPER) != 0;
        for (rect, argb) in m::slot_rects(&map.colors, palette, [x, y], edged, paper, high_res) {
            game.native_fill(batch, rect, argb);
        }
        for (rect, argb) in full_rects {
            game.native_fill(batch, rect, argb);
        }
    }

    batch.vertices.extend(decorations);
}
