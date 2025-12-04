struct Powder
{
    mass:     f64,
    volume:   f64,
    x:        f64,
    velocity: f64,
}

struct Gas
{
    pressure: f64,
    volume:   f64,
    n:        f64,
    x:        f64,
}

struct Projectile
{
    mass:     f64,
    velocity: f64,
}

enum LoadComponent
{
    Powder(Powder),
    Gas(Gas),
    Projectile(Projectile),
}

struct Barrel
{
    length:       f64,
    max_pressure: f64,
    load:         Vec<LoadComponent>,
}

struct Simulation
{
    timestep: f64,
    max_time: f64,
    barrel:   Barrel,
}

trait Simulated
{
    fn start(&mut self, world: &Simulation) {}
    fn update(&mut self, world: &mut Simulation) {}
}

impl Simulated for Barrel
{
    fn start(&mut self, world: &Simulation)
    {
        let mut i = 0;
        while let Some(ref lc) = self.load.get(i)
        {
            if let LoadComponent::Powder(p) = lc
            {
                self.load.insert(i,
                                 LoadComponent::Gas(Gas { pressure: 0.0,
                                                          volume:   0.0,
                                                          n:        0.0,
                                                          x:        p.x, }));
            }
            i += 1;
        }
    }

    fn update(&mut self, world: &mut Simulation) {}
}
