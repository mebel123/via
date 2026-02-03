use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::AppHandle;

pub struct Recording {
    is_recording: Arc<AtomicBool>,
    current_file: Option<PathBuf>,
    recording_thread: Option<thread::JoinHandle<()>>,
}

impl Recording {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            current_file: None,
            recording_thread: None,
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }
    pub fn start(&mut self, app: &AppHandle) -> Result<(), String> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Err("Already recording".into());
        }

        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input device available")?;
        let config = device.default_input_config().map_err(|e| e.to_string())?;

        let spec = hound::WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };


        let path = crate::paths::next_recording_path(app);
        self.current_file = Some(path.clone());

        // Hier entsteht standardmäßig ein WavWriter<BufWriter<File>>
        let writer = hound::WavWriter::create(path, spec).map_err(|e| e.to_string())?;
        let writer = Arc::new(Mutex::new(writer));

        self.is_recording.store(true, Ordering::SeqCst);
        let is_recording = self.is_recording.clone();

        // Writer für den Thread klonen
        let thread_writer = writer.clone();

        let handle = thread::spawn(move || {
            let err_fn = move |err| {
                eprintln!("an error occurred on stream: {}", err);
            };

            // Klone für die Closures erstellen
            let writer_f32 = thread_writer.clone();
            let writer_i16 = thread_writer.clone();
            let writer_u16 = thread_writer.clone();

            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| write_f32(data, &writer_f32), // Hier write_f32 nutzen
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::I16 => device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &_| write_i16(data, &writer_i16), // Hier write_i16 nutzen
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::U16 => device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &_| write_u16(data, &writer_u16), // Hier write_u16 nutzen
                    err_fn,
                    None,
                ),
                _ => panic!("Unsupported sample format"),
            }.unwrap();

            stream.play().unwrap();

            while is_recording.load(Ordering::SeqCst) {
                thread::sleep(std::time::Duration::from_millis(100));
            }

            drop(stream); // Wichtig: Stream stoppen, damit Writer freigegeben wird

            // Writer finalisieren (schreibt Header-Länge)
            if let Ok(mutex) = Arc::try_unwrap(thread_writer) {
                if let Ok(w) = mutex.into_inner() {
                    w.finalize().unwrap();
                }
            }
        });

        self.recording_thread = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<PathBuf, String> {
        if !self.is_recording.load(Ordering::SeqCst) {
            return Err("Not recording".into());
        }

        self.is_recording.store(false, Ordering::SeqCst);

        if let Some(handle) = self.recording_thread.take() {
            handle
                .join()
                .map_err(|_| "Failed to join recording thread".to_string())?;
        }

        // Pfad sichern, BEVOR wir ihn löschen
        let path = self
            .current_file
            .take()
            .ok_or("No recording file available")?;

        Ok(path)
    }
}

fn write_f32(input: &[f32], writer: &Arc<Mutex<hound::WavWriter<BufWriter<File>>>>) {
    if let Ok(mut guard) = writer.lock() {
        for &sample in input {
            // f32 ist schon f32, wir skalieren nur
            let s_i16 = (sample * i16::MAX as f32) as i16;
            guard.write_sample(s_i16).ok();
        }
    }
}

fn write_i16(input: &[i16], writer: &Arc<Mutex<hound::WavWriter<BufWriter<File>>>>) {
    if let Ok(mut guard) = writer.lock() {
        for &sample in input {
            // i16 ist schon das Zielformat
            guard.write_sample(sample).ok();
        }
    }
}

fn write_u16(input: &[u16], writer: &Arc<Mutex<hound::WavWriter<BufWriter<File>>>>) {
    if let Ok(mut guard) = writer.lock() {
        for &sample in input {
            // u16 in i16 umrechnen (verschieben)
            let s = (sample as f32 - 32768.0) / 32768.0; // erst zu f32 (-1.0 bis 1.0)
            let s_i16 = (s * i16::MAX as f32) as i16;
            guard.write_sample(s_i16).ok();
        }
    }
}
