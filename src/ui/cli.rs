// pub duration: f64,
// pub tempo: f32,
// pub border_radius: u8,
// pub bgimg_file: &'static str,
pub struct Parameters {
    pub width: usize,
    pub height: usize,
    pub samples: u8,
    pub framerate: f32,
    pub cores: usize,
    // Relative path from where the executable is called
    pub midi_file: String,
    // Relative path from where the executable is called
    pub index_file: String,
    pub mp4_file: String,
    pub png_file: String,
    pub clear_dir: bool,
}

impl Parameters {
    pub fn build(args: & [String]) -> Result<Parameters, &'static str> {
        if args.len() != 8 {
            return Err("The number of arguments is incorrect");
        }
        let mut args: std::slice::Iter<'_, String> = args.iter();
        let width_str = args.next().expect("A parameter is empty"); let width = width_str.parse::<usize>().unwrap();
        let height_str = args.next().expect("A parameter is empty"); let height = height_str.parse::<usize>().unwrap();
        let framerate_str = args.next().expect("A parameter is empty"); let framerate = framerate_str.parse::<f32>().unwrap();
        let samples_str = args.next().expect("A parameter is empty"); let samples = samples_str.parse::<u8>().unwrap();
        let midi_file = args.next().expect("A parameter is empty").to_string();
        let mp4_file = args.next().expect("A parameter is empty").to_string(); 
        let png_file = args.next().expect("A parameter is empty").to_string(); 
        let clear_dir_str = args.next().expect("A parameter is empty"); let clear_dir = clear_dir_str.parse::<bool>().unwrap();

        let cores: usize = num_cpus::get();
        let index_file = "index.txt".to_owned();

        Ok(Parameters {
            width,
            height,
            samples,
            framerate,
            cores,
            midi_file,
            index_file,
            mp4_file,
            png_file,
            clear_dir,
        })
    }
}