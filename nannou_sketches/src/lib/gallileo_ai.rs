use rapier3d::{na::Quaternion, prelude::*};
use std::f32::consts::PI;



//trinitrocellulose has an enthalpy of combusion of 2.2kcal/g
//typical loads for a 9mm are around 1g of nitrocellulose. 
//2.2kcal = 9.2004 Kj / g

fn main() {
    // gun parameters
    let barrel_length = 0.5;
    let bore_diameter = 0.005; 
    let bullet_diameter = 0.0049;
    let bullets = [(0.001, 0.004)]; // (kg of powder, kg of bullet) 
    let burn_constant = 1; 
    let bullet_spacing = 0.02;
    let ground_height = 0.0;


    // Initialize the physics world.
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut gravity = Vector::new(0.0, -9.81, 0.0);
    let mut integration_parameters = IntegrationParameters::default();
    let mut broad_phase = BroadPhaseBvh::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joints = ImpulseJointSet::new();
    let mut multibody_joints = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let mut colliders = ColliderSet::new();
    let mut bodies = RigidBodySet::new();

    let ground_size = 1000.0;
    // Create a ground plane
    let ground_body = RigidBodyBuilder::fixed().translation(vector![0.0, - 0.1 / 2.0 + ground_height, 0.0]);
    let ground_handle = bodies.insert(ground_body);
    let ground_collider = ColliderBuilder::cuboid(ground_size, 0.1, ground_size).friction(0.1);
    colliders.insert_with_parent(ground_collider, ground_handle, &mut bodies);

    // Create a cannon barrel
    let barrel = RigidBodyBuilder::fixed().translation(vector![0.0, ground_height + barrel_length / 2.0, 0.0]);
    let barrel_collider = ColliderBuilder::compound(
                        vec![(Isometry::translation(0.0, 0.0, bore_diameter/2.0),  
                            SharedShape::cuboid(bore_diameter/2.0, barrel_length/2.0, bore_diameter/2.0)),
                            (Isometry::translation(0.0, 0.0, -bore_diameter/2.0),  
                            SharedShape::cuboid(bore_diameter/2.0, barrel_length/2.0, bore_diameter/2.0)),
                            (Isometry::translation(bore_diameter/2.0, 0.0, 0.0),  
                            SharedShape::cuboid(bore_diameter/2.0, barrel_length/2.0, bore_diameter/2.0)),
                            (Isometry::translation(-bore_diameter/2.0, 0.0, 0.0),  
                            SharedShape::cuboid(bore_diameter/2.0, barrel_length/2.0, bore_diameter/2.0)),
                            (Isometry::translation(0.0, -bore_diameter/2.0 - barrel_length/2.0, 0.0),  
                            SharedShape::cuboid(bore_diameter/2.0, bore_diameter/2.0, bore_diameter/2.0))]);
    let barrel_handle = bodies.insert(barrel);
    colliders.insert_with_parent(barrel_collider, barrel_handle, &mut bodies);


    let mut bullet_handles = vec![];
    bullets.iter().enumerate().for_each(|(i,bullet)| 
    {
        let bullet_height = (bullet_spacing + bore_diameter/2.0) * ((i+1) as f32);
        let bullet_handle = bodies.insert(RigidBodyBuilder::dynamic().translation(vector![0.0, bullet_height, 0.0])); 
        let bullet_collider = ColliderBuilder::ball(bore_diameter/2.0).mass(bullet.1);
        colliders.insert_with_parent(bullet_collider,bullet_handle,&mut bodies)
        bullet_handles.push(bullet_handle);
    });

    // Cannon properties
    let powder_force = Vector::new(0.0, 200.0, 0.0); // Force applied by the powder

    // Apply powder force to the cannonball
    bullet_handles.iter().zip(bullets).for_each(|(handle, (powder,mass))| 
    {
        let simple_velocity = f32::sqrt(2.0*mass*9204.0*powder);
        bodies.get(*handle).unwrap().apply_impulse(vector![0.0,simple_velocity,0.0], true);
    });

    // // Calculate reaction force for the barrel (Newton's third law)
    // let reaction_force = -powder_force;
    // bodies.apply_impulse(barrel_handle, reaction_force, true);

    // Simulation loop
    for _ in 0..100 {
        physics_pipeline.step(
            &mut gravity,
            &mut integration_parameters,
            &mut broad_phase,
            &mut narrow_phase,
            &mut impulse_joints,
            &mut multibody_joints,
            &mut ccd_solver,
            &mut bodies,
            &mut colliders,
            true,
        );

        // Print the position of the cannonball
        let ball_pos = bodies.get(ball_handle).unwrap().translation();
        println!("Cannonball Position: {:?}", ball_pos);
    }
}