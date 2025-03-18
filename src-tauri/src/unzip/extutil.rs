use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    Gz,
    Bz2,
    SevenZip,
    Rar,
    Unknown,
}

// -------------------------
// 扩展名匹配逻辑
// -------------------------

impl ArchiveFormat {
    pub fn from_path(path: &Path) -> Self {
        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_lowercase(),
            None => return ArchiveFormat::Unknown,
        };

        // 优先检查复合扩展名
        if filename.ends_with(".tar.gz") {
            ArchiveFormat::TarGz
        } else if filename.ends_with(".tar.bz2") {
            ArchiveFormat::TarBz2
        } else if filename.ends_with(".tar.xz") {
            ArchiveFormat::TarXz
        } else if filename.ends_with(".tgz") {
            ArchiveFormat::TarGz
        } else if filename.ends_with(".tbz2") || filename.ends_with(".tbz") {
            ArchiveFormat::TarBz2
        } else {
            // 处理单扩展名
            match path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase())
            {
                Some(ext) if ext == "zip" => ArchiveFormat::Zip,
                Some(ext) if ext == "tar" => ArchiveFormat::Tar,
                Some(ext) if ext == "gz" => ArchiveFormat::TarGz,
                Some(ext) if ext == "bz2" => ArchiveFormat::TarBz2,
                Some(ext) if ext == "7z" => ArchiveFormat::SevenZip,
                Some(ext) if ext == "rar" => ArchiveFormat::Rar,
                _ => ArchiveFormat::Unknown,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format_from_path() {
        // 测试复合扩展名
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tar.gz.zip")),
            ArchiveFormat::Zip
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tar.bz2")),
            ArchiveFormat::TarBz2
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tar.xz")),
            ArchiveFormat::TarXz
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tgz")),
            ArchiveFormat::TarGz
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tbz2")),
            ArchiveFormat::TarBz2
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tbz")),
            ArchiveFormat::TarBz2
        );

        // 测试单扩展名
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.zip")),
            ArchiveFormat::Zip
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.tar")),
            ArchiveFormat::Tar
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.gz")),
            ArchiveFormat::Gz
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.bz2")),
            ArchiveFormat::Bz2
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.7z")),
            ArchiveFormat::SevenZip
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.rar")),
            ArchiveFormat::Rar
        );

        // 测试未知格式和边界情况
        assert_eq!(
            ArchiveFormat::from_path(Path::new("test.unknown")),
            ArchiveFormat::Unknown
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new("")),
            ArchiveFormat::Unknown
        );
        assert_eq!(
            ArchiveFormat::from_path(Path::new(".hidden")),
            ArchiveFormat::Unknown
        );
    }
}
