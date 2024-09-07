use core::f64::consts::E;

use crate::prelude::*;

pub type FlashMutex = Mutex<CriticalSectionRawMutex, TeenyDataWriter>;

#[task]
pub async fn update_flash(data: TeenyDataMutex, mut flasher: FlashMutex) {
    let mut prev = data.lock().await.clone();

    loop {
        if prev == *data.lock().await {
            info!("Flash is up to date");
        } else {
            info!("Updating flash");

            // TODO: Handle error
            flasher
                .lock()
                .await
                .write(data.lock().await.clone())
                .unwrap();

            prev = data.lock().await.clone();
        }

        Timer::after(Duration::from_millis(5000)).await;
    }
}

use embedded_storage::{ReadStorage, Storage};
use esp_storage::FlashStorage;

pub struct TeenyDataWriter {
    bytes: [u8; core::mem::size_of::<TeenyData>() + 4],
    flash: FlashStorage,
}

impl TeenyDataWriter {
    pub fn new(flash: FlashStorage) -> Self {
        Self {
            flash,
            bytes: [0; { core::mem::size_of::<TeenyData>() + 4 }],
        }
    }

    pub fn write(&mut self, data: TeenyData) -> Result<(), esp_storage::FlashStorageError> {
        let length = bincode::serde::encode_into_slice(
            data,
            &mut self.bytes[4..],
            bincode::config::legacy(),
        )
        //replace with proper error
        .map_err(|_| esp_storage::FlashStorageError::Other(0))?;

        let len = length.to_le_bytes();

        self.bytes[0] = len[0];
        self.bytes[1] = len[1];
        self.bytes[2] = len[2];
        self.bytes[3] = len[3];

        let length = length.next_multiple_of(WORD_SIZE);

        self.flash
            .write(TEENY_LENGTH_OFFSET, &self.bytes[..length + 4])?;

        Ok(())
    }

    pub fn read(&mut self) -> Result<TeenyData, esp_storage::FlashStorageError> {
        let len = self.read_length()?.next_multiple_of(WORD_SIZE);

        if len == 0 || len > self.bytes.len() {
            panic!("Len wrong");
        }

        self.flash.read(TEENY_DATA_OFFSET, &mut self.bytes[..len])?;

        let data =
            bincode::serde::decode_borrowed_from_slice(&self.bytes, bincode::config::legacy())
                .map_err(|_| esp_storage::FlashStorageError::Other(0))?;

        Ok(data)
    }

    fn read_length(&mut self) -> Result<usize, esp_storage::FlashStorageError> {
        let mut length: [u8; 4] = [0; 4];

        self.flash.read(TEENY_LENGTH_OFFSET, &mut length)?;

        Ok(usize::from_le_bytes(length))
    }

    fn write_length(&mut self, length: usize) -> Result<(), esp_storage::FlashStorageError> {
        self.flash
            .write(TEENY_LENGTH_OFFSET, &length.to_le_bytes())?;

        Ok(())
    }
}
