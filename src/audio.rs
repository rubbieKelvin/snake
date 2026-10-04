use std::fs::File;
use std::sync::Arc;

use lewton::inside_ogg::OggStreamReader;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;

/// sound events the game can emit
/// each one maps to a clip from the kenney sci-fi pack in assets/sounds
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
    Shoot,
    /// trigger pulled on an empty chamber
    Empty,
    Reload,
    /// a bullet hit an enemy that survived it
    Hit,
    /// a bullet popped a virus
    Pop,
}

const ALL: [Sfx; 17] = [
    Sfx::Start,
    Sfx::Eat,
    Sfx::EatGolden,
    Sfx::Power,
    Sfx::Heal,
    Sfx::Boost,
    Sfx::Hurt,
    Sfx::ShieldBreak,
    Sfx::Ram,
    Sfx::Crash,
    Sfx::LevelUp,
    Sfx::GameOver,
    Sfx::Shoot,
    Sfx::Empty,
    Sfx::Reload,
    Sfx::Hit,
    Sfx::Pop,
];

/// how a clip is played back
struct Cue {
    file: &'static str,
    /// playback speed; >1 is shorter and higher pitched
    pitch: f32,
    vol: f32,
    /// cut the clip off after this many ms (0 = play it all)
    max_ms: u32,
}

fn cue(file: &'static str, pitch: f32, vol: f32, max_ms: u32) -> Cue {
    return Cue {
        file,
        pitch,
        vol,
        max_ms,
    };
}

fn cue_for(sfx: Sfx) -> Cue {
    match sfx {
        Sfx::Start => cue("doorOpen_000", 1.0, 0.6, 0),
        Sfx::Eat => cue("laserRetro_000", 1.3, 0.35, 0),
        Sfx::EatGolden => cue("laserRetro_004", 1.6, 0.45, 0),
        Sfx::Power => cue("forceField_000", 1.0, 0.6, 0),
        Sfx::Heal => cue("forceField_002", 1.25, 0.6, 0),
        Sfx::Boost => cue("thrusterFire_000", 1.0, 0.45, 450),
        Sfx::Hurt => cue("impactMetal_000", 1.0, 0.7, 0),
        Sfx::ShieldBreak => cue("forceField_004", 0.75, 0.6, 0),
        Sfx::Ram => cue("impactMetal_003", 1.0, 0.7, 0),
        Sfx::Crash => cue("explosionCrunch_000", 1.0, 0.7, 0),
        Sfx::LevelUp => cue("doorOpen_001", 1.2, 0.6, 0),
        Sfx::GameOver => cue("lowFrequency_explosion_000", 1.0, 0.8, 0),
        Sfx::Shoot => cue("laserSmall_000", 1.0, 0.4, 0),
        Sfx::Empty => cue("doorClose_000", 2.0, 0.4, 120),
        Sfx::Reload => cue("doorClose_001", 1.5, 0.5, 0),
        Sfx::Hit => cue("impactMetal_002", 1.3, 0.5, 0),
        Sfx::Pop => cue("slime_000", 1.0, 0.6, 0),
    }
}

/// decoded mono clip at its native sample rate
struct Clip {
    samples: Vec<f32>,
    rate: f32,
}

/// Decodes an ogg into mono f32 samples; stereo files get averaged down.
fn load_clip(path: &str) -> Option<Clip> {
    let file = File::open(path).ok()?;
    let mut reader = OggStreamReader::new(file).ok()?;
    let channels = reader.ident_hdr.audio_channels.max(1) as usize;
    let rate = reader.ident_hdr.audio_sample_rate as f32;
    let mut samples = Vec::new();
    while let Some(packet) = reader.read_dec_packet_itl().ok()? {
        for frame in packet.chunks(channels) {
            let sum: f32 = frame.iter().map(|&s| s as f32 / 32768.0).sum();
            samples.push(sum / channels as f32);
        }
    }
    return Some(Clip { samples, rate });
}

struct Voice {
    clip: Arc<Clip>,
    /// fractional read position into the clip
    pos: f32,
    step: f32,
    /// position to stop at, and where the fade-out begins
    end: f32,
    fade_from: f32,
    vol: f32,
}

struct Mixer {
    voices: Vec<Voice>,
}

impl AudioCallback for Mixer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            let mut mix = 0.0f32;
            for v in self.voices.iter_mut() {
                if v.pos >= v.end {
                    continue;
                }
                // linear interpolation handles both pitch shift and rate mismatch
                let i = v.pos as usize;
                let frac = v.pos - i as f32;
                let a = v.clip.samples[i];
                let b = v.clip.samples.get(i + 1).copied().unwrap_or(0.0);
                let mut s = a + (b - a) * frac;
                // fade truncated clips so they don't click when cut
                if v.pos > v.fade_from {
                    s *= (v.end - v.pos) / (v.end - v.fade_from);
                }
                mix += s * v.vol;
                v.pos += v.step;
            }
            *sample = mix.clamp(-1.0, 1.0);
        }
        self.voices.retain(|v| v.pos < v.end);
    }
}

pub struct Audio {
    device: AudioDevice<Mixer>,
    rate: f32,
    clips: Vec<Option<Arc<Clip>>>,
    pub muted: bool,
}

impl Audio {
    /// Returns None when no audio device is available; the game just stays silent.
    /// A clip that fails to load is skipped rather than failing everything.
    pub fn new(subsystem: &AudioSubsystem) -> Option<Audio> {
        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: Some(512),
        };
        let device = subsystem
            .open_playback(None, &spec, |_| Mixer { voices: Vec::new() })
            .ok()?;
        device.resume();
        let rate = device.spec().freq as f32;
        let clips = ALL
            .iter()
            .map(|&sfx| {
                let path = format!("assets/sounds/{}.ogg", cue_for(sfx).file);
                let clip = load_clip(&path);
                if clip.is_none() {
                    eprintln!("couldn't load {path}");
                }
                clip.map(Arc::new)
            })
            .collect();
        return Some(Audio {
            device,
            rate,
            clips,
            muted: false,
        });
    }

    pub fn play(&mut self, sfx: Sfx) {
        if self.muted {
            return;
        }
        let index = ALL.iter().position(|&s| s == sfx).unwrap();
        let Some(clip) = self.clips[index].clone() else {
            return;
        };
        let cue = cue_for(sfx);
        let len = (clip.samples.len() - 1) as f32;
        let (end, fade_from) = if cue.max_ms > 0 {
            let end = (cue.max_ms as f32 * clip.rate / 1000.0).min(len);
            (end, end * 0.6)
        } else {
            (len, len)
        };
        let voice = Voice {
            step: cue.pitch * clip.rate / self.rate,
            clip,
            pos: 0.0,
            end,
            fade_from,
            vol: cue.vol,
        };
        let mut mixer = self.device.lock();
        // keep a cap so a burst of events can't pile up
        if mixer.voices.len() < 24 {
            mixer.voices.push(voice);
        }
    }
}
