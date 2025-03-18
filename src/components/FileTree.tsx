import { Tree } from "react-arborist";
import close from "../assets/close.svg";
import open from "../assets/open.svg";
import file from "../assets/file.svg";
// import {
//   Dropdown,
//   DropdownTrigger,
//   DropdownMenu,
//   DropdownItem,
//   Chip,
// } from "@heroui/react";

interface FileNode {
  id: string;
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
  modified?: string;
  children?: FileNode[];
}

// 转换数据结构
function transformData(data: any[]): FileNode[] {
  // 先转换数据结构
  const transformedData = data.map((node) => ({
    id: node.item.path,
    name: node.item.name || node.item.path.split("/").slice(-2)[0],
    path: node.item.path,
    size: node.item.size,
    is_dir: node.item.is_dir,
    modified: node.item.modified,
    children: node.children ? transformData(node.children) : undefined,
  }));

  // 排序：目录(is_dir=true)排在前面，然后按照modified时间最新的排序
  return transformedData.sort((a, b) => {
    // 首先按照是否为目录排序（目录排在前面）
    if (a.is_dir && !b.is_dir) return -1;
    if (!a.is_dir && b.is_dir) return 1;

    // 然后按照modified时间排序（最新的排在前面）
    if (a.modified && b.modified) {
      return new Date(b.modified).getTime() - new Date(a.modified).getTime();
    }
    // 如果没有modified时间，保持原有顺序
    return 0;
  });
}

function FileTree({
  data,
  width,
  height,
}: {
  data: any[];
  width?: number | string;
  height?: number;
}) {
  const treeData = transformData(data);
  return (
    <Tree
      data={treeData}
      height={height}
      width={width}
      indent={24}
      padding={32}
      rowHeight={32}
      openByDefault={false} // 添加这行，默认折叠所有节点
    >
      {({ node, style, dragHandle }) => (
        <div
          style={style}
          ref={dragHandle}
          onClick={() => !node.isLeaf && node.toggle()}
          className={`flex gap-2 px-2 cursor-pointer justify-items-stretch items-center ${
            node.isSelected ? "bg-blue-100" : ""
          } ${!node.isLeaf ? "hover:bg-gray-100" : ""}`}
        >
          {node.level > 0 && (
            <span
              className="absolute w-4 h-px bg-black-200"
              style={{
                left: "-1px",
                top: "16px",
              }}
            />
          )}

          <span className="flex items-center z-10">
            {!node.data.is_dir ? (
              <img src={file} className="w-8 h-8" />
            ) : node.isOpen ? (
              <img src={open} className="w-8 h-8" />
            ) : (
              <img src={close} className="w-8 h-8" />
            )}
            <span
              title={node.data.name}
              className="ml-2 text-ellipsis overflow-hidden whitespace-nowrap max-w-[400px]"
            >
              {node.data.name}
            </span>
          </span>
          {node.data.is_dir && (
            <span className="text-[12px] m-1 text-gray-500 ml-auto">
              {node.data.modified?.replace(new Date().getFullYear() + "-", "")}
            </span>
          )}

          {!node.data.is_dir && (
            <span className="text-xs text-gray-500 ml-auto">
              <span className="text-[12px] m-1 text-gray-500 ml-auto">
                {node.data.modified?.replace(
                  new Date().getFullYear() + "-",
                  ""
                )}
              </span>
              <span className="text-[12px] m-1 text-blue-500 ml-auto">
                {node.data.size > 1024 * 1024
                  ? `${(node.data.size / (1024 * 1024)).toFixed(2)} MB`
                  : `${(node.data.size / 1024).toFixed(2)} KB`}
              </span>
            </span>
          )}
          {/* <Dropdown>
            <DropdownTrigger>
              <Chip size="sm">...</Chip>
            </DropdownTrigger>
            <DropdownMenu aria-label="Static Actions">
              <DropdownItem key="new">解压</DropdownItem>
            </DropdownMenu>
          </Dropdown> */}
        </div>
      )}
    </Tree>
  );
}

export default FileTree;
