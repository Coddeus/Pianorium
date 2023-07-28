// pub duration: f64,
// pub tempo: f32,
// pub border_radius: u8,
// pub bgimg_file: &'static str,
pub struct Parameters {
    pub width: usize,
    pub height: usize,
    pub framerate: f32,
    pub samples: u8,
    // Relative path from where the executable is called
    pub midi_file: String,
    // Relative path from where the executable is called
    pub output_file: String,
    pub clear_dir: bool,
}

impl Parameters {
    pub fn build(args: &[String]) -> Result<Parameters, &'static str> {
        if args.len() != 7 {
            return Err("The number of arguments is incorrect");
        }

        let width_str = args[0].clone(); let width = width_str.parse::<usize>().unwrap();
        let height_str = args[1].clone(); let height = height_str.parse::<usize>().unwrap();
        let framerate_str = args[2].clone(); let framerate = framerate_str.parse::<f32>().unwrap();
        let samples_str = args[3].clone(); let samples = samples_str.parse::<u8>().unwrap();
        let midi_file = args[4].clone();
        let output_file = args[5].clone();
        let clear_dir_str = args[6].clone(); let clear_dir = clear_dir_str.parse::<bool>().unwrap();

        Ok(Parameters {
            width,
            height,
            framerate,
            samples,
            midi_file,
            output_file,
            clear_dir,
        })
    }
}