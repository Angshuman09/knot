use common::{LogEntry};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct StorageEngine{
    wal_path: PathBuf,
    writer: BufWriter<File>
}

impl StorageEngine{
    pub fn open(data_dir: impl AsRef<Path>) -> std::io::Result<Self>{
        let dir = data_dir.as_ref();
        create_dir_all(dir)?;

        let wal_path = dir.join("wal.log");
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&wal_path)?;
        
        Ok(Self{
            wal_path,
            writer: BufWriter::new(file)
        })
    }

    pub fn append_entry(&mut self, entry: &LogEntry)-> std::io::Result<()>{
        let json = serde_json::to_vec(entry).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let length = (json.len() as u32).to_be_bytes();
        self.writer.write_all(&length)?;
        self.writer.write_all(&json)?;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;

        Ok(())
    }

    pub fn recover_entries(&self) -> std::io::Result<Vec<LogEntry>>{
        let mut file = match File::open(&self.wal_path){
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };

        file.seek(SeekFrom::Start(0))?;

        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop{
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes){
                Ok(()) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e)
            }

            let length = u32::from_be_bytes(len_bytes) as usize;
            let mut payload = vec![0u8; length];
            reader.read_exact(&mut payload)?;

            let entry: LogEntry = serde_json::from_slice(&payload)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            entries.push(entry);
        }

        Ok(entries)
    }
}