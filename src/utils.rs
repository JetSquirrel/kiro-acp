//! Utility functions

/// Helper functions for the ACP adapter
pub mod stream {
    use std::io::Read;
    use tokio::io::AsyncWriteExt;

    /// Convert a reader to an async reader
    pub fn to_async_reader<R: Read + Send + 'static>(sync_reader: R) -> tokio::io::DuplexStream {
        let (mut writer, async_reader) = tokio::io::duplex(8192);

        tokio::task::spawn_blocking(move || {
            let mut sync_reader = sync_reader;
            let mut buffer = vec![0u8; 8192];

            loop {
                match sync_reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tokio::runtime::Handle::current()
                            .block_on(writer.write_all(&buffer[..n]))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        async_reader
    }
}
