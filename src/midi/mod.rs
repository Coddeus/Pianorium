use std::{io::Read, fs::File};
use midly::{Smf, TrackEventKind::Midi, TrackEventKind::Meta, MidiMessage::{NoteOn, NoteOff}, num::{u7, u15, u24, u28}, MetaMessage::{Tempo, EndOfTrack}, Timing::{Metrical, Timecode}};

const LAYOUT: [[f32 ; 2] ; 88] = [
    [-26./26., -25./26.],
    [-25.15/26., -24.55/26.],
    [-25./26., -24./26.],
    
    [-24./26., -23./26.],
    [-23.4/26., -22.8/26.],
    [-23./26., -22./26.],
    [-22.2/26., -21.6/26.],
    [-22./26., -21./26.],
    [-21./26., -20./26.],
    [-20.45/26., -19.85/26.],
    [-20./26., -19./26.],
    [-19.3/26., -18.7/26.],
    [-19./26., -18./26.],
    [-18.15/26., -17.55/26.],
    [-18./26., -17./26.],
    
    [-17./26., -16./26.],
    [-16.4/26., -15.8/26.],
    [-16./26., -15./26.],
    [-15.2/26., -14.6/26.],
    [-15./26., -14./26.],
    [-14./26., -13./26.],
    [-13.45/26., -12.85/26.],
    [-13./26., -12./26.],
    [-12.3/26., -11.7/26.],
    [-12./26., -11./26.],
    [-11.15/26., -10.55/26.],
    [-11./26., -10./26.],
    
    [-10./26., -9./26.],
    [-9.4/26., -8.8/26.],
    [-9./26., -8./26.],
    [-8.2/26., -7.6/26.],
    [-8./26., -7./26.],
    [-7./26., -6./26.],
    [-6.45/26., -5.85/26.],
    [-6./26., -5./26.],
    [-5.3/26., -4.7/26.],
    [-5./26., -4./26.],
    [-4.15/26., -3.55/26.],
    [-4./26., -3./26.],
    
    [-3./26., -2./26.],
    [-2.4/26., -1.8/26.],
    [-2./26., -1./26.],
    [-1.2/26., -0.6/26.],
    [-1./26., 0./26.],
    [0./26., 1./26.],
    [0.55/26., 1.15/26.],
    [1./26., 2./26.],
    [1.7/26., 2.3/26.],
    [2./26., 3./26.],
    [2.85/26., 3.45/26.],
    [3./26., 4./26.],
    
    [4./26., 5./26.],
    [4.6/26., 5.2/26.],
    [5./26., 6./26.],
    [5.8/26., 6.4/26.],
    [6./26., 7./26.],
    [7./26., 8./26.],
    [7.55/26., 8.15/26.],
    [8./26., 9./26.],
    [8.7/26., 9.3/26.],
    [9./26., 10./26.],
    [9.85/26., 10.45/26.],
    [10./26., 11./26.],
    
    [11./26., 12./26.],
    [11.6/26., 12.2/26.],
    [12./26., 13./26.],
    [12.8/26., 13.4/26.],
    [13./26., 14./26.],
    [14./26., 15./26.],
    [14.55/26., 15.15/26.],
    [15./26., 16./26.],
    [15.7/26., 16.3/26.],
    [16./26., 17./26.],
    [16.85/26., 17.45/26.],
    [17./26., 18./26.],
    
    [18./26., 19./26.],
    [18.6/26., 19.2/26.],
    [19./26., 20./26.],
    [19.8/26., 20.4/26.],
    [20./26., 21./26.],
    [21./26., 22./26.],
    [21.55/26., 22.15/26.],
    [22./26., 23./26.],
    [22.7/26., 23.3/26.],
    [23./26., 24./26.],
    [23.85/26., 24.45/26.],
    [24./26., 25./26.],
    
    [25./26., 26./26.],
]; // Look for LAYOUT[midinote-21]
const BLACK: [u8 ; 36] = [1, 4, 6, 9, 11, 13, 16, 18, 21, 23, 25, 28, 30, 33, 35, 37, 40, 42, 45, 47, 49, 52, 54, 57, 59, 61, 64, 66, 69, 71, 73, 76, 78, 81, 83, 85];

#[derive(Debug, Clone)]
struct Note {
    note: u8,       // A0 is 21 ; C8 is 108
    start: f32,
    end: f32,
}

pub fn midi_to_vertices(framerate: f32, midi_file: String) -> std::io::Result<(Vec<f32>, Vec<u32>, usize)> { // Done Twice instead of just ….clone().iter_mut { +0.5 }
    let mut vertices: Vec<f32> = vec![];
    let mut indices: Vec<u32> = vec![];

    let mut notes: Vec<Note> = vec![];
    let mut blacknotes: Vec<Note> = vec![];
    let mut active_notes: Vec<Option<Note>> = vec![None; 128];

    let mut file = File::open(midi_file).expect("\nMidi file could not be opened. \nCheck the file path, and retry");
    let mut buf: Vec<u8> = vec![];
    let numbytes: usize = file.read_to_end(&mut buf).expect("\nMidi file could not be read.");
    println!("{}-byte midi file successfully read!", numbytes);
    let midi_data = Smf::parse(&buf).unwrap();

    let mut spb: f32 = 0.5; // Seconds per tick
    let mut spt: f32; // Seconds per beat
    match midi_data.header.timing {
        Metrical(m) => {
            let ppq: f32 = <u15 as Into<u16>>::into(m) as f32;
            spt = spb / ppq;

        },
        Timecode(fps, sfpf) => {
            spt = 1./fps.as_f32()/sfpf as f32;
        }
    }
    let mut eot: usize = 0;

    for track in midi_data.tracks.iter() {
        let mut current_time: f32 = 2.;
        for event in track.iter() {
            current_time += <u28 as Into<u32>>::into(event.delta) as f32 * spt;
            match event.kind {

                Midi { channel: _, message } => {
                    match message {
                        NoteOn { key, vel } => {
                            if 20 < key && key < 109 {
                                if vel > 0 {
                                    let note_obj = Note {
                                        note: <u7 as Into<u8>>::into(key),
                                        start: current_time,
                                        end: 0.,
                                    };
                                    active_notes[<u7 as Into<u8>>::into(key) as usize] = Some(note_obj);
                                } else {
                                    if let Some(mut note_obj) = active_notes[<u7 as Into<u8>>::into(key) as usize].take() {
                                        note_obj.end = current_time;
                                        if BLACK.contains(&note_obj.note) {
                                            blacknotes.push(note_obj);
                                        } else {
                                            notes.push(note_obj);
                                        }
                                        active_notes[<u7 as Into<u8>>::into(key) as usize] = None;
                                    }
                                }
                            }
                        },
                        NoteOff { key, vel: _ } => {
                            if let Some(mut note_obj) = active_notes[<u7 as Into<u8>>::into(key) as usize].take() {
                                note_obj.end = current_time;
                                if BLACK.contains(&note_obj.note) {
                                    blacknotes.push(note_obj);
                                } else {
                                    notes.push(note_obj);
                                }
                                active_notes[<u7 as Into<u8>>::into(key) as usize] = None;
                            }
                        },
                        _ => {}
                    }
                },

                Meta(message) => {
                    match message {
                        Tempo(t) => {   // This event should only be present when header timing is "Metrical"
                            let tempo: f32 = <u24 as Into<u32>>::into(t) as f32/1000000.;
                            spt = spt/spb*tempo;
                            spb = tempo;
                        },
                        EndOfTrack => {      // Know when the render finishes
                            eot = ((current_time + 4.) * framerate) as usize;
                        },
                        _ => {}
                    }
                },

                _ => {}
            }
        }
    }

    notes.extend(blacknotes);

    for (i, n) in notes.iter().enumerate() {
        let ver2: Vec<f32> = vec![
             //               x                             y          color  
             LAYOUT[n.note as usize-21][0],         (n.start),          1.,
             LAYOUT[n.note as usize-21][1],         (n.start),          1.,
             LAYOUT[n.note as usize-21][1],         (n.end),            1.,
             LAYOUT[n.note as usize-21][0],         (n.end),            1.,
             //               x                             y          color 
             LAYOUT[n.note as usize-21][0]+0.004,   (n.start+0.004),    0.,
             LAYOUT[n.note as usize-21][1]-0.004,   (n.start+0.004),    0.,
             LAYOUT[n.note as usize-21][1]-0.004,   (n.end-0.004),      0.,
             LAYOUT[n.note as usize-21][0]+0.004,   (n.end-0.004),      0.,
        ];
        vertices.extend(ver2);
        
        let i2: u32 = i as u32;
        let ind2: Vec<u32> = vec![
            0+8*i2, 2+8*i2, 1+8*i2,
            0+8*i2, 2+8*i2, 3+8*i2,
            4+8*i2, 6+8*i2, 5+8*i2,
            4+8*i2, 6+8*i2, 7+8*i2,
        ];
        indices.extend(ind2);
    }

    Ok((vertices, indices, eot))
}