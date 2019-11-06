use super::{InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum DistanceAlgorithm { Pythagoras, Manhattan, Chebyshev }

pub struct VoronoiBuilder {
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm
}

impl InitialMapBuilder for VoronoiBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl VoronoiBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<VoronoiBuilder> {
        Box::new(VoronoiBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }

    //premade constructors
    #[allow(dead_code)]
    pub fn pythagoras() -> Box<VoronoiBuilder> {
        Box::new(VoronoiBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }
    
    #[allow(dead_code)]
    pub fn manhattan() -> Box<VoronoiBuilder> {
        Box::new(VoronoiBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Manhattan,
        })
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {

        // Make a Voronoi diagram. We'll do this the hard way to learn about the technique!
        let mut voronoi_seeds : Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, build_data.map.width-1);
            let vy = rng.roll_dice(1, build_data.map.height-1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }    

        //Determine a cell's Voronoi membership (defined as the seed to which it's the closest)
        let mut voronoi_distance = vec![(0, 0.0f32) ; self.n_seeds];
        let mut voronoi_membership : Vec<i32> = vec![0 ; build_data.map.width as usize * build_data.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % build_data.map.width;
            let y = i as i32 / build_data.map.width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance;
                match self.distance_algorithm {           
                    DistanceAlgorithm::Pythagoras => {
                        distance = rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Manhattan => {
                        distance = rltk::DistanceAlg::Manhattan.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Chebyshev => {
                        distance = rltk::DistanceAlg::Chebyshev.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                }
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voronoi_distance[0].0 as i32;
        }

        //draw map
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let mut neighbors = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[build_data.map.xy_idx(x-1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x+1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y-1)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y+1)] != my_seed { neighbors += 1; }

                if neighbors < 2 {
                    build_data.map.tiles[my_idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        } 

        //starting point handled by area_starting_points.rs
        // spawning handled by voronoi_spawning.rs

        // // Find a starting point; start at the middle and walk left until we find an open tile
        // self.starting_position = Position{ x: self.map.width / 2, y : self.map.height / 2 };
        // let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        // while self.map.tiles[start_idx] != TileType::Floor {
        //     self.starting_position.x -= 1;
        //     start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        // }
        // self.take_snapshot();


        // // Now we build a noise map for use in spawning entities later
        // self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        // // Spawn the entities
        // for area in self.noise_areas.iter() {
        //     spawner::spawn_region(&self.map, &mut rng, area.1, &mut self.list_spawns);
        // }
    }
}