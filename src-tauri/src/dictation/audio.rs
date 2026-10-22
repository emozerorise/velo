use super::types::{CapturedAudio, DictationError, Result, MAX_SECONDS, OUTPUT_RATE};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

pub(crate) fn finite_sample(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

pub fn to_16khz_mono(mut input: CapturedAudio) -> Result<Vec<f32>> {
    let frames = input.mono_samples.len();
    if !(8000..=192000).contains(&input.sample_rate) {
        return Err(DictationError::UnsupportedFormat);
    }
    if frames == 0 || frames > input.sample_rate as usize * MAX_SECONDS {
        return Err(DictationError::InvalidInput);
    }
    for sample in &mut input.mono_samples {
        *sample = finite_sample(*sample);
    }
    if input.sample_rate == OUTPUT_RATE {
        return Ok(input.mono_samples);
    }
    let mut resampler = SincFixedIn::<f32>::new(
        f64::from(OUTPUT_RATE) / f64::from(input.sample_rate),
        1.0,
        SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Cubic,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        },
        1024,
        1,
    )
    .map_err(|_| DictationError::CaptureFailed)?;
    let expected = ((frames as u64 * u64::from(OUTPUT_RATE) + u64::from(input.sample_rate / 2))
        / u64::from(input.sample_rate)) as usize;
    let delay = resampler.output_delay();
    let mut output = Vec::with_capacity(expected + delay + resampler.output_frames_max());
    for chunk in input.mono_samples.chunks(1024) {
        let block = resampler
            .process_partial(Some(&[chunk]), None)
            .map_err(|_| DictationError::CaptureFailed)?;
        output.extend_from_slice(&block[0]);
    }
    // Flush filter delay with silence, not a repeated tail or a shortened result.
    while output.len() < expected + delay {
        let block = resampler
            .process_partial::<&[f32]>(None, None)
            .map_err(|_| DictationError::CaptureFailed)?;
        output.extend_from_slice(&block[0]);
    }
    output.drain(..delay);
    output.truncate(expected);
    for sample in &mut output {
        *sample = finite_sample(*sample);
    }
    Ok(output)
}

pub fn has_speech_energy(samples: &[f32]) -> bool {
    if samples.is_empty() {
        return false;
    }
    let (squares, peak) = samples
        .iter()
        .fold((0.0_f64, 0.0_f32), |(sum, peak), &sample| {
            let sample = finite_sample(sample);
            (sum + f64::from(sample).powi(2), peak.max(sample.abs()))
        });
    // Compare at the input precision: summing many equal f32 samples in f64
    // must not turn an exactly-threshold signal into silence by rounding.
    let rms = (squares / samples.len() as f64).sqrt() as f32;
    rms >= 0.001 || peak >= 0.005
}

pub fn prepare_audio(input: CapturedAudio) -> Result<Vec<f32>> {
    let samples = to_16khz_mono(input)?;
    if !has_speech_energy(&samples) {
        return Err(DictationError::Silence);
    }
    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tone(rate: u32, frames: usize) -> CapturedAudio {
        CapturedAudio {
            sample_rate: rate,
            mono_samples: (0..frames)
                .map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 0.2)
                .collect(),
        }
    }
    #[test]
    fn bluetooth_8khz_and_44100_and_48000_resample() {
        for rate in [8000, 44100, 48000, 192000] {
            let samples = to_16khz_mono(tone(rate, rate as usize)).unwrap();
            assert_eq!(samples.len(), 16000);
            assert!(samples.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
            assert!(has_speech_energy(&samples));
            let rms = (samples.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>()
                / samples.len() as f64)
                .sqrt();
            assert!(
                (0.13..0.15).contains(&rms),
                "signal energy must survive resampling: {rms}"
            );
            let short = to_16khz_mono(tone(rate, 100)).unwrap();
            assert_eq!(
                short.len(),
                (100.0 * 16000.0 / f64::from(rate)).round() as usize
            );
        }
    }
    #[test]
    fn sixty_seconds_never_exceeds_960000_output_samples() {
        let output = to_16khz_mono(tone(8000, 8000 * 60)).unwrap();
        assert_eq!(output.len(), 960000);
        assert!(to_16khz_mono(tone(8000, 8000 * 60 + 1)).is_err());
    }
    #[test]
    fn silence_requires_both_rms_and_peak_below_threshold() {
        assert!(!has_speech_energy(&[]));
        assert!(!has_speech_energy(&[0.0009; 16000]));
        assert!(has_speech_energy(&[0.001; 16000]));
        let mut peak = vec![0.0; 16000];
        peak[0] = 0.005;
        assert!(has_speech_energy(&peak));
        assert!(to_16khz_mono(tone(16000, 0)).is_err());
    }
}
