use std::fs;
use std::path::PathBuf;
use zstd;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

pub struct BackupService {
    db_path: String,
    backup_dir: PathBuf,
    max_backups: usize,
}

impl BackupService {
    pub fn new(db_path: String, backup_dir: PathBuf, max_backups: usize) -> Self {
        Self { db_path, backup_dir, max_backups }
    }

    pub async fn create_backup(&self) -> Result<String> {
        // Create backup directory if it doesn't exist
        fs::create_dir_all(&self.backup_dir)?;

        // Generate backup filename
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_id = Uuid::new_v4();
        let filename = format!("backup_{}_{}.db.zst", timestamp, backup_id);
        let backup_path = self.backup_dir.join(&filename);

        // Read database file
        let db_data = fs::read(&self.db_path)?;

        // Compress with zstd
        let compressed = zstd::encode_all(db_data.as_slice(), 3)?;

        // Write compressed backup
        fs::write(&backup_path, compressed)?;

        // Rotate old backups
        self.rotate_backups()?;

        Ok(filename)
    }

    pub async fn restore_backup(&self, filename: &str) -> Result<()> {
        let backup_path = self.backup_dir.join(filename);

        // Read compressed backup
        let compressed = fs::read(&backup_path)?;

        // Decompress
        let decompressed = zstd::decode_all(compressed.as_slice())?;

        // Write to database file
        fs::write(&self.db_path, decompressed)?;

        Ok(())
    }

    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        self.list_backups_blocking()
    }

    fn rotate_backups(&self) -> Result<()> {
        let backups = self.list_backups_blocking()?;
        
        if backups.len() > self.max_backups {
            for backup in backups.iter().skip(self.max_backups) {
                let backup_path = self.backup_dir.join(&backup.filename);
                fs::remove_file(backup_path)?;
            }
        }

        Ok(())
    }

    fn list_backups_blocking(&self) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();

        if !self.backup_dir.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "zst").unwrap_or(false) {
                let metadata = fs::metadata(&path)?;
                let created = metadata.created()?;
                let size = metadata.len();
                
                backups.push(BackupInfo {
                    filename: path.file_name().unwrap().to_string_lossy().to_string(),
                    created_at: chrono::DateTime::<chrono::Utc>::from(created),
                    size,
                });
            }
        }

        // Sort by creation date, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub size: u64,
}
