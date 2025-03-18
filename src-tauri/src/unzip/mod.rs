use bzip2::read::BzDecoder;
use extutil::ArchiveFormat;
use flate2::read::GzDecoder;
use itertools::Itertools;
use plustree::TreeNode;
use serde::Serialize;
use serde_json::json;
use std::io::{BufReader, Read, Seek};
use std::path::PathBuf;
use std::{fs::File, path::Path};
use tar::Archive;
use unrar::error::UnrarError;
use xz2::read::XzDecoder;
use zip::result::ZipError;
use zip::ZipArchive;

pub mod extutil;
pub mod plustree;

#[derive(Debug, Serialize, Clone, Default)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,
    pub parent_path: Option<String>,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
}

pub type ResultR<T, E = ArchiveError> = core::result::Result<T, E>;

// -------------------------
// 类型定义
// -------------------------

/// 统一错误类型
#[derive(Debug, thiserror::Error, Serialize)]
pub enum ArchiveError {
    #[error("Error: {0}")]
    MsgError(String),
}

// -------------------------
// 核心 Trait 设计
// -------------------------

pub trait ArchiveHandler {
    /// 获取文件列表
    fn list_files(&mut self) -> ResultR<Vec<TreeNode<ArchiveEntry>>>;
    /// 执行解压
    fn extract(&mut self, target_dir: &std::path::Path) -> ResultR<()>;
}

// -------------------------
// 具体类型实现（空结构体示例）
// -------------------------

pub struct ZipHandler {
    // 内部状态存储
    archive_path: PathBuf,

    password: String,
}

impl ArchiveHandler for ZipHandler {
    fn list_files(&mut self) -> ResultR<Vec<TreeNode<ArchiveEntry>>> {
        let custom_error = |e: ZipError| match e {
            zip::result::ZipError::FileNotFound => {
                ArchiveError::MsgError(String::from("文件未找到"))
            }
            zip::result::ZipError::InvalidPassword => {
                ArchiveError::MsgError(String::from("密码错误"))
            }
            zip::result::ZipError::UnsupportedArchive(estr) => match estr {
                zip::result::ZipError::PASSWORD_REQUIRED => {
                    ArchiveError::MsgError(String::from("密码错误"))
                }
                _ => ArchiveError::MsgError(String::from("打开文件错误")),
            },
            _ => ArchiveError::MsgError(String::from("打开文件错误")),
        };
        let path = &self.archive_path;
        let file = File::open(path).map_err(|e| ArchiveError::MsgError(e.to_string()))?;
        let mut archive = ZipArchive::new(file).map_err(custom_error)?;
        let mut entries = Vec::new();

        for i in 0..archive.len() {
            let file = if self.password.is_empty() {
                archive.by_index(i).map_err(custom_error)?
            } else {
                archive
                    .by_index_decrypt(i, self.password.as_bytes())
                    .map_err(custom_error)?
            };

            let full_path = file.name().to_string();
            let is_dir = file.is_dir();
            let modified = file.last_modified().map(|m| m.to_string());
            let name = full_path.split('/').last().unwrap_or("").to_string();
            // 使用 Path::new 创建一个 Path 对象
            let path = Path::new(&full_path);
            let mut parent_path = Option::None;
            // 获取目录路径
            if let Some(parent) = path.parent() {
                parent_path = Some(
                    parent
                        .to_string_lossy()
                        .to_string()
                        .trim_end_matches('/')
                        .to_string()
                        + "/",
                );
            }
            // 跳过 macOS 系统文件
            if full_path.starts_with("__MACOSX") {
                continue;
            }
            entries.push(ArchiveEntry {
                name,
                path: full_path.clone(),
                parent_path,
                size: if is_dir { 0 } else { file.size() },
                is_dir,
                modified,
            });
        }

        let tree = plustree::TreeNode::build_tree(
            entries,
            String::from("/"),
            |i| i.path.clone(),
            |i| i.parent_path.clone().unwrap().clone(),
        );
        Ok(tree)
    }

    fn extract(&mut self, target_dir: &std::path::Path) -> ResultR<()> {
        let custom_error = |e: ZipError| match e {
            zip::result::ZipError::FileNotFound => {
                ArchiveError::MsgError(String::from("文件未找到"))
            }
            zip::result::ZipError::InvalidPassword => {
                ArchiveError::MsgError(String::from("密码错误"))
            }
            zip::result::ZipError::UnsupportedArchive(estr) => match estr {
                zip::result::ZipError::PASSWORD_REQUIRED => {
                    ArchiveError::MsgError(String::from("密码错误"))
                }
                _ => ArchiveError::MsgError(String::from("打开文件错误")),
            },
            _ => ArchiveError::MsgError(String::from("解压文件错误")),
        };

        let path = &self.archive_path;
        let file = File::open(path).map_err(|e| ArchiveError::MsgError(e.to_string()))?;
        let mut archive = ZipArchive::new(file).map_err(custom_error)?;

        // 创建目标目录（如果不存在）
        if !target_dir.exists() {
            std::fs::create_dir_all(target_dir)
                .map_err(|e| ArchiveError::MsgError(format!("创建目标目录失败: {}", e)))?;
        }

        // 遍历并解压所有文件
        for i in 0..archive.len() {
            let mut file = if self.password.is_empty() {
                archive.by_index(i).map_err(custom_error)?
            } else {
                archive
                    .by_index_decrypt(i, self.password.as_bytes())
                    .map_err(custom_error)?
            };

            let full_path = file.name().to_string();

            // 跳过 macOS 系统文件
            if full_path.starts_with("__MACOSX") {
                continue;
            }

            // 构建目标路径
            let outpath = target_dir.join(Path::new(&full_path));

            // 创建目录结构
            if (*file.name()).ends_with('/') {
                std::fs::create_dir_all(&outpath)
                    .map_err(|e| ArchiveError::MsgError(format!("创建目录失败: {}", e)))?;
            } else {
                // 确保父目录存在
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            ArchiveError::MsgError(format!("创建父目录失败: {}", e))
                        })?;
                    }
                }

                // 创建文件并写入内容
                let mut outfile = std::fs::File::create(&outpath)
                    .map_err(|e| ArchiveError::MsgError(format!("创建文件失败: {}", e)))?;

                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| ArchiveError::MsgError(format!("写入文件失败: {}", e)))?;
            }

            // 设置文件修改时间（如果可用）
            #[cfg(unix)]
            if let Some(mode) = file.unix_mode() {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&outpath) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(mode);
                    let _ = std::fs::set_permissions(&outpath, perms);
                }
            }
        }

        Ok(())
    }
}

// 其他格式类似

pub struct TarHandler {
    // 内部状态存储
    archive_path: std::path::PathBuf,
    password: String,
    archive_format: ArchiveFormat,
}

impl ArchiveHandler for TarHandler {
    fn list_files(&mut self) -> ResultR<Vec<TreeNode<ArchiveEntry>>> {
        let file =
            File::open(&self.archive_path).map_err(|e| ArchiveError::MsgError(e.to_string()))?;

        // 根据不同格式创建对应的解码器
        let reader: Box<dyn std::io::Read> = match self.archive_format {
            ArchiveFormat::TarGz => Box::new(GzDecoder::new(file)),
            ArchiveFormat::TarXz => Box::new(XzDecoder::new(file)),
            ArchiveFormat::TarBz2 => Box::new(BzDecoder::new(file)),
            ArchiveFormat::Tar => Box::new(file),
            _ => return Err(ArchiveError::MsgError("不支持的格式".to_string())),
        };

        let mut archive = Archive::new(reader);

        let mut entries = Vec::new();

        for entry in archive
            .entries()
            .map_err(|e| ArchiveError::MsgError(e.to_string()))?
        {
            let entry = entry.map_err(|e| {
                println!("{:#?}", e);
                ArchiveError::MsgError(e.to_string())
            })?;
            let path = entry
                .path()
                .map_err(|e| ArchiveError::MsgError(e.to_string()))?;
            let full_path = path.to_string_lossy().to_string();

            // 跳过 macOS 系统文件
            if full_path.starts_with("__MACOSX") {
                continue;
            }

            let is_dir = entry.header().entry_type().is_dir();
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            let mut parent_path = None;
            if let Some(parent) = path.parent() {
                parent_path = Some(
                    parent
                        .to_string_lossy()
                        .to_string()
                        .trim_end_matches('/')
                        .to_string()
                        + "/",
                );
            }

            entries.push(ArchiveEntry {
                name,
                path: full_path.clone(),
                parent_path,
                size: entry.header().size().unwrap_or(0),
                is_dir,
                modified: entry
                    .header()
                    .mtime()
                    .map(|m| {
                        chrono::DateTime::from_timestamp(m as i64, 0)
                            .map_or(None, |t| Some(t.format("%Y-%m-%d %H:%M:%S").to_string()))
                    })
                    .unwrap_or(None),
            });
        }
        println!("{:#?}", entries);

        let tree = plustree::TreeNode::build_tree(
            entries,
            String::from("/"),
            |i| i.path.clone(),
            |i| i.parent_path.clone().unwrap_or_default(),
        );
        Ok(tree)
    }

    fn extract(&mut self, target_dir: &std::path::Path) -> ResultR<()> {
        let file =
            File::open(&self.archive_path).map_err(|e| ArchiveError::MsgError(e.to_string()))?;

        let reader: Box<dyn std::io::Read> = match self.archive_format {
            ArchiveFormat::TarGz => Box::new(GzDecoder::new(file)),
            ArchiveFormat::TarXz => Box::new(XzDecoder::new(file)),
            ArchiveFormat::TarBz2 => Box::new(BzDecoder::new(file)),
            ArchiveFormat::Tar => Box::new(file),
            _ => return Err(ArchiveError::MsgError("不支持的格式".to_string())),
        };
        let mut archive = Archive::new(reader);

        archive
            .unpack(target_dir)
            .map_err(|e| ArchiveError::MsgError(e.to_string()))?;

        Ok(())
    }
}

pub struct SevenZipHandler {
    // 内部状态存储
    archive_path: std::path::PathBuf,
    password: String,
}

#[test]
fn testf() -> () {
    let path = std::path::Path::new("/Users/apple/Downloads/Compressed/test.7z");
    let mut handle = SevenZipHandler {
        archive_path: path.to_path_buf(),
        password: String::from("3"),
    };

    let ff = handle.list_files();
    println!("{:#?}", ff);
}

impl ArchiveHandler for SevenZipHandler {
    fn list_files(&mut self) -> ResultR<Vec<TreeNode<ArchiveEntry>>> {
        let archive_path = &self.archive_path.clone().to_string_lossy().to_string();

        let custom_err = |e: sevenz_rust::Error| {
            if e.to_string().contains("Password") {
                return ArchiveError::MsgError(String::from("密码错误"));
            } else {
                return ArchiveError::MsgError(format!("打开文件错误: {}", e));
            }
        };
        let mut sz = sevenz_rust::SevenZReader::open(
            archive_path,
            self.password
                .is_empty()
                .then(|| sevenz_rust::Password::empty())
                .unwrap_or(sevenz_rust::Password::from(self.password.as_str())),
        )
        .map_err(custom_err)?;
        // 使用sevenz-rust库打开7z文件
        let mut entries = Vec::new();

        let mut add_entry = |entry: &sevenz_rust::SevenZArchiveEntry| {
            // 遍历所有文件条目
            let full_path = entry.name().to_string();

            // 跳过 macOS 系统文件
            // if full_path.starts_with("__MACOSX") {
            //     return;
            // }

            let path = Path::new(&full_path);
            let is_dir = entry.is_directory();
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            let mut parent_path = None;
            if let Some(parent) = path.parent() {
                parent_path = Some(
                    parent
                        .to_string_lossy()
                        .to_string()
                        .trim_end_matches('/')
                        .to_string()
                        + "/",
                );
            }

            // 获取修改时间
            let modified =
                chrono::DateTime::from_timestamp(entry.last_modified_date().to_unix_time(), 0)
                    .map_or(None, |t| Some(t.format("%Y-%m-%d %H:%M:%S").to_string()));

            entries.push(ArchiveEntry {
                name,
                path: if is_dir {
                    full_path.clone() + "/"
                } else {
                    full_path.clone()
                },
                parent_path,
                size: entry.size(),
                is_dir,
                modified,
            });
        };
        sz.for_each_entries(|entry, _reader| {
            add_entry(entry);
            Ok(true)
        })
        .map_err(custom_err)?;

        println!("{:#?}", json!(entries));
        // 构建缺失的文件夹
        let parents_dir = entries
            .clone()
            .into_iter()
            .map(|x| x.parent_path.unwrap())
            .unique()
            .collect::<Vec<_>>();

        println!("parents_dir:{:#?}", json!(parents_dir));

        parents_dir.iter().for_each(|x| {
            let mut parent_path = None;
            if let Some(parent) = Path::new(x).parent() {
                parent_path = Some(
                    parent
                        .to_string_lossy()
                        .to_string()
                        .trim_end_matches('/')
                        .to_string()
                        + "/",
                );
            }

            entries.push(ArchiveEntry {
                name: String::from(""),
                path: x.clone(),
                parent_path: parent_path,
                size: 0,
                is_dir: true,
                modified: None,
            });
        });

        let mut entries: Vec<_> = entries
            .into_iter()
            .unique_by(|item| item.path.clone())
            .collect();

        let tree = plustree::TreeNode::build_tree(
            entries,
            String::from("/"),
            |i| i.path.clone(),
            |i| i.parent_path.clone().unwrap_or_default(),
        );
        Ok(tree)
    }

    fn extract(&mut self, target_dir: &std::path::Path) -> ResultR<()> {
        let archive_path = &self.archive_path.clone().to_string_lossy().to_string();
        let custom_err = |e: sevenz_rust::Error| {
            if e.to_string().contains("Password") {
                return ArchiveError::MsgError(String::from("密码错误"));
            } else {
                return ArchiveError::MsgError(format!("打开文件错误: {}", e));
            }
        };

        // sevenz_rustdefault_entry_extract_fn
        sevenz_rust::decompress_file_with_password(
            archive_path,
            target_dir.to_str().unwrap(),
            self.password
                .is_empty()
                .then(|| sevenz_rust::Password::empty())
                .unwrap_or(sevenz_rust::Password::from(self.password.as_str())),
        )
        .map_err(custom_err)?;
        Ok(())
    }
}

pub struct RarHandler {
    // 内部状态存储
    archive_path: std::path::PathBuf,
    password: String,
}

impl ArchiveHandler for RarHandler {
    fn list_files(&mut self) -> ResultR<Vec<TreeNode<ArchiveEntry>>> {
        let archive_path = &self.archive_path.clone().to_string_lossy().to_string();
        // 使用unrar库打开RAR文件
        let archive = if self.password.is_empty() {
            unrar::Archive::new(archive_path)
        } else {
            unrar::Archive::with_password(archive_path, &self.password)
        };

        let mut entries = Vec::new();

        // 列出所有文件
        match archive.open_for_listing() {
            Ok(list) => {
                for entry in list {
                    match entry {
                        Ok(entry) => {
                            let full_path = &entry.filename;

                            // 跳过 macOS 系统文件
                            if full_path.starts_with("__MACOSX") {
                                continue;
                            }

                            let path = Path::new(&full_path);
                            let is_dir = entry.is_directory();
                            let name = path
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();

                            let mut parent_path = None;
                            if let Some(parent) = path.parent() {
                                parent_path = Some(
                                    parent
                                        .to_string_lossy()
                                        .to_string()
                                        .trim_end_matches('/')
                                        .to_string()
                                        + "/",
                                );
                            }

                            // 转换时间格式
                            let modified =
                                chrono::DateTime::from_timestamp(entry.file_time as i64, 0)
                                    .map_or(None, |dt| {
                                        Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    });

                            entries.push(ArchiveEntry {
                                name,
                                path: format!(
                                    "{}{}",
                                    full_path.clone().to_string_lossy().to_string(),
                                    if is_dir { "/" } else { "" }
                                ),
                                parent_path,
                                size: entry.unpacked_size,
                                is_dir,
                                modified,
                            });
                        }
                        Err(e) => return Err(ArchiveError::MsgError(e.to_string())),
                    }
                }
            }
            Err(e) => match e {
                UnrarError { code, when } => match code {
                    unrar::error::Code::MissingPassword | unrar::error::Code::BadPassword => {
                        return Err(ArchiveError::MsgError(String::from("密码错误")))
                    }
                    _ => return Err(ArchiveError::MsgError(format!("发生错误: {}", e,))),
                },
            },
        }

        let tree = plustree::TreeNode::build_tree(
            entries,
            String::from("/"),
            |i| i.path.clone(),
            |i| i.parent_path.clone().unwrap_or_default(),
        );
        Ok(tree)
    }

    fn extract(&mut self, target_dir: &std::path::Path) -> ResultR<()> {
        // // 使用unrar库解压文件

        let archive_path = &self.archive_path.clone().to_string_lossy().to_string();
        let archive = if self.password.is_empty() {
            unrar::Archive::new(archive_path)
        } else {
            unrar::Archive::with_password(archive_path, &self.password)
        };

        let merr = |e: UnrarError| match e {
            UnrarError { code, when } => match code {
                unrar::error::Code::MissingPassword | unrar::error::Code::BadPassword => {
                    return ArchiveError::MsgError(String::from("密码错误"))
                }
                _ => return ArchiveError::MsgError(format!("发生错误: {}", e,)),
            },
        };

        let mut archive = archive.open_for_processing().map_err(merr)?;
        while let Some(header) = archive.read_header().map_err(merr)? {
            let fname = header.entry().filename.to_string_lossy();

            let is_file = header.entry().is_file();
            let target_file = format!("{}/{}", target_dir.to_string_lossy().to_string(), fname);

            archive = if is_file {
                // 跳过 macOS 系统文件
                if fname.starts_with("__MACOSX") {
                    header.skip().map_err(merr)?
                } else {
                    header.extract_to(target_file).map_err(merr)?
                }
            } else {
                header.skip().map_err(merr)?
            };
        }
        Ok(())
    }
}

// -------------------------
// 工厂模式设计
// -------------------------

pub fn create_handler(
    path: &std::path::Path,
    format: ArchiveFormat,
    password: String,
) -> Box<dyn ArchiveHandler> {
    match format {
        ArchiveFormat::Zip => Box::new(ZipHandler {
            archive_path: path.to_path_buf(),
            password,
        }),
        ArchiveFormat::Tar
        | ArchiveFormat::TarXz
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::Gz
        | ArchiveFormat::Bz2 => Box::new(TarHandler {
            archive_path: path.to_path_buf(),
            password,
            archive_format: format,
        }),
        ArchiveFormat::SevenZip => Box::new(SevenZipHandler {
            archive_path: path.to_path_buf(),
            password,
        }),
        ArchiveFormat::Rar => Box::new(RarHandler {
            archive_path: path.to_path_buf(),
            password,
        }),
        _ => Box::new(ZipHandler {
            archive_path: path.to_path_buf(),
            password,
        }),
    }
}

#[tauri::command(async)]
pub fn archive_list_files(path: String, password: String) -> ResultR<Vec<TreeNode<ArchiveEntry>>> {
    let path = std::path::Path::new(&path);
    let format = extutil::ArchiveFormat::from_path(path);
    println!("{:#?}", format);
    let mut handle = create_handler(path, format, password);
    handle.list_files()
}

#[tauri::command(async)]
pub fn archive_extract(path: String, target_path: String, password: String) -> ResultR<()> {
    let path = std::path::Path::new(&path);
    let target_path = std::path::Path::new(&target_path);
    let format = extutil::ArchiveFormat::from_path(path);
    println!("{:#?}", format);
    let mut handle = create_handler(path, format, password);
    handle.extract(target_path)
}
