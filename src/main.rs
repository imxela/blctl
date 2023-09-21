use std::{
    error::Error,
    future::pending,
    fs::OpenOptions,
    io::{Read, Write}
};

use zbus::{ConnectionBuilder, dbus_interface};

struct BacklightController {
    kernel_brightness_fp: String,
    kernel_max_brightness_fp: String
}

#[dbus_interface(name = "me.xela.blctl1")]
impl BacklightController {
    /// Increases the backlight brightness level.
    ///
    /// # Arguments
    ///
    /// * `amount` - The backlight brightness level to increase
    /// by as a percentage of the maximum supported backlight
    /// brightness level (see [max]).
    async fn increase(&mut self, amount: u32) {
        println!("Received 'increase(amount: {})' message", amount);

        // let current = self.get().await;
        let current = ki_read(&self.kernel_brightness_fp)
            .trim()
            .parse::<u32>()
            .expect(format!(
                "failed to parse kernel interface ({}) data to u32",
                self.kernel_brightness_fp
            ).as_str());

        // let max = self.max().await;
        let max = ki_read(&self.kernel_max_brightness_fp)
            .trim()
            .parse::<u32>()
            .expect(format!(
                "failed to parse kernel interface ({}) to u32",
                &self.kernel_max_brightness_fp
            ).as_str());

        let fraction = max as f32 / 100f32;
        let actual_amount = fraction * amount as f32;

        let mut new_current = current + actual_amount as u32;
        
        if new_current > max {
            new_current = max;
        }

        ki_write(
            &self.kernel_brightness_fp,
            new_current.to_string()
        );
    }

    /// Decreases the backlight brightness level.
    /// 
    /// # Arguments
    ///
    /// * `amount` - The backlight brightness level to reduce
    /// by as a percentage of the maximum supported backlight
    /// brightness level (see [max]).
    async fn decrease(&mut self, amount: u32) {
        println!("Received 'decrease(amount: {})' message", amount);

        // let mut current = self.get().await;
        let mut current = ki_read(&self.kernel_brightness_fp)
            .trim()
            .parse::<u32>()
            .expect(format!(
                "failed to parse kernel interface ({}) data to u32",
                self.kernel_brightness_fp
            ).as_str());
        
        // let max = self.max().await;
        let max = ki_read(&self.kernel_max_brightness_fp)
            .trim()
            .parse::<u32>()
            .expect(format!(
                "failed to parse kernel interface ({}) data to u32",
                self.kernel_max_brightness_fp
            ).as_str());

        let fraction = max as f32 / 100f32;
        let actual_amount = fraction * amount as f32;

        // Prevent u32 underflow
        if current < actual_amount as u32 {
            current = actual_amount as u32;
        }

        let new_current = current - actual_amount as u32;
        ki_write(
            &self.kernel_brightness_fp,
            new_current.to_string()
        );
    }

    /// Sets the backlight brightness level to the specified
    /// value.
    ///
    /// # Arguments
    ///
    /// * `value` - The brightness level to set the backlight
    /// to. Clamped between 0 and the maximum supported
    /// backlight brightness level (see [max]).
    async fn set(&mut self, mut value: u32) {
        println!("Recieved 'set(value: {})' message", value);

        let max = self.max().await;
        if value > max {
            value = max;
        }

        ki_write(&self.kernel_brightness_fp, value.to_string());
    }

    /// Returns the current backlight brightness level.
    async fn get(&mut self) -> u32 {
        println!("Recieved 'get()' message");

        ki_read(&self.kernel_brightness_fp)
            .trim()
            .parse::<u32>()
            .expect(format!(
                "failed to parse kernel interface ({}) data to u32",
                &self.kernel_brightness_fp
            ).as_str())
    }

    /// Returns the maximum support backlight brightness level.
    async fn max(&mut self) -> u32 {
        println!("Recieved 'max()' mesage");

        ki_read(&self.kernel_max_brightness_fp)
            .trim()
            .parse::<u32>()
            .expect(format!(
                "failed to parse kernel interface ({}) data to u32",
                &self.kernel_max_brightness_fp
            ).as_str())
    }
}

fn ki_read(filepath: &String) -> String {
    let mut file = OpenOptions::new()
        .read(true)
        .create(false)
        .open(&filepath)
        .expect(format!(
                "failed to open kernel interface ({}) for reading",
                &filepath
                ).as_str());

    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect(format!(
                "failed to read data from kernel interface ({})",
                &filepath
                ).as_str());

    println!(
        "Read data ({:?}) from kernel interface ({})", 
        &data.as_bytes(), 
        &filepath
    );

    data
}

fn ki_write(filepath: &String, data: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(false)
        .open(&filepath)
        .expect(format!(
            "failed to open kernel interface ({}) for writing",
            &filepath
        ).as_str());

    file.write_all(data.trim().as_bytes())
            .expect(format!(
                "failed to write data to kernel interface ({})",
                &filepath
            ).as_str());

    println!(
        "Wrote data ({:?}) to kernel interface ({})",
        &data.as_bytes(),
        &filepath
    );
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Running blctl service");

    println!("Creating BacklightController");

    let bl_controller = BacklightController {
        kernel_brightness_fp: "/sys/class/backlight/amdgpu_bl0/brightness".to_string(),
        kernel_max_brightness_fp: "/sys/class/backlight/amdgpu_bl0/max_brightness".to_string()
    };

    println!("Building connection");

    let _conn = ConnectionBuilder::system()?
        .name("me.xela.blctl")?
        .serve_at("/me/xela/blctl", bl_controller)?
        .build()
        .await?;

    println!("Awaiting message");

    pending::<()>().await;

    Ok(())
}
