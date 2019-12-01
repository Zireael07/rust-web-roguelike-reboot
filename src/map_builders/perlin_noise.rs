use super::{InitialMapBuilder, BuilderMap, TileType, apply_paint};
use rltk::RandomNumberGenerator;
use rltk::{NoiseType, FractalType};

pub struct NoiseMapBuilder {}

impl InitialMapBuilder for NoiseMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl NoiseMapBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<NoiseMapBuilder> {
        Box::new(NoiseMapBuilder{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        //generate noise
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(NoiseType::SimplexFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(5);
        noise.set_fractal_gain(0.6);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(2.0);

        //draw map
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let n = noise.get_noise((x as f32) / ((build_data.map.width*2) as f32), (y as f32) / ((build_data.map.height*2) as f32));
                let idx = ((y * build_data.map.width) + x) as usize;
                if n < 0.0 {
                    apply_paint(&mut build_data.map, 2, x, y, TileType::Floor);
                    //build_data.map.tiles[idx] = TileType::Floor;
                    //self.colors[idx] = RGB::from_f32(0.0, 0.0, 1.0 - (0.0 - n));
                } else {
                    apply_paint(&mut build_data.map, 2, x, y, TileType::Tree);
                    //build_data.map.tiles[idx] = TileType::Tree;
                    //self.colors[idx] = RGB::from_f32(0.0, n, 0.0);
                }
            }
        }
        build_data.take_snapshot();
    }

}