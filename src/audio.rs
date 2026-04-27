use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::lufs::KWeighting;

pub struct AudioSnapshot {
    data: Vec<f32>
}
impl AudioSnapshot {
    pub fn windowed(&self, n_samples: usize) -> f32 {
        let scale = self.data.iter().take(n_samples).map(|x| x*x).sum::<f32>()/(n_samples as f32);
        scale.log10() * 10. - 0.691
    }
    pub fn momentary(&self) -> f32 {
        self.windowed((48000 / 1000) * 400)
    }
    pub fn short_term(&self) -> f32 {
        self.windowed(48000 * 3)
    }
}

pub struct AudioProcessor {
    _stream: cpal::Stream,
    data: Arc<Mutex<circular_buffer::CircularBuffer::<{48000 * 3}, f32>>>
}
impl AudioProcessor {
    pub fn new(device: Option<String>) -> Self {
        let host = cpal::default_host();
        let device =
            if let Some(device) = device {
                host.device_by_id(&cpal::DeviceId(host.id(), device))
            }
            else {
                host.default_input_device()
            }.expect("No input device");
        println!("Selected: {}:{:?}", host.id(), device.description());
        let default_config = device.default_input_config().expect("Failed to get default config");
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: 48000,
            .. default_config.config()
        };
        let data = Arc::new(Mutex::new(circular_buffer::CircularBuffer::from_iter(
            std::iter::repeat_n(0f32, 48000 * 3)
        )));
        let data_copy = data.clone();
        let mut processor = KWeighting::new();
        let stream = device.build_input_stream(&config, move |samples: &[f32], _| {
            let mut data = data_copy.lock().unwrap();
            for i in samples {
                data.push_front(processor.process(*i));
            }
        }, |e| eprintln!("Stream error: {e}"), None).unwrap();
        stream.play().unwrap();
        Self { _stream: stream, data }
    }
    pub fn snapshot(&self) -> AudioSnapshot {
        // Copy to minimise lock contention
        AudioSnapshot { data: self.data.lock().unwrap().iter().copied().collect() }
    }
}
