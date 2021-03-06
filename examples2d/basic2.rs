extern crate nalgebra as na;

use na::{Isometry2, Point2, Point3, Vector2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::object::{
    BodyPartHandle, ColliderDesc, DefaultBodySet, DefaultColliderSet, Ground, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use nphysics_testbed2d::{objects::FluidRenderingMode, Testbed};
use salva2d::coupling::{ColliderCouplingSet, CouplingMethod};
use salva2d::object::{Boundary, Fluid};
use salva2d::solver::{ArtificialViscosity, IISPHSolver};
use salva2d::LiquidWorld;

pub fn init_world(testbed: &mut Testbed) {
    /*
     * World
     */
    let mechanical_world = DefaultMechanicalWorld::new(Vector2::new(0.0, -9.81));
    let geometrical_world = DefaultGeometricalWorld::new();
    let mut bodies = DefaultBodySet::new();
    let mut colliders = DefaultColliderSet::new();
    let joint_constraints = DefaultJointConstraintSet::new();
    let force_generators = DefaultForceGeneratorSet::new();

    /*
     * Liquid world.
     */
    let particle_rad = 0.1;
    let solver = IISPHSolver::<f32>::new();
    let mut liquid_world = LiquidWorld::new(solver, particle_rad, 2.0);
    let mut coupling_set = ColliderCouplingSet::new();

    // Liquid.
    let mut points1 = Vec::new();
    let mut points2 = Vec::new();
    let ni = 25;
    let nj = 15;

    let shift2 = (nj as f32) * particle_rad * 2.0;

    for i in 0..ni {
        for j in 0..nj {
            let x = (i as f32) * particle_rad * 2.0 - ni as f32 * particle_rad;
            let y = (j as f32 + 1.0) * particle_rad * 2.0;
            points1.push(Point2::new(x, y));
            points2.push(Point2::new(x, y + shift2));
        }
    }

    let viscosity = ArtificialViscosity::new(0.5, 0.0);
    let mut fluid = Fluid::new(points1, particle_rad, 1.0);
    fluid.nonpressure_forces.push(Box::new(viscosity.clone()));
    let fluid_handle = liquid_world.add_fluid(fluid);
    testbed.set_fluid_color(fluid_handle, Point3::new(0.8, 0.7, 1.0));

    let mut fluid = Fluid::new(points2, particle_rad, 1.0);
    fluid.nonpressure_forces.push(Box::new(viscosity.clone()));
    let fluid_handle = liquid_world.add_fluid(fluid);
    testbed.set_fluid_color(fluid_handle, Point3::new(0.6, 0.8, 0.5));

    /*
     *
     * Ground cuboid.
     *
     */

    let ground_size = 25.0;
    let ground_shape = ShapeHandle::new(Cuboid::new(Vector2::new(ground_size, 1.0)));

    let ground_handle = bodies.insert(Ground::new());
    let co = ColliderDesc::new(ground_shape.clone())
        .translation(-Vector2::y())
        .build(BodyPartHandle(ground_handle, 0));
    let co_handle = colliders.insert(co);
    let bo_handle = liquid_world.add_boundary(Boundary::new(Vec::new()));
    coupling_set.register_coupling(bo_handle, co_handle, CouplingMethod::DynamicContactSampling);

    let co = ColliderDesc::new(ground_shape.clone())
        .position(Isometry2::new(
            Vector2::x() * -5.0,
            std::f32::consts::FRAC_PI_2 + 0.1,
        ))
        .build(BodyPartHandle(ground_handle, 0));
    let co_handle = colliders.insert(co);
    let bo_handle = liquid_world.add_boundary(Boundary::new(Vec::new()));
    coupling_set.register_coupling(bo_handle, co_handle, CouplingMethod::DynamicContactSampling);

    let co = ColliderDesc::new(ground_shape)
        .position(Isometry2::new(
            Vector2::x() * 5.0,
            std::f32::consts::FRAC_PI_2 - 0.1,
        ))
        .build(BodyPartHandle(ground_handle, 0));
    let co_handle = colliders.insert(co);
    let bo_handle = liquid_world.add_boundary(Boundary::new(Vec::new()));
    coupling_set.register_coupling(bo_handle, co_handle, CouplingMethod::DynamicContactSampling);

    /*
     * Create a dynamic box.
     */
    let rad = 0.4;
    let cuboid = ShapeHandle::new(Cuboid::new(Vector2::repeat(rad)));
    let cuboid_sample =
        salva2d::sampling::shape_surface_ray_sample(&*cuboid, particle_rad).unwrap();

    // Build the rigid body.
    let rb = RigidBodyDesc::new()
        .translation(Vector2::y() * 10.0)
        .build();
    let rb_handle = bodies.insert(rb);
    testbed.set_body_color(rb_handle, Point3::new(0.3, 0.3, 0.7));

    // Build the collider.
    let co = ColliderDesc::new(cuboid)
        .density(0.9)
        .build(BodyPartHandle(rb_handle, 0));
    let co_handle = colliders.insert(co);
    let bo_handle = liquid_world.add_boundary(Boundary::new(Vec::new()));
    coupling_set.register_coupling(
        bo_handle,
        co_handle,
        CouplingMethod::StaticSampling(cuboid_sample),
    );

    /*
    /*
     * Create the deformable body and a collider for its boundary.
     */
    let mut deformable = FEMSurfaceDesc::quad(10, 1)
        .scale(Vector2::new(5.0, 0.5))
        .translation(Vector2::y() * 15.0)
        .young_modulus(500.0)
        .mass_damping(0.2)
        .density(1.0)
        .build();
    let collider_desc = deformable.boundary_collider_desc();
    let deformable_handle = bodies.insert(deformable);
    testbed.set_body_color(deformable_handle, Point3::new(0.2, 0.2, 0.2));

    let co = collider_desc.build(deformable_handle);
    let co_handle = colliders.insert(co);
    let bo_handle = liquid_world.add_boundary(Boundary::new(Vec::new()));
    coupling_set.register_coupling(bo_handle, co_handle, CouplingMethod::DynamicContactSampling);

    /*
     * Create a multibody.
     */
    let rad = 0.2;
    let num = 4;
    let body_shift = Vector2::x() * (rad * 2.5);
    let free = FreeJoint::new(Isometry2::translation(3.0, 20.0));

    let mut multibody_desc = MultibodyDesc::new(free);
    let mut curr = &mut multibody_desc;

    // Rotate the first joint so that the chain is vertical.
    let revo = RevoluteJoint::new(std::f32::consts::FRAC_PI_2);
    curr = curr.add_child(revo).set_body_shift(body_shift);

    for _ in 1usize..num {
        let revo = RevoluteJoint::new(0.0);
        curr = curr.add_child(revo).set_body_shift(body_shift);
    }

    let multibody = multibody_desc.build();
    let multibody_handle = bodies.insert(multibody);
    testbed.set_body_color(multibody_handle, Point3::new(0.7, 0.4, 0.5));

    // Create one collider for each link.
    let ball = ShapeHandle::new(Ball::new(rad));
    let collider_desc = ColliderDesc::new(ball.clone()).density(2.0);

    for i in 0..num + 1 {
        let co = collider_desc.build(BodyPartHandle(multibody_handle, i));
        let co_handle = colliders.insert(co);
        let bo_handle = liquid_world.add_boundary(Boundary::new(Vec::new()));
        coupling_set.register_coupling(
            bo_handle,
            co_handle,
            CouplingMethod::DynamicContactSampling,
        );
    }
    */

    /*
     * Set up the testbed.
     */
    testbed.set_ground_handle(Some(ground_handle));
    testbed.set_world(
        mechanical_world,
        geometrical_world,
        bodies,
        colliders,
        joint_constraints,
        force_generators,
    );
    testbed.set_liquid_world(liquid_world, coupling_set);
    testbed.look_at(Point2::new(0.0, 2.5), 95.0);
    testbed.set_fluid_rendering_mode(FluidRenderingMode::VelocityColor { min: 0.0, max: 5.0 });
    testbed.mechanical_world_mut().set_timestep(1.0 / 100.0);
    //    testbed.enable_boundary_particles_rendering(true);
}

fn main() {
    let testbed = Testbed::from_builders(0, vec![("Boxes", init_world)]);
    testbed.run()
}
