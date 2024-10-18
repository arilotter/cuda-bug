use cudarc::driver::{result, sys, CudaDevice, DriverError};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const SIZE: usize = 1 * 1024 * 1024;

fn log(message: &str) {
    println!("{}", message);
    io::stdout().flush().unwrap();
}

fn malloc_thread(stream: sys::CUstream) -> Result<(), DriverError> {
    let mut iter = 0;
    loop {
        log(&format!("Malloc thread: Starting iteration {}", iter));

        unsafe {
            let mem =
                cudarc::driver::result::malloc_async(stream, SIZE * std::mem::size_of::<u8>())?;
            result::free_async(mem, stream)?;
        };
        iter += 1;
    }
}

fn memcpy_thread(stream: sys::CUstream) -> Result<(), DriverError> {
    let mut iter = 0;
    loop {
        log(&format!("Memcpy thread: Starting iteration {}", iter));

        let mut host_data = vec![0u8; SIZE];

        let device_data = unsafe {
            cudarc::driver::result::malloc_async(stream, SIZE * std::mem::size_of::<u8>())?
        };

        log("Memcpy thread: Async memcpy started");
        unsafe { result::memcpy_dtoh_async(&mut host_data, device_data, stream) }?;

        // Explicitly drop to deallocate
        unsafe {
            result::free_async(device_data, stream)?;
        }
        iter += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_count = CudaDevice::count()?;
    if device_count < 2 {
        return Err("This program requires at least 2 CUDA devices.".into());
    }

    let device0 = CudaDevice::new(0)?;
    let device1 = CudaDevice::new(1)?;

    let _malloc_handle = thread::spawn(move || malloc_thread(device0.cu_stream().clone()));

    let _memcpy_handle = thread::spawn(move || memcpy_thread(device1.cu_stream().clone()));

    println!("Press Enter to stop the program...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // The threads will continue running. In a real application, you'd want to implement
    // a proper shutdown mechanism. For this example, we'll just wait a bit and then exit.
    thread::sleep(Duration::from_secs(1));

    println!("Program completed");
    Ok(())
}
