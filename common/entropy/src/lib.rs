use blake3::Hasher;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn collect_hardware_entropy() -> [u8; 32] {
    let mut hasher = Hasher::new();

    #[cfg(target_os = "android")]
    collect_mobile_entropy(&mut hasher);

    #[cfg(target_os = "ios")]
    collect_mobile_entropy(&mut hasher);

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    collect_desktop_entropy(&mut hasher);

    #[cfg(feature = "embedded")]
    collect_embedded_entropy(&mut hasher);

    // Fallback entropy
    collect_fallback_entropy(&mut hasher);

    let mut result = [0u8; 32];
    result.copy_from_slice(hasher.finalize().as_bytes());
    result
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn collect_mobile_entropy(hasher: &mut Hasher) {
    // Accelerometer noise
    if let Ok(accel) = read_accelerometer() {
        hasher.update(&accel);
    }

    // Microphone ambient noise
    if let Ok(mic) = sample_microphone_noise() {
        hasher.update(&mic);
    }

    // Light sensor variations
    if let Ok(light) = read_light_sensor() {
        hasher.update(&light.to_le_bytes());
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn collect_desktop_entropy(hasher: &mut Hasher) {
    // CPU timing jitter
    for _ in 0..100 {
        let start = precise_time_ns();
        volatile_nop();
        let jitter = precise_time_ns().wrapping_sub(start);
        hasher.update(&jitter.to_le_bytes());
    }

    // CPU temperature if available
    if let Ok(temp) = read_cpu_temperature() {
        hasher.update(&temp.to_le_bytes());
    }

    // Microphone if available
    if let Ok(mic) = sample_microphone_noise() {
        hasher.update(&mic);
    }
}

#[cfg(feature = "embedded")]
fn collect_embedded_entropy(hasher: &mut Hasher) {
    // ADC on floating pin
    if let Ok(adc) = read_floating_adc() {
        hasher.update(&adc.to_le_bytes());
    }

    // Timing jitter
    for _ in 0..50 {
        let jitter = timing_jitter();
        hasher.update(&jitter.to_le_bytes());
    }

    // Temperature sensor
    if let Ok(temp) = read_temperature_sensor() {
        hasher.update(&temp.to_le_bytes());
    }
}

fn collect_fallback_entropy(hasher: &mut Hasher) {
    // System time with nanosecond precision
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    hasher.update(&now.as_nanos().to_le_bytes());

    // Process ID
    hasher.update(&std::process::id().to_le_bytes());

    // Thread ID hash (stable alternative)
    let thread_id = format!("{:?}", std::thread::current().id());
    hasher.update(thread_id.as_bytes());

    // Stack address entropy
    let stack_var = 0u8;
    hasher.update(&(&stack_var as *const u8 as usize).to_le_bytes());
}

// Platform-specific implementations
#[cfg(any(target_os = "android", target_os = "ios"))]
fn read_accelerometer() -> Result<[u8; 12], ()> {
    // Platform-specific accelerometer reading
    Err(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn read_light_sensor() -> Result<f32, ()> {
    // Platform-specific light sensor reading
    Err(())
}

fn sample_microphone_noise() -> Result<[u8; 1024], ()> {
    // Platform-specific microphone sampling
    Err(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn read_cpu_temperature() -> Result<f32, ()> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
            .map(|s| s.trim().parse::<f32>().unwrap_or(0.0) / 1000.0)
            .map_err(|_| ())
    }
    #[cfg(target_os = "macos")]
    Err(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn precise_time_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[inline(never)]
fn volatile_nop() {
    unsafe {
        std::ptr::read_volatile(&0u8);
    }
}

#[cfg(feature = "embedded")]
fn read_floating_adc() -> Result<u16, ()> {
    // Embedded ADC reading on floating pin
    Err(())
}

#[cfg(feature = "embedded")]
fn timing_jitter() -> u32 {
    // Embedded timing jitter measurement
    0
}

#[cfg(feature = "embedded")]
fn read_temperature_sensor() -> Result<f32, ()> {
    // Embedded temperature sensor reading
    Err(())
}
