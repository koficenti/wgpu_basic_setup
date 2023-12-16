fn main() {
    pollster::block_on(wgpu_basic_setup::window::run("Rainbow Square"));
}
