use std::{io::Read, fs::File};

use midly::{Smf, TrackEventKind::Midi, TrackEventKind::Meta, MidiMessage::{NoteOn, NoteOff}, num::{u7, u15, u24, u28}, MetaMessage::{Tempo, EndOfTrack}, Timing::{Metrical, Timecode}};

use crate::{LAYOUT, BLACK};

#[derive(Debug, Clone)]
pub struct Note {
    pub note: u8,       // A0 is 21 ; C8 is 108
    pub start: f32,
    pub end: f32,
}

#[derive(Clone)]
pub struct Notes {
    pub notes: Vec<Note>,
    pub vert: Vec<f32>,
    pub ind: Vec<u32>,
}

impl Notes {
    pub fn from_midi(wh_ratio: f32, framerate: f32, midi_file: &str) -> std::io::Result<(Notes, usize)> { // Done Twice instead of just ….clone().iter_mut { +0.5 }
        let mut notes: Vec<Note> = vec![];
        let mut blacknotes: Vec<Note> = vec![];
        let mut active_notes: Vec<Option<Note>> = vec![None; 128];

        let mut file = File::open(midi_file).expect("\nMidi file could not be opened. \nCheck the file path, and retry");
        let mut buf: Vec<u8> = vec![];
        let numbytes: usize = file.read_to_end(&mut buf).expect("\nMidi file could not be read.");
        println!("Reading {}-byte midi file…", numbytes);
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
        let mut max_frame: usize = 0;

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
                                max_frame = ((current_time + 4.) * framerate) as usize;
                            },
                            _ => {}
                        }
                    },

                    _ => {}
                }
            }
        }

        notes.extend(blacknotes);

        
        let mut new = Notes { 
            notes,
            vert: vec![],
            ind: vec![],
        };
        new.notes_to_vertices(wh_ratio).unwrap();

        Ok((new, max_frame))
    }

    pub fn notes_to_vertices(&mut self, wh_ratio: f32) -> std::io::Result<()>{
        for (i, n) in self.notes.iter().enumerate() {
            let ver2: Vec<f32> = vec![
                 //               x                             y          color  
                 LAYOUT[n.note as usize-21][0],         (n.start),          1.0,
                 LAYOUT[n.note as usize-21][1],         (n.start),          1.0,
                 LAYOUT[n.note as usize-21][1],         (n.end),            1.0,
                 LAYOUT[n.note as usize-21][0],         (n.end),            1.0,
                 //               x                             y          color 
                 LAYOUT[n.note as usize-21][0]+0.007,   (n.start+0.007*wh_ratio),    0.9,
                 LAYOUT[n.note as usize-21][1]-0.007,   (n.start+0.007*wh_ratio),    0.9,
                 LAYOUT[n.note as usize-21][1]-0.007,   (n.end-0.007*wh_ratio),      0.9,
                 LAYOUT[n.note as usize-21][0]+0.007,   (n.end-0.007*wh_ratio),      0.9,
            ];
            self.vert.extend(ver2);

            let i2: u32 = i as u32;
            let ind2: Vec<u32> = vec![
                0+8*i2, 2+8*i2, 1+8*i2,
                0+8*i2, 2+8*i2, 3+8*i2,
                4+8*i2, 6+8*i2, 5+8*i2,
                4+8*i2, 6+8*i2, 7+8*i2,
            ];
            self.ind.extend(ind2);
        }

        Ok(())
    }
}