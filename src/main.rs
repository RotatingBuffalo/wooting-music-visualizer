use byteorder::{ReadBytesExt, BE};
use spoofylightslib::raymond::wooting;
use spoofylightslib::{frame::pixel::Pixel, raymond::wooting::draw_frame};
use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
fn cava_setup(num_bars: u8, smoothing: u8, framerate: u8, sensitivity: u8) {
    let config = format!(
        "
[general]
bars = {}
framerate = {}
sensitivity = {}
[input]
method = fifo
source = /tmp/cava.fifo
sample_rate = 48000
[output]
method = raw
raw_target = \"/dev/stdout\"
bit_format = \"16bit\"
[smoothing]
integral = {}
",
        num_bars, framerate, sensitivity, smoothing,
    );
    fs::write("cavaconf", config).ok();
}
fn main() {
    // fifo setup
    let mkfifo = Command::new("wsl")
        .arg("mkfifo")
        .arg("/tmp/cava.fifo")
        .spawn()
        .expect("what");
    let winscap = Command::new("wsl")
        .arg("/mnt/c/Users/maxbe/Theming/JankWorkarounds/winscap.exe")
        .arg("2")
        .arg("48000")
        .arg("16")
        .arg(">")
        .arg("/tmp/cava.fifo")
        .spawn()
        .expect("oh god what have i done");

    // cava setup
    const NUM_BARS: u8 = 21;
    const SMOOTHING: u8 = 35;
    const FRAMERATE: u8 = 60;
    const SENSITIVITY: u8 = 50;
    cava_setup(NUM_BARS, SMOOTHING, FRAMERATE, SENSITIVITY);
    let mut cava = Command::new("wsl")
        .arg("cava")
        .arg("-p")
        .arg("cavaconf")
        .stdout(Stdio::piped())
        .spawn()
        .expect(
            "Failed to run cava in WSL.
    \nHonestly this could happen for a whole host of reasons,
    \nand I would just recommend you look at the source code and debug it yourself.",
        );

    let mut bar_vals: [u16; NUM_BARS as usize] = [0; NUM_BARS as usize];
    let mut stdout = cava.stdout.take().unwrap();

    loop {
        stdout
            .read_u16_into::<BE>(&mut bar_vals)
            .expect("something has gone awry with byteorder.");
        println!("{:?}", bar_vals);

        let mut pixel_vector = vec![vec![Pixel::new(Some((255, 255, 255))); 21]; 6];
        for i in 0..6 {
            for j in 0..21 {
                let this_pixel = if (bar_vals[j] as u32 <= 65536 / (i + 1)) {
                    Pixel::new(Some((0, 0, 0)))
                } else {
                    Pixel::new(Some((255, 255, 255)))
                };
                pixel_vector[i as usize][j as usize] = this_pixel;
            }
        }
        draw_frame(pixel_vector);
        //thread::sleep(Duration::new(0, 1000));
    }
}
