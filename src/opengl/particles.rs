#[derive(Clone, Debug)]
pub struct Particle { 
    pub position: (f32, f32),
    pub direction: (f32, f32),
    pub lifetime: f32,
}

impl Particle {
    fn new(x: f32) -> Self{
        Particle { 
            position: (x, -1.),
            direction: (0.5, 0.5),
            lifetime: 1.,
        }
    }
    
    fn update(&mut self, elapsed: f32) {
        self.position.0 += self.direction.0*elapsed;
        self.position.1 += self.direction.1*elapsed;
        self.lifetime -= elapsed;
    }
}

#[derive(Clone, Debug)]
pub struct Particles {
    pub particles: Vec<Particle>,
    pub particle_vert: Vec<f32>,
    pub particle_ind: Vec<u32>,
}

impl Particles {
    pub fn new() -> Self {
        Particles {
            particles: vec![],
            particle_vert: vec![],
            particle_ind: vec![],
        }
    }

    pub fn update(&mut self, elapsed: f32, note_vert: &Vec<f32>) {
        for p in self.particles.iter_mut() {
            p.update(elapsed)
        }
        self.particles.retain(|p| p.lifetime>0.);

        let mut i: usize = 0;
        while i < note_vert.len() {
            if note_vert[1]<(-1.) && note_vert[7]>(-1.) {
                self.particles.push(Particle::new((note_vert[0]+note_vert[6])/2.));
            }
            i+=24;
        }

        self.particles_to_vertices().unwrap();
    }

    pub fn particles_to_vertices(&mut self) -> std::io::Result<()>{
        self.particle_vert = vec![];
        self.particle_ind = vec![];
        for (i, p) in self.particles.iter().enumerate() {
            let ver2: Vec<f32> = vec![
                 //      x                 y        color  
                 p.position.0-0.01, p.position.1-0.01, 0.8,
                 p.position.0+0.01, p.position.1-0.01, 0.8,
                 p.position.0+0.01, p.position.1+0.01, 0.8,
                 p.position.0-0.01, p.position.1+0.01, 0.8,
            ];
            self.particle_vert.extend(ver2);

            let i2: u32 = i as u32;
            let ind2: Vec<u32> = vec![
                0+4*i2, 2+4*i2, 1+4*i2,
                0+4*i2, 2+4*i2, 3+4*i2,
            ];
            self.particle_ind.extend(ind2);
        }
        Ok(())
    }
}
