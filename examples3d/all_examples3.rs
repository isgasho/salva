#![allow(dead_code)]

extern crate nalgebra as na;
extern crate ncollide3d;
extern crate nphysics3d;
extern crate nphysics_testbed3d;

use inflector::Inflector;

use nphysics_testbed3d::Testbed;

mod basic3;
mod custom_forces3;
mod elasticity3;
mod faucet3;
mod surface_tension3;

fn demo_name_from_command_line() -> Option<String> {
    let mut args = std::env::args();

    while let Some(arg) = args.next() {
        if &arg[..] == "--example" {
            return args.next();
        }
    }

    None
}

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
fn demo_name_from_url() -> Option<String> {
    let window = stdweb::web::window();
    let hash = window.location()?.search().ok()?;
    if !hash.is_empty() {
        Some(hash[1..].to_string())
    } else {
        None
    }
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
fn demo_name_from_url() -> Option<String> {
    None
}

fn main() {
    let demo = demo_name_from_command_line()
        .or_else(|| demo_name_from_url())
        .unwrap_or(String::new())
        .to_camel_case();

    let mut builders: Vec<(_, fn(&mut Testbed))> = vec![
        ("Basic", basic3::init_world),
        ("Custom forces", custom_forces3::init_world),
        ("Elasticity", elasticity3::init_world),
        ("Faucet", faucet3::init_world),
        ("Surface tension", surface_tension3::init_world),
    ];

    builders.sort_by_key(|builder| builder.0);

    let i = builders
        .iter()
        .position(|builder| builder.0.to_camel_case().as_str() == demo.as_str())
        .unwrap_or(0);
    let testbed = Testbed::from_builders(i, builders);

    testbed.run()
}
