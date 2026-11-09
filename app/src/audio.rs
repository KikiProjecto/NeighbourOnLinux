use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

#[derive(Clone, Default)]
pub struct Audio {
    pub step: Option<Sound>,
    pub door: Option<Sound>,
    pub key: Option<Sound>,
    pub throw: Option<Sound>,
    pub impact: Option<Sound>,
    pub alert: Option<Sound>,
    pub caught: Option<Sound>,
    pub win: Option<Sound>,
}

impl Audio {
    pub fn empty() -> Self {
        Self::default()
    }

    pub async fn load() -> Self {
        Self {
            step: load_wav(&gen_step_wav()).await,
            door: load_wav(&gen_door_wav()).await,
            key: load_wav(&gen_key_wav()).await,
            throw: load_wav(&gen_throw_wav()).await,
            impact: load_wav(&gen_impact_wav()).await,
            alert: load_wav(&gen_alert_wav()).await,
            caught: load_wav(&gen_caught_wav()).await,
            win: load_wav(&gen_win_wav()).await,
        }
    }

    pub fn play_step(&self) {
        self.play(&self.step);
    }

    pub fn play_door(&self) {
        self.play(&self.door);
    }

    pub fn play_key(&self) {
        self.play(&self.key);
    }

    pub fn play_throw(&self) {
        self.play(&self.throw);
    }

    pub fn play_impact(&self) {
        self.play(&self.impact);
    }

    pub fn play_alert(&self) {
        self.play(&self.alert);
    }

    pub fn play_caught(&self) {
        self.play(&self.caught);
    }

    pub fn play_win(&self) {
        self.play(&self.win);
    }

    fn play(&self, sound: &Option<Sound>) {
        if let Some(s) = sound {
            play_sound_once(s);
        }
    }
}

async fn load_wav(bytes: &[u8]) -> Option<Sound> {
    load_sound_from_bytes(bytes).await.ok()
}

fn create_wav(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&file_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 2;
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

fn lcg_noise(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    ((*state >> 16) as f32 / 32767.5) - 1.0
}

fn gen_step_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.06) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 12345u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (1.0 - t / 0.06).max(0.0).powi(2);
        let freq = 90.0 - t * 400.0;
        let tone = (2.0 * std::f32::consts::PI * freq * t).sin();
        let noise = lcg_noise(&mut rng) * 0.25;
        let val = (tone * 0.75 + noise) * env * 0.45;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_door_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.35) as usize;
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = if t < 0.05 {
            t / 0.05
        } else {
            (1.0 - (t - 0.05) / 0.3).max(0.0)
        };
        let wobble = (2.0 * std::f32::consts::PI * 8.0 * t).sin() * 15.0;
        let freq = 130.0 + t * 90.0 + wobble;
        let phase = 2.0 * std::f32::consts::PI * freq * t;
        let tone = phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3;
        let val = tone * env * 0.5;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_key_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.35) as usize;
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env1 = (-t * 12.0).exp();
        let env2 = if t > 0.08 { (-(t - 0.08) * 10.0).exp() } else { 0.0 };
        let tone1 = (2.0 * std::f32::consts::PI * 1318.5 * t).sin() * env1;
        let tone2 = (2.0 * std::f32::consts::PI * 1975.5 * t).sin() * env2;
        let val = (tone1 * 0.5 + tone2 * 0.5) * 0.6;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_throw_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.16) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 54321u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (-(t - 0.06).powi(2) / 0.003).exp();
        let freq = 280.0 - t * 800.0;
        let tone = (2.0 * std::f32::consts::PI * freq.max(50.0) * t).sin() * 0.4;
        let noise = lcg_noise(&mut rng) * 0.6;
        let val = (tone + noise) * env * 0.5;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_impact_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.22) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 98765u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (-t * 18.0).exp();
        let freq = 85.0 - t * 200.0;
        let tone = (2.0 * std::f32::consts::PI * freq.max(35.0) * t).sin();
        let click = if t < 0.015 { lcg_noise(&mut rng) * 0.5 } else { 0.0 };
        let val = (tone * 0.7 + click) * env * 0.65;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_alert_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.5) as usize;
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = if t < 0.04 {
            t / 0.04
        } else {
            (1.0 - (t - 0.04) / 0.46).max(0.0)
        };
        let tone1 = (2.0 * std::f32::consts::PI * 220.0 * t).sin();
        let tone2 = (2.0 * std::f32::consts::PI * 311.1 * t).sin();
        let tone3 = (2.0 * std::f32::consts::PI * 622.2 * t).sin();
        let val = (tone1 * 0.4 + tone2 * 0.4 + tone3 * 0.2) * env * 0.55;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_caught_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.6) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 13579u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (-t * 6.0).exp();
        let freq = 70.0 - t * 40.0;
        let tone = (2.0 * std::f32::consts::PI * freq.max(30.0) * t).sin();
        let noise = if t < 0.12 { lcg_noise(&mut rng) * (1.0 - t / 0.12) * 0.6 } else { 0.0 };
        let val = (tone * 0.65 + noise) * env * 0.7;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_win_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.8) as usize;
    let mut samples = Vec::with_capacity(count);
    let notes = [(523.25, 0.0, 0.18), (659.25, 0.18, 0.36), (783.99, 0.36, 0.54), (1046.50, 0.54, 0.80)];
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let mut val = 0.0;
        for &(freq, start, end) in &notes {
            if t >= start && t < end {
                let note_t = t - start;
                let note_len = end - start;
                let env = (1.0 - note_t / note_len).max(0.0).powi(2);
                val += (2.0 * std::f32::consts::PI * freq * note_t).sin() * env * 0.5;
            }
        }
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}
