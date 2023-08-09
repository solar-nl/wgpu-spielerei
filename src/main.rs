use wgpu_solar::run;

fn main() {
    // we call the run function using pollster since we using some async functions with wgpu
    pollster::block_on(run("Solar Winner Demo", 960.0, 540.0));
}