use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;

/// sound events the game can emit
/// we're synthesizing this at runtime
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Start,
    Eat,
    EatGolden,
    Power,
    Heal,
    Boost,
    Hurt,
    ShieldBreak,
    Ram,
    Crash,
    LevelUp,
    GameOver,
}

#[derive(Clone, Copy)]
enum Wave {
    Square,
    Triangle,
    Noise,
}

struct Voice {
    delay: u32,
    age: u32,
    len: u32,
    f0: f32,
    f1: f32,
    wave: Wave,
    vol: f32,
    phase: f32,
}

struct Synth {
    voices: Vec<Voice>,
    rate: f32,
    noise: u32,
}

impl AudioCallback for Synth {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            let mut mix = 0.0f32;
            for v in self.voices.iter_mut() {
                if v.delay > 0 {
                    v.delay -= 1;
                    continue;
                }
                let t = v.age as f32 / v.len as f32;
                let freq = v.f0 + (v.f1 - v.f0) * t;
                v.phase = (v.phase + freq / self.rate).fract();
                let raw = match v.wave {
                    Wave::Square => {
                        if v.phase < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    Wave::Triangle => 4.0 * (v.phase - 0.5).abs() - 1.0,
                    Wave::Noise => {
                        self.noise = self.noise.wrapping_mul(1664525).wrapping_add(1013904223);
                        (self.noise >> 16) as f32 / 32768.0 - 1.0
                    }
                };
                // short attack avoids clicks, linear decay for the rest
                let env = (1.0 - t) * (v.age as f32 / 100.0).min(1.0);
                mix += raw * env * v.vol;
                v.age += 1;
            }
            *sample = mix.clamp(-1.0, 1.0);
        }
        self.voices.retain(|v| v.age < v.len);
    }
}

pub struct Audio {
    device: AudioDevice<Synth>,
    rate: f32,
    pub muted: bool,
}

impl Audio {
    /// Returns None when no audio device is available; the game just stays silent.
    pub fn new(subsystem: &AudioSubsystem) -> Option<Audio> {
        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(512),
        };
        let device = subsystem
            .open_playback(None, &spec, |obtained| Synth {
                voices: Vec::new(),
                rate: obtained.freq as f32,
                noise: 12345,
            })
            .ok()?;
        device.resume();
        let rate = device.spec().freq as f32;
        return Some(Audio {
            device,
            rate,
            muted: false,
        });
    }

    pub fn play(&mut self, sfx: Sfx) {
        if self.muted {
            return;
        }
        let voices = voices_for(sfx, self.rate);
        let mut synth = self.device.lock();
        for v in voices {
            // keep a cap so a burst of events can't pile up
            if synth.voices.len() < 24 {
                synth.voices.push(v);
            }
        }
    }
}

fn note(rate: f32, delay_ms: u32, dur_ms: u32, f0: f32, f1: f32, wave: Wave, vol: f32) -> Voice {
    let ms = rate / 1000.0;
    return Voice {
        delay: (delay_ms as f32 * ms) as u32,
        age: 0,
        len: (dur_ms as f32 * ms) as u32,
        f0,
        f1,
        wave,
        vol,
        phase: 0.0,
    };
}

fn voices_for(sfx: Sfx, r: f32) -> Vec<Voice> {
    use Wave::*;
    match sfx {
        Sfx::Start => vec![
            note(r, 0, 90, 330.0, 330.0, Square, 0.14),
            note(r, 90, 160, 495.0, 495.0, Square, 0.14),
        ],
        Sfx::Eat => vec![note(r, 0, 80, 520.0, 800.0, Square, 0.16)],
        Sfx::EatGolden => vec![
            note(r, 0, 70, 660.0, 660.0, Square, 0.15),
            note(r, 60, 70, 880.0, 880.0, Square, 0.15),
            note(r, 120, 120, 1320.0, 1320.0, Square, 0.15),
        ],
        Sfx::Power => vec![note(r, 0, 260, 350.0, 1300.0, Triangle, 0.3)],
        Sfx::Heal => vec![
            note(r, 0, 100, 523.0, 523.0, Triangle, 0.3),
            note(r, 100, 100, 784.0, 784.0, Triangle, 0.3),
            note(r, 200, 200, 1047.0, 1047.0, Triangle, 0.3),
        ],
        Sfx::Boost => vec![note(r, 0, 110, 180.0, 420.0, Noise, 0.08)],
        Sfx::Hurt => vec![
            note(r, 0, 220, 220.0, 70.0, Square, 0.18),
            note(r, 0, 160, 0.0, 0.0, Noise, 0.2),
        ],
        Sfx::ShieldBreak => vec![note(r, 0, 150, 900.0, 250.0, Triangle, 0.3)],
        Sfx::Ram => vec![
            note(r, 0, 140, 300.0, 90.0, Square, 0.16),
            note(r, 0, 120, 0.0, 0.0, Noise, 0.22),
        ],
        Sfx::Crash => vec![note(r, 0, 240, 0.0, 0.0, Noise, 0.22)],
        Sfx::LevelUp => vec![
            note(r, 0, 90, 523.0, 523.0, Triangle, 0.3),
            note(r, 90, 90, 659.0, 659.0, Triangle, 0.3),
            note(r, 180, 90, 784.0, 784.0, Triangle, 0.3),
            note(r, 270, 220, 1047.0, 1047.0, Triangle, 0.3),
        ],
        Sfx::GameOver => vec![
            note(r, 0, 220, 392.0, 392.0, Triangle, 0.3),
            note(r, 220, 220, 330.0, 330.0, Triangle, 0.3),
            note(r, 440, 220, 262.0, 262.0, Triangle, 0.3),
            note(r, 660, 520, 196.0, 120.0, Triangle, 0.3),
        ],
    }
}
