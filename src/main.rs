use cudarc::driver::{CudaDevice, CudaSlice, DriverError};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SIZE: usize = 1 * 1024 * 1024;
fn log(message: &str) {
    println!("{}", message);
    io::stdout().flush().unwrap();
}

fn malloc_thread(device: Arc<CudaDevice>, iteration: Arc<Mutex<u32>>) -> Result<(), DriverError> {
    loop {
        let iter = {
            let mut iter = iteration.lock().unwrap();
            *iter += 1;
            *iter
        };

        log(&format!("Malloc thread: Starting iteration {}", iter));

        let _memory: CudaSlice<u8> = unsafe { device.alloc(SIZE)? };

        // Explicitly drop the memory to deallocate
        drop(_memory);
    }
}

fn memcpy_thread(device: Arc<CudaDevice>, iteration: Arc<Mutex<u32>>) -> Result<(), DriverError> {
    loop {
        let iter = {
            let mut iter = iteration.lock().unwrap();
            *iter += 1;
            *iter
        };

        log(&format!("Memcpy thread: Starting iteration {}", iter));

        let mut host_data = vec![0u8; SIZE];

        let mut device_data: CudaSlice<u8> = unsafe { device.alloc(SIZE) }?;

        log("Memcpy thread: Async memcpy started");
        device.dtoh_sync_copy_into(&mut device_data, &mut host_data)?;

        // Explicitly drop to deallocate
        drop(device_data);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_count = CudaDevice::count()?;
    if device_count < 2 {
        return Err("This program requires at least 2 CUDA devices.".into());
    }

    let device0 = CudaDevice::new(0)?;
    let device1 = CudaDevice::new(1)?;

    let malloc_iteration = Arc::new(Mutex::new(0u32));
    let memcpy_iteration = Arc::new(Mutex::new(0u32));

    let malloc_device = Arc::clone(&device1);
    let malloc_iter = Arc::clone(&malloc_iteration);
    let _malloc_handle = thread::spawn(move || malloc_thread(malloc_device, malloc_iter));

    let memcpy_device = Arc::clone(&device0);
    let memcpy_iter = Arc::clone(&memcpy_iteration);
    let _memcpy_handle = thread::spawn(move || memcpy_thread(memcpy_device, memcpy_iter));

    println!("Press Enter to stop the program...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // The threads will continue running. In a real application, you'd want to implement
    // a proper shutdown mechanism. For this example, we'll just wait a bit and then exit.
    thread::sleep(Duration::from_secs(1));

    println!("Program completed");
    Ok(())
}
