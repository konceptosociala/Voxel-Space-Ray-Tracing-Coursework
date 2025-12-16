use std::collections::HashSet;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        eprintln!("Usage: {} <input> <output>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let model = dot_vox::load(path)
        .map_err(|e| anyhow::format_err!("MagicaVoxel model loading error: {e}"))?;

    let mut result = String::from("\\begin{tikzpicture}\n");

    let palette = model.palette.clone();
    let voxels = model.models[0].voxels.clone();

    let mut colors = HashSet::new();
    for voxel in &voxels {
        colors.insert(voxel.i);
    }
    let mut colors: Vec<u8> = colors.into_iter().collect();
    colors.sort_unstable();

    for (i, color) in colors.iter().enumerate() {
        let c = palette[*color as usize];
        result.push_str(&format!(
            "\\definecolor{{color{}}}{{HTML}}{{{:02X}{:02X}{:02X}}}\n",
            i, c.r, c.g, c.b
        ));
    }

    let mut new_voxels = Vec::new();

    for x in 0..model.models[0].size.x {
        for z in 0..model.models[0].size.z {
            for y in 0..model.models[0].size.y {
                if let Some(voxel) = voxels
                    .iter()
                    .find(|v| v.x == x as u8 && v.y == y as u8 && v.z == z as u8)
                {
                    new_voxels.push(*voxel);
                } else {
                    new_voxels.push(dot_vox::Voxel {
                        x: x as u8,
                        y: y as u8,
                        z: z as u8,
                        i: 255, // Transparent voxel
                    });
                }
            }
        }
    }

    new_voxels.sort_by_key(|v| ((v.z as i32), -(v.y as i32), (v.x as i32)));

    for voxel in new_voxels {
        if voxel.i == 255 {
            result.push_str(&format!("\\cube{{{}}}{{{}}}{{{}}}{{1}}\n", voxel.x, voxel.y, voxel.z));
            continue;
        }

        result.push_str(&format!(
            "\\cubesurface{{{}}}{{{}}}{{{}}}{{1}}{{color{}}}\n",
            voxel.x,
            voxel.y,
            voxel.z,
            colors.iter().position(|&c| c == voxel.i).unwrap(),
        ));
    }

    result.push_str("\\end{tikzpicture}\n");
    
    let output = &args[2];

    let file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(file);
    use std::io::Write;
    writer.write_all(result.as_bytes())?;

    println!("Successfully converted .vox to .tex at: {}", output);

    Ok(())
}
