import { invoke } from "@tauri-apps/api/core";

// 类型定义
export interface ArchiveEntry {
  name: string;
  path: string;
  parent_path: string | null;
  size: number;
  is_dir: boolean;
  modified: string | null;
}

export interface TreeNode<T> {
  item: T;
  children: TreeNode<T>[] | null;
}

// 命令调用函数
export async function archiveListFiles(
  path: string,
  password: string
): Promise<TreeNode<ArchiveEntry>[]> {
  return invoke<TreeNode<ArchiveEntry>[]>("archive_list_files", {
    path,
    password,
  });
}

export async function archiveExtract(
  path: string,
  targetPath: string,
  password: string
): Promise<void> {
  return invoke<void>("archive_extract", { path, targetPath, password });
}

export async function run_args(): Promise<string[]> {
  return invoke<string[]>("run_args");
}
