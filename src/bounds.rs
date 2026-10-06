//! Fixed view fitting only: never reposition from the current animation frame.
use bevy::prelude::*;
use bevy_flash::{
    sampling::VabSkin,
    vab_asset::{CommandList, VabAsset, VabCommand},
};

pub fn measure(asset: &VabAsset, skin: &VabSkin) -> Result<(Vec2, Vec2), String> {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    // Union all clips and frames once. Filter outputs contribute expanded bounds.
    for (clip, data) in asset.baked.clips.iter().enumerate() {
        for frame in 0..data.frames.len() {
            let commands = asset
                .sample(clip, frame, skin, Vec3::ONE)
                .map_err(|e| e.to_string())?;
            accumulate(asset, &commands, &mut min, &mut max);
        }
    }
    if !min.is_finite() || !max.is_finite() {
        return Err("资源没有可显示的图形".into());
    }
    Ok(((min + max) * 0.5, (max - min).max(Vec2::ONE)))
}

fn accumulate(asset: &VabAsset, commands: &CommandList, min: &mut Vec2, max: &mut Vec2) {
    for command in &commands.commands {
        match command {
            VabCommand::RenderShape { handle, transform } => {
                let Some(mesh) = asset.render_meshes.get(*handle) else {
                    continue;
                };
                let [x0, y0, x1, y1] = mesh.local_bounds;
                let m = transform.matrix;
                for (x, y) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
                    let p = Vec2::new(m.a * x + m.c * y + m.tx, m.b * x + m.d * y + m.ty);
                    *min = min.min(p);
                    *max = max.max(p);
                }
            }
            VabCommand::ApplyFilter {
                bounds, filters, ..
            } => {
                let [x, y, w, h] = *bounds;
                let (x, y, w, h) = vatf::animation::filter_dest_rect(x, y, w, h, filters);
                *min = min.min(Vec2::new(x, y));
                *max = max.max(Vec2::new(x + w, y + h));
            }
            VabCommand::Blend(commands, _) => accumulate(asset, commands, min, max),
            _ => {}
        }
    }
}
